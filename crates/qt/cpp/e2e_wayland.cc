#include <QAbstractButton>
#include <QApplication>
#include <QComboBox>
#include <QFile>
#include <QKeyEvent>
#include <QLineEdit>
#include <QMenu>
#include <QMenuBar>
#include <QMouseEvent>
#include <QScrollArea>
#include <QTextStream>
#include <QTreeWidget>
#include <QtTest/QTest>
#include <memory>

#include "app.h"
#include "color_picker.h"
#include "graph_canvas.h"
#include "main_window.h"
#include "opsis-qt/src/bridge.rs.h"
#include "scenario.h"
#include "start_screen.h"

namespace opsis {

namespace {

constexpr int kRealInputMs = 15'000;
constexpr int kProjectMs = 20'000;
constexpr int kSettleMs = 300;
constexpr int kCornerInset = 30;
constexpr double kTooltipY = 900;

QString Expect(bool ok, const QString& what) { return ok ? QString() : what; }

class InputLog : public QObject {
 public:
  using QObject::QObject;

  QString last() const {
    return QStringLiteral("the last press the app got: %1; the last key: %2")
        .arg(press_, key_);
  }

 protected:
  bool eventFilter(QObject* watched, QEvent* event) override {
    if (!watched->isWidgetType()) return false;
    const QString target =
        QString::fromLatin1(watched->metaObject()->className());
    if (event->type() == QEvent::MouseButtonPress) {
      const QPoint at =
          static_cast<QMouseEvent*>(event)->globalPosition().toPoint();
      press_ =
          QStringLiteral("%1 at %2,%3").arg(target).arg(at.x()).arg(at.y());
    } else if (event->type() == QEvent::KeyPress) {
      const auto* key = static_cast<QKeyEvent*>(event);
      key_ = QStringLiteral("%1 to %2")
                 .arg(QKeySequence(key->keyCombination()).toString(), target);
    }
    return false;
  }

 private:
  QString press_ = QStringLiteral("none");
  QString key_ = QStringLiteral("none");
};

void AskForRealInput(const QString& request) {
  QFile file(qEnvironmentVariable("OPSIS_QT_INPUT"));
  if (!file.open(QIODevice::Append | QIODevice::Text)) return;
  QTextStream(&file) << request << '\n';
}

QPoint OnScreen(const QWidget* widget, const QPoint& local) {
  const QWidget* top = widget->window();
  const QPoint frame =
      top->geometry().topLeft() - top->frameGeometry().topLeft();
  return widget->mapTo(top, local) + frame;
}

void RealClick(const QWidget* widget, const QPoint& local) {
  const QPoint at = OnScreen(widget, local);
  AskForRealInput(QStringLiteral("click %1 %2").arg(at.x()).arg(at.y()));
}

void RealClick(const QWidget* widget) {
  RealClick(widget, widget->rect().center());
}

void RealMove(const QWidget* widget, const QPoint& local) {
  const QPoint at = OnScreen(widget, local);
  AskForRealInput(QStringLiteral("move %1 %2").arg(at.x()).arg(at.y()));
}

void RealKey(const QString& combo) {
  AskForRealInput(QStringLiteral("key ") + combo);
}

void RealType(const QString& text) {
  AskForRealInput(QStringLiteral("type ") + text);
}

void ShowInItsScrollArea(QWidget* widget) {
  for (QWidget* up = widget->parentWidget(); up != nullptr;
       up = up->parentWidget()) {
    if (auto* area = qobject_cast<QScrollArea*>(up)) {
      area->ensureWidgetVisible(widget);
      return;
    }
  }
}

void Select(const QString& id) {
  rust::Vec<rust::String> ids;
  ids.push_back(ToRust(id));
  App::Get().session().select(std::move(ids));
}

QWidget* Popup() { return QApplication::activePopupWidget(); }

bool ShowsCardTitled(const QString& title) {
  for (const auto& card : App::Get().session().graph().nodes) {
    if (ToQ(card.title) == title) return true;
  }
  return false;
}

QMenuBar* MenuTitle(MainWindow* window, const QString& title, QPoint* at) {
  for (QMenuBar* bar : window->findChildren<QMenuBar*>()) {
    for (QAction* action : bar->actions()) {
      if (action->text().remove(QLatin1Char('&')) == title) {
        *at = bar->actionGeometry(action).center();
        return bar;
      }
    }
  }
  return nullptr;
}

QComboBox* KeyframesList(MainWindow* window) {
  for (QComboBox* combo : window->findChildren<QComboBox*>()) {
    if (combo->isVisible() &&
        combo->currentText() == QStringLiteral("No keyframes")) {
      return combo;
    }
  }
  return nullptr;
}

QLineEdit* NameField(MainWindow* window) {
  for (QLineEdit* line :
       window->inspector_dock()->widget()->findChildren<QLineEdit*>()) {
    if (line->isVisible() && !line->placeholderText().isEmpty() &&
        line->placeholderText() ==
            ToQ(App::Get().session().inspector().title)) {
      return line;
    }
  }
  return nullptr;
}

bool HoverCardShown() {
  for (QWidget* top : QApplication::topLevelWidgets()) {
    if (top->isVisible() && top->objectName() == QStringLiteral("HoverCard")) {
      return true;
    }
  }
  return false;
}

bool TimelinePoppedOut(MainWindow* window) {
  return window->timeline_dock()->isFloating() &&
         window->timeline_dock()->QWidget::window()->isVisible();
}

}

void Scenario::BuildWayland() {
  auto* input = new InputLog(this);
  qApp->installEventFilter(input);
  const QString projects = qEnvironmentVariable("OPSIS_QT_PROJECTS");
  MainWindow* window = window_;
  GraphCanvas* canvas = window_->canvas();
  auto count = std::make_shared<std::size_t>(0);
  const auto closes = [this, input](const QString& name, const QString& what) {
    Until(
        name, [] { RealKey(QStringLiteral("Escape")); }, kRealInputMs,
        [] { return Popup() == nullptr; },
        [input, what] {
          return Expect(Popup() == nullptr,
                        QStringLiteral("Escape didn't close %1; %2")
                            .arg(what, input->last()));
        });
  };
  Add(
      QStringLiteral("wayland-skip-setup"),
      [this] { Click(window_, QStringLiteral("Skip for now")); }, 500,
      [window] {
        return Expect(window->start_screen()->isVisible(),
                      QStringLiteral("skipping setup didn't show the start "
                                     "screen"));
      });
  Until(
      QStringLiteral("wayland-create-a-project"),
      [projects] {
        App::Get().session().create_project(ToRust(QStringLiteral("Wayland")),
                                            ToRust(QStringLiteral("todomvc")),
                                            ToRust(projects));
      },
      kProjectMs, [] { return !App::Get().session().graph().nodes.empty(); },
      [] {
        return Expect(!App::Get().session().graph().nodes.empty(),
                      QStringLiteral("no project opened"));
      });
  Until(
      QStringLiteral("wayland-menu-opens-from-a-real-click"),
      [this, window] {
        QPoint at;
        QMenuBar* bar = MenuTitle(window, QStringLiteral("File"), &at);
        if (bar == nullptr) {
          Miss(QStringLiteral("no File menu"));
          return;
        }
        RealClick(bar, at);
      },
      kRealInputMs, [] { return qobject_cast<QMenu*>(Popup()) != nullptr; },
      [input] {
        return Expect(
            qobject_cast<QMenu*>(Popup()) != nullptr,
            QStringLiteral("the File menu didn't open; %1").arg(input->last()));
      });
  closes(QStringLiteral("wayland-escape-closes-the-menu"),
         QStringLiteral("the menu"));
  Until(
      QStringLiteral("wayland-list-opens-from-a-real-click"),
      [this, window] {
        QComboBox* list = KeyframesList(window);
        if (list == nullptr) {
          Miss(QStringLiteral("no keyframes list"));
          return;
        }
        RealClick(list);
      },
      kRealInputMs, [] { return Popup() != nullptr; },
      [input] {
        return Expect(Popup() != nullptr,
                      QStringLiteral("the keyframes list didn't open; %1")
                          .arg(input->last()));
      });
  closes(QStringLiteral("wayland-escape-closes-the-list"),
         QStringLiteral("the list"));
  Until(
      QStringLiteral("wayland-colour-picker-opens-from-a-real-click"),
      [this, window] {
        try {
          Select(ToQ(App::Get().session().add_node(
              ToRust(QStringLiteral("text.tooltip")), 0, kTooltipY)));
        } catch (const rust::Error& error) {
          Miss(QString::fromUtf8(error.what()));
          return;
        }
        QTest::qWait(kSettleMs);
        QAbstractButton* pick = Find(window->inspector_dock()->widget(),
                                     QStringLiteral("Pick colour"));
        if (pick == nullptr) {
          Miss(QStringLiteral("no Pick colour button"));
          return;
        }
        ShowInItsScrollArea(pick);
        QTest::qWait(kSettleMs);
        RealClick(pick);
      },
      kRealInputMs,
      [] { return qobject_cast<ColorPicker*>(Popup()) != nullptr; },
      [input] {
        return Expect(qobject_cast<ColorPicker*>(Popup()) != nullptr,
                      QStringLiteral("the colour picker didn't open; %1")
                          .arg(input->last()));
      });
  closes(QStringLiteral("wayland-escape-closes-the-picker"),
         QStringLiteral("the picker"));
  Until(
      QStringLiteral("wayland-rename-by-typing"),
      [this, window] {
        Select(QStringLiteral("todo1"));
        QTest::qWait(kSettleMs);
        QLineEdit* name = NameField(window);
        if (name == nullptr) {
          Miss(QStringLiteral("no Name field"));
          return;
        }
        ShowInItsScrollArea(name);
        QTest::qWait(kSettleMs);
        RealClick(name);
        RealKey(QStringLiteral("ctrl+a"));
        RealType(QStringLiteral("Wayland name"));
        RealKey(QStringLiteral("Return"));
      },
      kRealInputMs,
      [] { return ShowsCardTitled(QStringLiteral("Wayland name")); },
      [input] {
        return Expect(ShowsCardTitled(QStringLiteral("Wayland name")),
                      QStringLiteral("typing didn't rename the node; %1")
                          .arg(input->last()));
      });
  Until(
      QStringLiteral("wayland-undo-with-real-keys"),
      [canvas] {
        RealClick(
            canvas->viewport(),
            QPoint(kCornerInset, canvas->viewport()->height() - kCornerInset));
        RealKey(QStringLiteral("ctrl+z"));
      },
      kRealInputMs,
      [] { return !ShowsCardTitled(QStringLiteral("Wayland name")); },
      [input] {
        return Expect(!ShowsCardTitled(QStringLiteral("Wayland name")),
                      QStringLiteral("Ctrl+Z didn't undo the rename; %1")
                          .arg(input->last()));
      });
  Until(
      QStringLiteral("wayland-copy-and-paste-with-real-keys"),
      [canvas, count] {
        *count = App::Get().session().graph().nodes.size();
        RealClick(canvas->viewport(),
                  canvas->ViewPointOf(QStringLiteral("todo2")));
        RealKey(QStringLiteral("ctrl+c"));
        RealKey(QStringLiteral("ctrl+v"));
      },
      kRealInputMs,
      [count] {
        return App::Get().session().graph().nodes.size() == *count + 1;
      },
      [count, input] {
        return Expect(App::Get().session().graph().nodes.size() == *count + 1,
                      QStringLiteral("Ctrl+C, Ctrl+V didn't add a copy; %1")
                          .arg(input->last()));
      });
  Until(
      QStringLiteral("wayland-hover-shows-the-card"),
      [this, window] {
        auto* tree = window->nodes_dock()->widget()->findChild<QTreeWidget*>();
        if (tree == nullptr || tree->topLevelItemCount() == 0) {
          Miss(QStringLiteral("no node list"));
          return;
        }
        QTreeWidgetItem* row = tree->topLevelItem(0)->child(0);
        RealMove(tree->viewport(), tree->visualItemRect(row).center());
      },
      kRealInputMs, [] { return HoverCardShown(); },
      [input] {
        return Expect(HoverCardShown(),
                      QStringLiteral("hovering a node showed no card; %1")
                          .arg(input->last()));
      });
  Until(
      QStringLiteral("wayland-pop-out-the-timeline"),
      [this, window] {
        QAbstractButton* out =
            Find(window, QStringLiteral("Open timeline in a new window"));
        if (out == nullptr) {
          Miss(QStringLiteral("no pop-out button"));
          return;
        }
        RealClick(out);
      },
      kRealInputMs, [window] { return TimelinePoppedOut(window); },
      [window, input] {
        return Expect(TimelinePoppedOut(window),
                      QStringLiteral("the timeline didn't open in its own "
                                     "window; %1")
                          .arg(input->last()));
      });
}

}
