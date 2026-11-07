#include <QAction>
#include <QApplication>
#include <QComboBox>
#include <QDialogButtonBox>
#include <QDir>
#include <QDropEvent>
#include <QFileDialog>
#include <QImage>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QLabel>
#include <QLineEdit>
#include <QListWidget>
#include <QMenu>
#include <QMenuBar>
#include <QMimeData>
#include <QPointer>
#include <QPushButton>
#include <QScrollArea>
#include <QScrollBar>
#include <QTextCursor>
#include <QTimer>
#include <QToolButton>
#include <QTreeWidget>
#include <QTreeWidgetItemIterator>
#include <QUrl>
#include <QtTest/QTest>
#include <algorithm>
#include <cmath>
#include <memory>

#include "app.h"
#include "color_picker.h"
#include "gradient_field.h"
#include "graph_canvas.h"
#include "main_window.h"
#include "marks_field.h"
#include "mixer_controls.h"
#include "node_list.h"
#include "opsis-qt/src/bridge.rs.h"
#include "playback_clock.h"
#include "scenario.h"
#include "start_screen.h"
#include "timeline_canvas.h"
#include "timeline_panel.h"
#include "top_bar.h"
#include "var_text.h"
#include "visual_fields.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr int kMenuLeftMost = 80;
constexpr int kDropSide = 64;
constexpr int kNudgeX = 120;
constexpr int kNudgeY = 60;
constexpr int kGrow = 60;
constexpr double kNear = 1.0;
constexpr int kSettleMs = 300;
constexpr double kMaxQuality = 12;
constexpr int kEventuallyMs = 2000;
constexpr int kControlsWidth = 220;
constexpr double kBottomShareLow = 0.3;
constexpr double kBottomShareHigh = 0.5;

struct Memo {
  std::size_t nodes = 0;
  QString key;
  QString id;
  QString source;
  QString source_port;
  QString target_port;
  QString other;
  double x = 0;
  double y = 0;
  double width = 0;
  std::size_t count = 0;
  std::size_t edges = 0;
};

bridge::GraphView Graph() { return App::Get().session().graph(); }

const bridge::CardView* CardTitled(const bridge::GraphView& graph,
                                   const QString& title) {
  for (const auto& card : graph.nodes) {
    if (ToQ(card.title) == title) return &card;
  }
  return nullptr;
}

const bridge::CardView* CardWithId(const bridge::GraphView& graph,
                                   const QString& id) {
  for (const auto& card : graph.nodes) {
    if (ToQ(card.id) == id) return &card;
  }
  return nullptr;
}

const bridge::CardView* CardWithId(bridge::GraphView&& graph,
                                   const QString& id) = delete;
const bridge::CardView* CardTitled(bridge::GraphView&& graph,
                                   const QString& title) = delete;

QString Expect(bool ok, const QString& what) { return ok ? QString() : what; }

int BackwardEdges(const bridge::GraphView& graph) {
  int backward = 0;
  for (const auto& edge : graph.edges) {
    const auto* from = CardWithId(graph, ToQ(edge.source));
    const auto* to = CardWithId(graph, ToQ(edge.target));
    if (from == nullptr || to == nullptr || !to->parent.empty() ||
        !from->parent.empty()) {
      continue;
    }
    if (to->x < from->x + from->width) ++backward;
  }
  return backward;
}

QString FieldText(const QString& key) {
  for (const auto& field : App::Get().session().inspector().fields) {
    if (ToQ(field.key) == key) {
      return field.kind == bridge::FieldKind::Number ||
                     field.kind == bridge::FieldKind::PresetNumber
                 ? QString::number(field.number)
                 : ToQ(field.text);
    }
  }
  return QStringLiteral("<missing>");
}

int ListLength(bridge::FieldKind kind) {
  for (const auto& field : App::Get().session().inspector().fields) {
    if (field.kind == kind && field.group.empty()) {
      return static_cast<int>(
          QJsonDocument::fromJson(ToQ(field.value_json).toUtf8())
              .array()
              .size());
    }
  }
  return -1;
}

QWidget* VisibleText(QWidget* root, const QString& text) {
  for (QLabel* label : root->findChildren<QLabel*>()) {
    if (label->isVisible() && label->text() == text) return label;
  }
  return nullptr;
}

}

QAbstractButton* Scenario::Find(QWidget* root, const QString& name) {
  for (QAbstractButton* button : root->findChildren<QAbstractButton*>()) {
    if (!button->isVisible() || button->width() <= 0 || button->height() <= 0) {
      continue;
    }
    if (button->text() == name || button->toolTip() == name ||
        button->toolTip().startsWith(name + QStringLiteral("  "))) {
      return button;
    }
    for (QLabel* label : button->findChildren<QLabel*>()) {
      if (label->text() == name) return button;
    }
  }
  return nullptr;
}

void Scenario::Click(QWidget* root, const QString& name) {
  QAbstractButton* button = Find(root, name);
  if (button == nullptr) {
    Miss(QStringLiteral("no '%1' to click").arg(name));
    return;
  }
  QTest::mouseClick(button, Qt::LeftButton, {}, button->rect().center());
}

void Scenario::ClickLater(QWidget* root, const QString& name) {
  QAbstractButton* button = Find(root, name);
  if (button == nullptr) {
    Miss(QStringLiteral("no '%1' to click").arg(name));
    return;
  }
  QTimer::singleShot(0, button, [button] {
    QTest::mouseClick(button, Qt::LeftButton, {}, button->rect().center());
  });
}

void Scenario::ClickAt(QWidget* target, QPoint at) {
  QTest::mouseMove(target, at);
  QTest::mouseClick(target, Qt::LeftButton, {}, at);
}

void Scenario::ClickMenu(const QString& section, const QString& item) {
  auto* bar = window_->findChild<QMenuBar*>();
  QAction* top = nullptr;
  for (QAction* action : bar->actions()) {
    if (action->text() == section) top = action;
  }
  if (top == nullptr || top->menu() == nullptr) {
    Miss(QStringLiteral("no '%1' menu").arg(section));
    return;
  }
  QTest::mouseClick(bar, Qt::LeftButton, {}, bar->actionGeometry(top).center());
  QMenu* menu = top->menu();
  QAction* chosen = nullptr;
  for (QAction* action : menu->actions()) {
    if (action->text() == item) chosen = action;
  }
  if (chosen == nullptr || !menu->isVisible()) {
    menu->close();
    Miss(QStringLiteral("no '%1' in the %2 menu").arg(item, section));
    return;
  }
  const QPoint at = menu->actionGeometry(chosen).center();
  QTimer::singleShot(
      0, menu, [menu, at] { QTest::mouseClick(menu, Qt::LeftButton, {}, at); });
}

void Scenario::BuildTour() {
  MainWindow* window = window_;
  GraphCanvas* canvas = window->canvas();
  QWidget* view = canvas->viewport();
  auto memo = std::make_shared<Memo>();
  const auto undo = [this, window] { Click(window, QStringLiteral("Undo")); };

  Add(
      QStringLiteral("start-screen"), [] {}, 300,
      [window] {
        QString problems =
            Expect(window->start_screen()->isVisible(),
                   QStringLiteral("the start screen isn't showing"));
        for (QFrame* rule : window->menuWidget()->findChildren<QFrame*>()) {
          if (rule->isVisible() &&
              rule->property("role") == QStringLiteral("vrule") &&
              rule->x() > window->width() / 2 &&
              rule->x() < window->width() - kControlsWidth) {
            problems +=
                QStringLiteral(" a divider sits at %1 px").arg(rule->x());
          }
        }
        for (QMenuBar* menus :
             window->menuWidget()->findChildren<QMenuBar*>()) {
          if (menus->isVisible() && menus->x() > kMenuLeftMost) {
            problems +=
                QStringLiteral(" the menu sits at %1 px").arg(menus->x());
          }
        }
        return problems.trimmed();
      });
  Add(
      QStringLiteral("pick-sample-template"),
      [this, window] {
        ClickLater(window, QStringLiteral("Sample: to-do app"));
      },
      900,
      [] {
        QWidget* dialog = QApplication::activeModalWidget();
        return Expect(
            dialog != nullptr &&
                dialog->objectName() == QStringLiteral("NewProjectDialog"),
            QStringLiteral("the new project dialog didn't open"));
      });
  Add(
      QStringLiteral("create-sample-project"),
      [this] {
        QWidget* dialog = QApplication::activeModalWidget();
        if (dialog == nullptr) {
          Miss(QStringLiteral("no dialog"));
          return;
        }
        const QList<QLineEdit*> lines = dialog->findChildren<QLineEdit*>();
        if (lines.size() < 2) {
          Miss(QStringLiteral("the dialog has no name and location fields"));
          return;
        }
        for (int i = 0; i < 2; ++i) {
          QLineEdit* line = lines.at(i);
          line->setFocus();
          QTest::keyClick(line, Qt::Key_A, Qt::ControlModifier);
          QTest::keyClicks(
              line, i == 0 ? QStringLiteral("Shop demo")
                           : QDir(qEnvironmentVariable("OPSIS_USER_DATA"))
                                 .filePath(QStringLiteral("projects")));
        }
        Click(dialog, QStringLiteral("Create"));
      },
      2500,
      [] {
        const bridge::GraphView graph = Graph();
        return Expect(ToQ(graph.name) == QStringLiteral("Shop demo") &&
                          !graph.nodes.empty(),
                      QStringLiteral("the project '%1' with %2 nodes is open")
                          .arg(ToQ(graph.name))
                          .arg(graph.nodes.size()));
      });
  Add(
      QStringLiteral("top-bar"), [] {}, 200,
      [window] { return window->top_bar()->Problems(); });

  Add(
      QStringLiteral("bottom-panel-height"), [] {}, 100,
      [window] {
        const double dock = window->timeline_dock()->height();
        const double column = dock + window->canvas()->height();
        const double share = column > 0 ? dock / column : 0;
        return Expect(share > kBottomShareLow && share < kBottomShareHigh,
                      QStringLiteral("the bottom panel takes %1% of the column")
                          .arg(std::lround(share * 100)));
      });
  Add(
      QStringLiteral("pop-out-timeline"),
      [this, window] {
        Click(window, QStringLiteral("Open timeline in a new window"));
      },
      800,
      [window] {
        return Expect(
            window->timeline_dock()->isFloating(),
            QStringLiteral("the timeline didn't open in its own window"));
      });
  Add(
      QStringLiteral("dock-timeline-back"),
      [this, window] {
        QWidget* floating = window->timeline_dock()->QWidget::window();
        QWidget* title = nullptr;
        for (QWidget* child : floating->findChildren<QWidget*>()) {
          if (child->isVisible() && child->height() > 0 &&
              QString::fromLatin1(child->metaObject()->className())
                  .contains(QStringLiteral("TitleBar"))) {
            title = child;
            break;
          }
        }
        if (title == nullptr) {
          Miss(QStringLiteral("the floating timeline has no title bar"));
          return;
        }
        QTest::mouseDClick(title, Qt::LeftButton, {},
                           QPoint(title->width() / 3, title->height() / 2));
      },
      800,
      [window] {
        return Expect(
            !window->timeline_dock()->isFloating(),
            QStringLiteral("double-clicking the title didn't dock it"));
      });
  Add(
      QStringLiteral("collapse-bottom-panel"),
      [this, window] {
        Click(window, QStringLiteral("Hide the bottom panel"));
      },
      600,
      [window] {
        return Expect(!window->timeline_dock()->isOpen() &&
                          !window->timeline_dock()->isVisible(),
                      QStringLiteral("the bottom panel is still open"));
      });
  Add(
      QStringLiteral("show-bottom-panel-from-top-bar"),
      [this, window] { Click(window, QStringLiteral("Show bottom panel")); },
      600,
      [window] {
        return Expect(window->timeline_dock()->isOpen(),
                      QStringLiteral("the bottom panel didn't come back"));
      });
  Add(
      QStringLiteral("search-node-list"),
      [window] {
        auto* search = window->nodes_dock()->widget()->findChild<QLineEdit*>();
        search->setFocus();
        QTest::keyClicks(search, QStringLiteral("zzzqqq"));
      },
      300,
      [window] {
        QWidget* list = window->nodes_dock()->widget();
        QString problems = qobject_cast<NodeList*>(list)->Problems();
        if (VisibleText(list, QStringLiteral("No nodes found.")) == nullptr) {
          problems += QStringLiteral(" no 'No nodes found.'");
        }
        return problems.trimmed();
      });
  Add(QStringLiteral("clear-node-list-search"), [window] {
    auto* search = window->nodes_dock()->widget()->findChild<QLineEdit*>();
    QTest::keyClick(search, Qt::Key_A, Qt::ControlModifier);
    QTest::keyClick(search, Qt::Key_Backspace);
  });
  Add(
      QStringLiteral("hover-node-list-row"),
      [window] {
        auto* tree = window->nodes_dock()->widget()->findChild<QTreeWidget*>();
        QTreeWidgetItem* row = tree->topLevelItem(0)->child(0);
        QTest::mouseMove(tree->viewport(), tree->visualItemRect(row).center());
      },
      900,
      [] {
        for (QWidget* top : QApplication::topLevelWidgets()) {
          if (top->isVisible() &&
              top->objectName() == QStringLiteral("HoverCard")) {
            return QString();
          }
        }
        return QStringLiteral("no hover card");
      });
  Add(
      QStringLiteral("add-node-from-list"),
      [window, memo] {
        memo->count = Graph().nodes.size();
        auto* tree = window->nodes_dock()->widget()->findChild<QTreeWidget*>();
        QTreeWidgetItem* row = tree->topLevelItem(0)->child(0);
        QTest::mouseClick(tree->viewport(), Qt::LeftButton, {},
                          tree->visualItemRect(row).center());
      },
      700,
      [memo] {
        return Expect(Graph().nodes.size() == memo->count + 1,
                      QStringLiteral("no node was added"));
      });
  Add(QStringLiteral("undo-add-with-top-bar"), undo, 500, [memo] {
    return Expect(Graph().nodes.size() == memo->count,
                  QStringLiteral("Undo didn't remove the node"));
  });

  Add(
      QStringLiteral("project-canvas-toggle"),
      [this, window, view] {
        ClickAt(view, QPoint(4, 4));
        QTimer::singleShot(300, window, [this, window] {
          Click(window->inspector_dock()->widget(), QStringLiteral("1:1"));
        });
      },
      900,
      [] {
        const bridge::ProjectView project = App::Get().session().project();
        const QString problem =
            Expect(project.width == project.height,
                   QStringLiteral("the canvas is %1 × %2, not square")
                       .arg(project.width)
                       .arg(project.height));
        return problem;
      });
  Add(QStringLiteral("back-to-16-9"), [this, window] {
    Click(window->inspector_dock()->widget(), QStringLiteral("16:9"));
  });
  Add(
      QStringLiteral("select-card"),
      [this, canvas, view, memo] {
        const bridge::GraphView graph = Graph();
        for (const auto& edge : graph.edges) {
          const auto* from = CardWithId(graph, ToQ(edge.source));
          const auto* to = CardWithId(graph, ToQ(edge.target));
          if (from != nullptr && to != nullptr &&
              to->kind == bridge::CardKind::Card &&
              from->kind == bridge::CardKind::Card) {
            memo->id = ToQ(edge.target);
            memo->source = ToQ(edge.source);
            memo->source_port = ToQ(edge.source_handle);
            memo->target_port = ToQ(edge.target_handle);
            break;
          }
        }
        if (memo->id.isEmpty()) {
          Miss(QStringLiteral("no connected cards"));
          return;
        }
        Click(canvas, QStringLiteral("Fit to view"));
        ClickAt(view, canvas->ViewPointOf(memo->id));
      },
      900,
      [window, memo] {
        QString problems;
        if (ToQ(App::Get().session().selected()) != memo->id) {
          problems += QStringLiteral("the card isn't selected; ");
        }
        if (ToQ(App::Get().session().inspector().id) != memo->id) {
          problems += QStringLiteral("the Inspector doesn't show it");
        }
        QWidget* panel = window->inspector_dock()->widget();
        for (QWidget* child : panel->findChildren<QWidget*>()) {
          if (!child->isVisible() || child->width() <= 0) continue;
          const int right = child->mapTo(panel, QPoint(child->width(), 0)).x();
          if (right > panel->width() + 1) {
            problems += QStringLiteral(
                            "something in the Inspector runs %1 px past "
                            "its edge")
                            .arg(right - panel->width());
            break;
          }
        }
        return problems;
      });
  Add(
      QStringLiteral("drag-card"),
      [this, canvas, view, memo] {
        const bridge::GraphView graph_1 = Graph();
        const auto* card = CardWithId(graph_1, memo->id);
        if (card == nullptr) {
          Miss(QStringLiteral("no card"));
          return;
        }
        memo->x = card->x;
        memo->y = card->y;
        const QPoint from = canvas->ViewPointOf(memo->id);
        Drag(view, from, from + QPoint(kNudgeX, kNudgeY));
      },
      700,
      [memo] {
        const bridge::GraphView graph_2 = Graph();
        const auto* card = CardWithId(graph_2, memo->id);
        const bool moved = card != nullptr && card->x > memo->x + kNear &&
                           card->y > memo->y + kNear;
        return Expect(moved,
                      QStringLiteral("the saved position didn't change"));
      });
  Add(
      QStringLiteral("undo-drag-with-keys"),
      [window, canvas] {
        window->activateWindow();
        QTest::qWaitForWindowActive(window);
        canvas->setFocus();
        QTest::keyClick(canvas, Qt::Key_Z, Qt::ControlModifier);
      },
      500,
      [memo] {
        const bridge::GraphView graph_3 = Graph();
        const auto* card = CardWithId(graph_3, memo->id);
        return Expect(card != nullptr && std::abs(card->x - memo->x) < kNear,
                      QStringLiteral("Ctrl+Z didn't move it back"));
      });

  Add(
      QStringLiteral("type-in-variable-field"),
      [this, window] {
        auto* surface =
            window->inspector_dock()->widget()->findChild<VarSurface*>();
        if (surface == nullptr) {
          Miss(QStringLiteral("no variable field"));
          return;
        }
        QTest::mouseClick(surface->viewport(), Qt::LeftButton, {},
                          surface->viewport()->rect().center());
        QTest::keyClick(surface, Qt::Key_End);
        QTest::keyClicks(surface, QStringLiteral("home"));
        QTest::keyClick(surface, Qt::Key_Return);
      },
      600,
      [] {
        for (const auto& field : App::Get().session().inspector().fields) {
          if (ToQ(field.text).endsWith(QStringLiteral("/#/home")) &&
              ToQ(field.text).startsWith(QStringLiteral("{{Site address}}"))) {
            return QString();
          }
        }
        return QStringLiteral(
            "the page address wasn't saved with its variable");
      });
  Add(QStringLiteral("undo-typing"), undo, 500, [window] {
    for (VarSurface* surface :
         window->inspector_dock()->widget()->findChildren<VarSurface*>()) {
      if (surface->isVisible() &&
          surface->Value().endsWith(QStringLiteral("home"))) {
        return QStringLiteral("the Inspector still shows the typed address");
      }
    }
    return QString();
  });

  Add(
      QStringLiteral("rename-in-inspector"),
      [this, window, memo] {
        QTest::qWait(kSettleMs);
        QLineEdit* name = nullptr;
        for (QLineEdit* line :
             window->inspector_dock()->widget()->findChildren<QLineEdit*>()) {
          if (line->isVisible() && !line->placeholderText().isEmpty() &&
              line->placeholderText() ==
                  ToQ(App::Get().session().inspector().title)) {
            name = line;
          }
        }
        if (name == nullptr) {
          Miss(QStringLiteral("no Name field"));
          return;
        }
        QTest::mouseClick(name, Qt::LeftButton);
        if (!QTest::qWaitFor([name] { return name->hasFocus(); }, kSettleMs)) {
          Miss(QStringLiteral("the Name field didn't take focus"));
          return;
        }
        QTest::keyClick(name, Qt::Key_A, Qt::ControlModifier);
        QTest::keyClicks(name, QStringLiteral("Shop page"));
        QTest::keyClick(name, Qt::Key_Return);
      },
      600,
      [memo] {
        QTest::qWaitFor(
            [memo] {
              const bridge::GraphView graph_4 = Graph();
              const auto* card = CardWithId(graph_4, memo->id);
              return card != nullptr &&
                     ToQ(card->title) == QStringLiteral("Shop page");
            },
            kEventuallyMs);
        const bridge::GraphView graph_5 = Graph();
        const auto* card = CardWithId(graph_5, memo->id);
        return Expect(
            card != nullptr && ToQ(card->title) == QStringLiteral("Shop page"),
            QStringLiteral(
                "the card wasn't renamed: title '%1', inspector '%2'")
                .arg(card != nullptr ? ToQ(card->title) : QString(),
                     ToQ(App::Get().session().inspector().label)));
      });
  Add(QStringLiteral("undo-rename"), undo, 500, [window] {
    for (QLineEdit* line :
         window->inspector_dock()->widget()->findChildren<QLineEdit*>()) {
      if (line->isVisible() && line->text() == QStringLiteral("Shop page")) {
        return QStringLiteral("the Inspector still shows the new name");
      }
    }
    return QString();
  });
  Add(
      QStringLiteral("skip-node-switch"),
      [this, window] {
        QWidget* inspector = window->inspector_dock()->widget();
        QWidget* label =
            VisibleText(inspector, QStringLiteral("Skip this node"));
        auto* toggle = label != nullptr
                           ? label->parentWidget()->findChild<ToggleSwitch*>()
                           : nullptr;
        if (toggle == nullptr) {
          Miss(QStringLiteral("no Skip this node switch"));
          return;
        }
        QTest::mouseClick(toggle, Qt::LeftButton);
      },
      600,
      [memo] {
        const bridge::GraphView graph_6 = Graph();
        const auto* card = CardWithId(graph_6, memo->id);
        return Expect(card != nullptr && card->disabled,
                      QStringLiteral("the card isn't skipped"));
      });
  Add(QStringLiteral("undo-skip"), undo, 500, [memo] {
    const bridge::GraphView graph_7 = Graph();
    const auto* card = CardWithId(graph_7, memo->id);
    return Expect(card != nullptr && !card->disabled,
                  QStringLiteral("Undo didn't bring the card back"));
  });
  Add(
      QStringLiteral("delete-from-inspector"),
      [this, window, memo] {
        memo->count = Graph().nodes.size();
        Click(window->inspector_dock()->widget(), QStringLiteral("Delete"));
      },
      600,
      [memo] {
        return Expect(Graph().nodes.size() + 1 == memo->count &&
                          App::Get().session().inspector().id.empty(),
                      QStringLiteral("Delete didn't remove the card"));
      });
  Add(QStringLiteral("undo-delete"), undo, 600, [memo] {
    return Expect(Graph().nodes.size() == memo->count,
                  QStringLiteral("Undo didn't bring the card back"));
  });
  Add(
      QStringLiteral("reselect-card"),
      [this, view, canvas, memo] {
        ClickAt(view, canvas->ViewPointOf(memo->id));
      },
      500,
      [memo] {
        return Expect(ToQ(App::Get().session().selected()) == memo->id,
                      QStringLiteral("the card isn't selected again"));
      });
  Add(
      QStringLiteral("remove-edge-with-keys"),
      [this, canvas, view, memo] {
        const bridge::GraphView graph = Graph();
        memo->edges = graph.edges.size();
        QString into;
        for (const auto& edge : graph.edges) {
          if (ToQ(edge.target) == memo->id) into = ToQ(edge.id);
        }
        if (into.isEmpty()) {
          Miss(QStringLiteral("no edge into the card"));
          return;
        }
        ClickAt(view, canvas->ViewPointOfEdge(into));
        canvas->setFocus();
        QTest::keyClick(canvas, Qt::Key_Delete);
      },
      600,
      [memo] {
        return Expect(
            Graph().edges.size() + 1 == memo->edges,
            QStringLiteral("clicking the edge and Delete didn't remove it"));
      });
  Add(
      QStringLiteral("reconnect-by-drag"),
      [this, canvas, view, memo] {
        if (memo->source.isEmpty()) {
          Miss(QStringLiteral("no edge to put back"));
          return;
        }
        Drag(view,
             canvas->ViewPointOfPort(memo->source, memo->source_port, true),
             canvas->ViewPointOfPort(memo->id, memo->target_port, false), 20);
      },
      700,
      [memo] {
        return Expect(Graph().edges.size() == memo->edges,
                      QStringLiteral("dragging port to port didn't connect"));
      });

  Add(
      QStringLiteral("copy-paste-card"),
      [this, canvas, view, memo] {
        memo->count = Graph().nodes.size();
        ClickAt(view, canvas->ViewPointOf(memo->id));
        canvas->setFocus();
        QTest::keyClick(canvas, Qt::Key_C, Qt::ControlModifier);
        QTest::keyClick(canvas, Qt::Key_V, Qt::ControlModifier);
      },
      700,
      [memo] {
        return Expect(Graph().nodes.size() == memo->count + 1,
                      QStringLiteral("Ctrl+C, Ctrl+V didn't add a copy"));
      });
  Add(QStringLiteral("undo-paste"), undo, 500, [memo] {
    return Expect(Graph().nodes.size() == memo->count,
                  QStringLiteral("Undo didn't remove the copy"));
  });

  Add(
      QStringLiteral("select-all-from-menu"),
      [this] {
        ClickMenu(QStringLiteral("Edit"), QStringLiteral("Select all"));
      },
      600,
      [canvas, memo] {
        memo->count = Graph().nodes.size();
        const auto selected = canvas->scene()->selectedItems().size();
        return Expect(
            static_cast<std::size_t>(selected) >= memo->count,
            QStringLiteral("%1 of %2 selected").arg(selected).arg(memo->count));
      });
  Add(
      QStringLiteral("multi-select-shows-project-panel"), [] {}, 300,
      [this, window] {
        QWidget* panel = window->inspector_dock()->widget();
        bool counted = false;
        for (QLabel* label : panel->findChildren<QLabel*>()) {
          counted = counted ||
                    (label->isVisible() &&
                     label->text().endsWith(QStringLiteral("nodes selected")));
        }
        return Expect(counted && Find(panel, QStringLiteral(
                                                 "Group selection")) != nullptr,
                      QStringLiteral("no group prompt for the selection"));
      });
  Add(
      QStringLiteral("group-from-menu"),
      [this] {
        ClickMenu(QStringLiteral("Edit"), QStringLiteral("Group selection"));
      },
      800,
      [canvas] {
        return Expect(!canvas->GroupIds().isEmpty(),
                      QStringLiteral("no group was made"));
      });
  Add(
      QStringLiteral("drag-group-by-its-tag"),
      [this, canvas, view, memo] {
        const QStringList groups = canvas->GroupIds();
        if (groups.isEmpty()) return;
        memo->other = groups.first();
        const bridge::GraphView graph = Graph();
        const auto* group = CardWithId(graph, memo->other);
        const auto* child = CardWithId(graph, memo->id);
        if (group == nullptr || child == nullptr) {
          Miss(QStringLiteral("no group with the card"));
          return;
        }
        memo->x = child->x;
        memo->y = child->y;
        ClickAt(view, QPoint(4, 4));
        const QPoint tag =
            canvas->mapFromScene(QPointF(group->x + 20, group->y + 10));
        Drag(view, tag, tag + QPoint(kNudgeX, 0));
      },
      1200,
      [memo] {
        const bridge::GraphView graph_8 = Graph();
        const auto* child = CardWithId(graph_8, memo->id);
        return Expect(child != nullptr && child->x > memo->x + kNear,
                      QStringLiteral("the group's card didn't move with it"));
      });
  Add(
      QStringLiteral("resize-group-by-its-corner"),
      [this, canvas, view, memo] {
        const bridge::GraphView graph = Graph();
        const auto* group = CardWithId(graph, memo->other);
        const auto* child = CardWithId(graph, memo->id);
        if (group == nullptr || child == nullptr) {
          Miss(QStringLiteral("no group"));
          return;
        }
        memo->x = child->x;
        memo->y = child->y;
        memo->width = group->width;
        const QPoint tag =
            canvas->mapFromScene(QPointF(group->x + 20, group->y + 10));
        ClickAt(view, tag);
        const QPoint corner = canvas->mapFromScene(
            QPointF(group->x + group->width, group->y + group->height));
        QTimer::singleShot(200, view, [this, view, corner] {
          Drag(view, corner, corner + QPoint(kGrow, 0));
        });
      },
      900,
      [canvas, memo] {
        const bridge::GraphView graph = Graph();
        const auto* group = CardWithId(graph, memo->other);
        const auto* child = CardWithId(graph, memo->id);
        if (group == nullptr || child == nullptr) return QStringLiteral("gone");
        const double grown = group->width - memo->width;
        const double scale = canvas->zoom();
        QString problems;
        if (std::abs(grown - kGrow / scale) > kNear * 3) {
          problems += QStringLiteral("the group grew %1, not %2; ")
                          .arg(grown)
                          .arg(kGrow / scale);
        }
        if (std::abs(child->x - memo->x) > kNear) {
          problems += QStringLiteral("its card moved");
        }
        return problems;
      });
  Add(
      QStringLiteral("undo-group-work"),
      [this, window] {
        for (int i = 0; i < 3; ++i) Click(window, QStringLiteral("Undo"));
      },
      700,
      [canvas] {
        return Expect(canvas->GroupIds().isEmpty(),
                      QStringLiteral("three Undos didn't remove the group"));
      });

  Add(
      QStringLiteral("tidy-up-from-top-bar"),
      [this, window] { Click(window, QStringLiteral("Tidy up layout")); }, 900,
      [] {
        const int backward = BackwardEdges(Graph());
        return Expect(
            backward == 0,
            QStringLiteral("%1 edges run right to left").arg(backward));
      });
  Add(QStringLiteral("undo-tidy-up"), undo, 500);

  Add(
      QStringLiteral("hover-card-plays-motion"),
      [canvas, view, memo] {
        canvas->FitAll();
        for (const auto& card : Graph().nodes) {
          if (!card.motion.empty()) {
            memo->other = ToQ(card.id);
            break;
          }
        }
        QTest::mouseMove(view, QPoint(2, 2));
        QTest::mouseMove(view, canvas->ViewPointOf(memo->other));
      },
      200,
      [canvas, memo] {
        return Expect(canvas->MotionElapsed(memo->other) >= 0,
                      QStringLiteral("hovering didn't play the motion"));
      });

  Add(
      QStringLiteral("run-from-top-bar"),
      [this, window] { Click(window, QStringLiteral("Run")); }, 2500,
      [] {
        for (const auto& card : Graph().nodes) {
          if (!card.badge.empty()) return QString();
        }
        return QStringLiteral("no card shows a run status");
      });
  Add(QStringLiteral("stop-run"), [this, window] {
    if (Find(window, QStringLiteral("Stop")) != nullptr) {
      Click(window, QStringLiteral("Stop"));
    }
  });

  Add(
      QStringLiteral("quick-add"),
      [this, window, memo] {
        memo->count = Graph().nodes.size();
        ClickLater(window, QStringLiteral("Add node"));
      },
      700,
      [] {
        QWidget* dialog = QApplication::activeModalWidget();
        return Expect(dialog != nullptr &&
                          dialog->objectName() == QStringLiteral("QuickAdd"),
                      QStringLiteral("Quick add didn't open"));
      });
  Add(
      QStringLiteral("quick-add-pick"),
      [this] {
        QWidget* dialog = QApplication::activeModalWidget();
        if (dialog == nullptr) {
          Miss(QStringLiteral("no Quick add"));
          return;
        }
        auto* search = dialog->findChild<QLineEdit*>();
        QTest::keyClicks(search, QStringLiteral("wait"));
        auto* tree = dialog->findChild<QTreeWidget*>();
        for (QTreeWidgetItemIterator it(tree,
                                        QTreeWidgetItemIterator::NotHidden);
             *it != nullptr; ++it) {
          if ((*it)->childCount() == 0) {
            QTest::mouseClick(tree->viewport(), Qt::LeftButton, {},
                              tree->visualItemRect(*it).center());
            return;
          }
        }
        Miss(QStringLiteral("no match for 'wait'"));
      },
      700,
      [memo] {
        return Expect(Graph().nodes.size() == memo->count + 1 &&
                          QApplication::activeModalWidget() == nullptr,
                      QStringLiteral("picking in Quick add didn't add a node"));
      });
  Add(QStringLiteral("undo-quick-add"), undo, 500);

  const auto add_from_list = [this, window, memo](const QString& section,
                                                  const QString& title) {
    memo->nodes = Graph().nodes.size();
    auto* tree = window->nodes_dock()->widget()->findChild<QTreeWidget*>();
    for (int i = 0; i < tree->topLevelItemCount(); ++i) {
      QTreeWidgetItem* group = tree->topLevelItem(i);
      if (group->text(0) != section) continue;
      for (int j = 0; j < group->childCount(); ++j) {
        if (group->child(j)->text(0) != title) continue;
        tree->scrollToItem(group->child(j));
        QTest::mouseClick(tree->viewport(), Qt::LeftButton, {},
                          tree->visualItemRect(group->child(j)).center());
        return;
      }
    }
    Miss(QStringLiteral("no %1 › %2 in the node list").arg(section, title));
  };
  const auto back = [memo] {
    const std::size_t nodes = Graph().nodes.size();
    return Expect(nodes == memo->nodes,
                  QStringLiteral("%1 nodes after the undo, not %2")
                      .arg(nodes)
                      .arg(memo->nodes));
  };
  Add(
      QStringLiteral("size-preset"),
      [this, window, canvas, view, memo] {
        const bridge::GraphView graph = Graph();
        const auto* card = CardTitled(graph, QStringLiteral("Open browser"));
        if (card == nullptr) {
          Miss(QStringLiteral("no Open browser card"));
          return;
        }
        memo->id = ToQ(card->id);
        Click(canvas, QStringLiteral("Fit to view"));
        ClickAt(view, canvas->ViewPointOf(memo->id));
        QTest::qWait(kSettleMs);
        Click(window->inspector_dock()->widget(), QStringLiteral("1280 × 720"));
      },
      700,
      [] {
        return Expect(FieldText(QStringLiteral("viewportWidth")) ==
                          QStringLiteral("1280"),
                      QStringLiteral("the size preset left the width at %1")
                          .arg(FieldText(QStringLiteral("viewportWidth"))));
      });
  Add(
      QStringLiteral("advanced-tab-timezone"),
      [this, window] {
        QWidget* panel = window->inspector_dock()->widget();
        QAbstractButton* tab = nullptr;
        for (QAbstractButton* button :
             panel->findChildren<QAbstractButton*>()) {
          if (button->isVisible() &&
              button->text().startsWith(QStringLiteral("Advanced"))) {
            tab = button;
          }
        }
        if (tab == nullptr) {
          Miss(QStringLiteral("no Advanced tab"));
          return;
        }
        QTest::mouseClick(tab, Qt::LeftButton);
        for (QComboBox* combo : panel->findChildren<QComboBox*>()) {
          if (combo->isVisible() && combo->count() > 1 &&
              combo->itemText(0) ==
                  QStringLiteral("This computer's time zone")) {
            combo->setFocus();
            QTest::keyClick(combo, Qt::Key_Down);
            return;
          }
        }
        Miss(QStringLiteral("no time zone list in Advanced"));
      },
      700,
      [] {
        return Expect(!FieldText(QStringLiteral("timezone")).isEmpty(),
                      QStringLiteral("no time zone was chosen"));
      });
  Add(QStringLiteral("undo-timezone-and-size"), [this, window] {
    Click(window, QStringLiteral("Undo"));
    Click(window, QStringLiteral("Undo"));
    for (QAbstractButton* button :
         window->inspector_dock()->widget()->findChildren<QAbstractButton*>()) {
      if (button->isVisible() && button->text() == QStringLiteral("Basic")) {
        QTest::mouseClick(button, Qt::LeftButton);
      }
    }
  });
  Add(
      QStringLiteral("keys-field"),
      [this, window, add_from_list] {
        add_from_list(QStringLiteral("Terminal"), QStringLiteral("Press keys"));
        QTest::qWait(kSettleMs);
        for (QPushButton* button :
             window->inspector_dock()->widget()->findChildren<QPushButton*>()) {
          if (button->isVisible() &&
              button->property("variant") == QStringLiteral("keys")) {
            QTest::mouseClick(button, Qt::LeftButton);
            QTest::keyClick(button, Qt::Key_K,
                            Qt::ControlModifier | Qt::ShiftModifier);
            return;
          }
        }
        Miss(QStringLiteral("no keys field"));
      },
      700,
      [] {
        return Expect(FieldText(QStringLiteral("keys")) ==
                          QStringLiteral("Control+Shift+K"),
                      QStringLiteral("the keys field holds '%1'")
                          .arg(FieldText(QStringLiteral("keys"))));
      });
  Add(
      QStringLiteral("undo-keys"),
      [this, window] {
        Click(window, QStringLiteral("Undo"));
        Click(window, QStringLiteral("Undo"));
      },
      500, back);
  Add(
      QStringLiteral("backdrop-select"),
      [this, window, add_from_list] {
        add_from_list(QStringLiteral("Record"), QStringLiteral("Screenshot"));
        QTest::qWait(kSettleMs);
        for (QComboBox* combo :
             window->inspector_dock()->widget()->findChildren<QComboBox*>()) {
          if (combo->isVisible() &&
              combo->findData(QStringLiteral("glaze")) >= 0) {
            combo->setFocus();
            QTest::keyClick(combo, Qt::Key_Down);
            return;
          }
        }
        Miss(QStringLiteral("no backdrop list"));
      },
      700,
      [] {
        const QString backdrop = FieldText(QStringLiteral("backdrop"));
        return Expect(
            !backdrop.isEmpty() && backdrop != QStringLiteral("glaze"),
            QStringLiteral("the backdrop is still '%1'").arg(backdrop));
      });
  Add(
      QStringLiteral("undo-backdrop"),
      [this, window] {
        Click(window, QStringLiteral("Undo"));
        Click(window, QStringLiteral("Undo"));
      },
      500, back);
  Add(
      QStringLiteral("quality-toggles"),
      [this, window, add_from_list] {
        add_from_list(QStringLiteral("Media"), QStringLiteral("Convert"));
        QTest::qWait(kSettleMs);
        Click(window->inspector_dock()->widget(), QStringLiteral("Max"));
      },
      700,
      [] {
        for (const auto& field : App::Get().session().inspector().fields) {
          if (field.kind == bridge::FieldKind::PresetNumber) {
            return Expect(std::abs(field.number - kMaxQuality) < kNear,
                          QStringLiteral("quality is %1").arg(field.number));
          }
        }
        return QStringLiteral("no quality toggles");
      });
  Add(
      QStringLiteral("undo-quality"),
      [this, window] {
        Click(window, QStringLiteral("Undo"));
        Click(window, QStringLiteral("Undo"));
      },
      500, back);
  const auto select_titled = [this, canvas, view](const QString& title) {
    const bridge::GraphView graph = Graph();
    const auto* card = CardTitled(graph, title);
    if (card == nullptr) {
      Miss(QStringLiteral("no %1 card").arg(title));
      return;
    }
    const QString id = ToQ(card->id);
    Click(canvas, QStringLiteral("Fit to view"));
    ClickAt(view, canvas->ViewPointOf(id));
    QTest::qWait(kSettleMs);
  };
  Add(
      QStringLiteral("object-list-add-env"),
      [this, window, memo, add_from_list] {
        add_from_list(QStringLiteral("Terminal"),
                      QStringLiteral("Open terminal"));
        QTest::qWait(kSettleMs);
        memo->count = static_cast<std::size_t>(
            std::max(0, ListLength(bridge::FieldKind::ObjectList)));
        for (QAbstractButton* button : window->inspector_dock()
                                           ->widget()
                                           ->findChildren<QAbstractButton*>()) {
          if (button->isVisible() &&
              button->text().startsWith(QStringLiteral("Advanced"))) {
            QTest::mouseClick(button, Qt::LeftButton);
          }
        }
        Click(window->inspector_dock()->widget(), QStringLiteral("Add"));
      },
      700,
      [memo] {
        const int length = ListLength(bridge::FieldKind::ObjectList);
        return Expect(length == static_cast<int>(memo->count) + 1,
                      QStringLiteral("the variable list has %1 items, not %2")
                          .arg(length)
                          .arg(memo->count + 1));
      });
  Add(
      QStringLiteral("object-list-remove-env"),
      [this, window] {
        QAbstractButton* last = nullptr;
        for (QAbstractButton* button : window->inspector_dock()
                                           ->widget()
                                           ->findChildren<QAbstractButton*>()) {
          if (button->isVisible() &&
              button->toolTip().startsWith(QStringLiteral("Remove "))) {
            last = button;
          }
        }
        if (last == nullptr) {
          Miss(QStringLiteral("no Remove button on a variable"));
          return;
        }
        QTest::mouseClick(last, Qt::LeftButton);
      },
      700,
      [memo] {
        return Expect(
            ListLength(bridge::FieldKind::ObjectList) ==
                static_cast<int>(memo->count),
            QStringLiteral("removing the cut didn't shorten the list"));
      });
  Add(
      QStringLiteral("undo-open-terminal"),
      [this, window] {
        for (int i = 0; i < 3; ++i) Click(window, QStringLiteral("Undo"));
      },
      500, back);
  Add(
      QStringLiteral("color-list-add"),
      [this, window, memo, select_titled] {
        select_titled(QStringLiteral("Shader mesh gradient"));
        memo->count = static_cast<std::size_t>(
            std::max(0, ListLength(bridge::FieldKind::ColorList)));
        Click(window->inspector_dock()->widget(), QStringLiteral("Add colour"));
      },
      700,
      [memo] {
        return Expect(ListLength(bridge::FieldKind::ColorList) ==
                          static_cast<int>(memo->count) + 1,
                      QStringLiteral("no colour was added"));
      });
  Add(QStringLiteral("undo-color-list"), undo, 500);
  Add(
      QStringLiteral("string-list-add-and-type"),
      [this, window, add_from_list] {
        add_from_list(QStringLiteral("Effects"), QStringLiteral("Redact take"));
        QTest::qWait(kSettleMs);
        Click(window->inspector_dock()->widget(), QStringLiteral("Add"));
        QTest::qWait(kSettleMs);
        VarSurface* last = nullptr;
        for (VarSurface* surface :
             window->inspector_dock()->widget()->findChildren<VarSurface*>()) {
          if (surface->isVisible()) last = surface;
        }
        if (last == nullptr) {
          Miss(QStringLiteral("no new row to type in"));
          return;
        }
        QTest::mouseClick(last->viewport(), Qt::LeftButton);
        QTest::keyClicks(last, QStringLiteral("Cookie banner"));
        QTest::keyClick(last, Qt::Key_Return);
      },
      800,
      [] {
        for (const auto& field : App::Get().session().inspector().fields) {
          if (field.kind == bridge::FieldKind::StringList &&
              ToQ(field.value_json).contains(QStringLiteral("Cookie banner"))) {
            return QString();
          }
        }
        return QStringLiteral("the typed row wasn't saved in the list");
      });
  Add(
      QStringLiteral("undo-string-list"),
      [this, window] {
        for (int i = 0; i < 3; ++i) Click(window, QStringLiteral("Undo"));
      },
      500, back);
  Add(
      QStringLiteral("export-video-panel"),
      [add_from_list] {
        add_from_list(QStringLiteral("Export"), QStringLiteral("Export video"));
      },
      900,
      [window] {
        QString problems;
        QWidget* panel = window->inspector_dock()->widget();
        for (QScrollArea* scroll : panel->findChildren<QScrollArea*>()) {
          if (scroll->widget() != nullptr &&
              scroll->widget()->minimumSizeHint().width() >
                  scroll->viewport()->width()) {
            problems += QStringLiteral(" the panel is wider than the dock");
          }
        }
        bool toggles = false;
        for (const auto& field : App::Get().session().inspector().fields) {
          if (ToQ(field.label) == QStringLiteral("Shapes")) {
            toggles = field.kind == bridge::FieldKind::EnumList;
          }
        }
        if (!toggles) problems += QStringLiteral(" Shapes isn't toggles");
        for (VarSurface* surface : panel->findChildren<VarSurface*>()) {
          if (surface->isVisible() && surface->MissingChips() > 0) {
            problems += QStringLiteral(" a built-in variable shows as missing");
          }
        }
        if (VisibleText(panel, QStringLiteral("Something needs attention")) !=
            nullptr) {
          problems += QStringLiteral(" the problems still have a title");
        }
        return problems.trimmed();
      });
  Add(QStringLiteral("undo-export-video"), undo, 500, back);
  const auto colour_value = [] {
    for (const auto& field : App::Get().session().inspector().fields) {
      if (field.kind == bridge::FieldKind::Color &&
          ToQ(field.label) == QStringLiteral("Colour")) {
        return ToQ(field.text);
      }
    }
    return QStringLiteral("<no colour field>");
  };
  Add(
      QStringLiteral("color-field-opens-picker"),
      [this, window, add_from_list, memo, colour_value] {
        add_from_list(QStringLiteral("Text & overlays"),
                      QStringLiteral("Tooltip"));
        QTest::qWait(kSettleMs);
        memo->other = colour_value();
        Click(window->inspector_dock()->widget(),
              QStringLiteral("Pick colour"));
      },
      700,
      [] {
        auto* picker =
            qobject_cast<ColorPicker*>(QApplication::activePopupWidget());
        return Expect(picker != nullptr && picker->isVisible(),
                      QStringLiteral("no colour picker opened"));
      });
  Add(
      QStringLiteral("color-picker-hue-and-oklch"),
      [this] {
        auto* picker =
            qobject_cast<ColorPicker*>(QApplication::activePopupWidget());
        if (picker == nullptr) {
          Miss(QStringLiteral("the picker closed"));
          return;
        }
        QWidget* area = nullptr;
        QWidget* hue = nullptr;
        for (QWidget* child : picker->findChildren<QWidget*>()) {
          if (child->accessibleName() == QStringLiteral("Hue")) hue = child;
          if (child->accessibleName() ==
              QStringLiteral("Saturation and brightness")) {
            area = child;
          }
        }
        if (area == nullptr || hue == nullptr) {
          Miss(QStringLiteral("the picker has no area or hue slider"));
          return;
        }
        QTest::mouseClick(area, Qt::LeftButton, {},
                          QPoint(area->width() * 3 / 4, area->height() / 4));
        QTest::mouseClick(hue, Qt::LeftButton, {},
                          QPoint(hue->width() / 3, hue->height() / 2));
        Click(picker, QStringLiteral("OKLCH"));
      },
      700,
      [colour_value] {
        const QString value = colour_value();
        return Expect(value.startsWith(QStringLiteral("oklch(")),
                      QStringLiteral("the colour is %1").arg(value));
      });
  Add(
      QStringLiteral("close-color-picker"),
      [] {
        if (QWidget* popup = QApplication::activePopupWidget()) {
          QTest::keyClick(popup, Qt::Key_Escape);
        }
      },
      600,
      [window, colour_value] {
        QString shown;
        for (QLineEdit* line :
             window->inspector_dock()->widget()->findChildren<QLineEdit*>()) {
          if (line->isVisible() &&
              line->accessibleName() == QStringLiteral("Colour")) {
            shown = line->text();
          }
        }
        return Expect(!ColorPicker::IsOpen() && shown == colour_value(),
                      QStringLiteral("the field shows %1").arg(shown));
      });
  Add(QStringLiteral("undo-color-pick"), undo, 600, [memo, colour_value] {
    return Expect(colour_value() == memo->other,
                  QStringLiteral("one undo left %1").arg(colour_value()));
  });
  Add(QStringLiteral("undo-add-tooltip"), undo, 500, back);
  const auto gradient_field = [window]() -> GradientField* {
    for (auto* field :
         window->inspector_dock()->widget()->findChildren<GradientField*>()) {
      if (field->isVisible()) return field;
    }
    return nullptr;
  };
  const auto stops_in = [](const QString& css) {
    return static_cast<int>(css.count(QLatin1Char('%')));
  };
  const auto pick_kind = [window](const QString& kind) {
    for (QComboBox* combo :
         window->inspector_dock()->widget()->findChildren<QComboBox*>()) {
      if (combo->isVisible() && combo->findData(kind) >= 0) {
        if (combo->currentData().toString() == kind) return true;
        combo->setFocus();
        QTest::keyClick(combo, Qt::Key_Down);
        return false;
      }
    }
    return true;
  };
  Add(
      QStringLiteral("gradient-field"),
      [add_from_list, pick_kind] {
        add_from_list(QStringLiteral("Camera"), QStringLiteral("Backdrop"));
        QTest::qWait(kSettleMs);
        for (int i = 0; i < 3 && !pick_kind(QStringLiteral("gradient")); ++i) {
          QTest::qWait(kSettleMs);
        }
      },
      700,
      [gradient_field] {
        GradientField* field = gradient_field();
        return Expect(field != nullptr && field->StopCount() == 2,
                      QStringLiteral("no gradient with two stops"));
      });
  Add(
      QStringLiteral("gradient-add-stop"),
      [this, gradient_field] {
        GradientField* field = gradient_field();
        if (field == nullptr) {
          Miss(QStringLiteral("no gradient field"));
          return;
        }
        StopBar* bar = field->bar();
        const QRectF track = bar->Track();
        QTest::mouseClick(bar, Qt::LeftButton, {},
                          QPoint(static_cast<int>(track.center().x()),
                                 static_cast<int>(track.center().y())));
      },
      600,
      [stops_in] {
        const QString css = FieldText(QStringLiteral("gradient"));
        return Expect(stops_in(css) == 3,
                      QStringLiteral("the gradient is %1").arg(css));
      });
  Add(
      QStringLiteral("gradient-drag-stop"),
      [this, gradient_field, memo] {
        memo->other = FieldText(QStringLiteral("gradient"));
        GradientField* field = gradient_field();
        if (field == nullptr) {
          Miss(QStringLiteral("no gradient field"));
          return;
        }
        StopBar* bar = field->bar();
        const QPoint from = bar->HandleOf(0).toPoint();
        const QRectF track = bar->Track();
        const QPoint to(static_cast<int>(track.left() + track.width() * 0.3),
                        from.y());
        Drag(bar, from, to, 6);
      },
      700,
      [memo] {
        const QString css = FieldText(QStringLiteral("gradient"));
        return Expect(
            css != memo->other && css.contains(QStringLiteral(" 30%")),
            QStringLiteral("the gradient is %1").arg(css));
      });
  Add(QStringLiteral("undo-gradient-drag"), undo, 600, [memo] {
    const QString css = FieldText(QStringLiteral("gradient"));
    return Expect(css == memo->other,
                  QStringLiteral("one undo left %1").arg(css));
  });
  Add(
      QStringLiteral("undo-gradient"),
      [this, window] {
        for (int i = 0; i < 4; ++i) Click(window, QStringLiteral("Undo"));
      },
      600, back);
  Add(
      QStringLiteral("asset-choose-opens-dialog"),
      [this, window, add_from_list, pick_kind] {
        add_from_list(QStringLiteral("Camera"), QStringLiteral("Backdrop"));
        QTest::qWait(kSettleMs);
        for (int i = 0; i < 4 && !pick_kind(QStringLiteral("asset")); ++i) {
          QTest::qWait(kSettleMs);
        }
        ClickLater(window->inspector_dock()->widget(),
                   QStringLiteral("Choose"));
      },
      900,
      [] {
        return Expect(qobject_cast<QFileDialog*>(
                          QApplication::activeModalWidget()) != nullptr,
                      QStringLiteral("no file dialog opened"));
      });
  Add(
      QStringLiteral("asset-pick-file"),
      [] {
        auto* dialog =
            qobject_cast<QFileDialog*>(QApplication::activeModalWidget());
        if (dialog == nullptr) return;
        const QString file = QDir(qEnvironmentVariable("OPSIS_USER_DATA"))
                                 .filePath(QStringLiteral("backdrop.png"));
        QImage picture(kDropSide, kDropSide, QImage::Format_RGB32);
        picture.fill(Qt::darkCyan);
        picture.save(file);
        auto* name =
            dialog->findChild<QLineEdit*>(QStringLiteral("fileNameEdit"));
        if (name == nullptr) {
          dialog->reject();
          return;
        }
        name->setFocus();
        QTest::keyClicks(name, file);
        QTest::qWait(kSettleMs);
        auto* buttons = dialog->findChild<QDialogButtonBox*>();
        QPushButton* open = buttons != nullptr
                                ? buttons->button(QDialogButtonBox::Open)
                                : nullptr;
        if (open == nullptr) {
          QTest::keyClick(name, Qt::Key_Return);
          return;
        }
        QTest::mouseClick(open, Qt::LeftButton);
      },
      1200,
      [] {
        if (QApplication::activeModalWidget() != nullptr) {
          QApplication::activeModalWidget()->close();
          return QStringLiteral("the file dialog is still open");
        }
        const QString asset = FieldText(QStringLiteral("asset"));
        return Expect(asset.contains(QStringLiteral("backdrop")) &&
                          !asset.startsWith(QLatin1Char('/')),
                      QStringLiteral("the image is '%1'").arg(asset));
      });
  Add(
      QStringLiteral("undo-asset"),
      [this, window] {
        for (int i = 0; i < 5; ++i) Click(window, QStringLiteral("Undo"));
      },
      600, back);
  const auto field_of = [](bridge::FieldKind kind) {
    for (const auto& field : App::Get().session().inspector().fields) {
      if (field.kind == kind) return field;
    }
    return bridge::FieldView{};
  };
  Add(
      QStringLiteral("offsets-field"),
      [this, window, select_titled] {
        select_titled(QStringLiteral("Layers"));
        QTest::qWait(kSettleMs);
        for (QLineEdit* line :
             window->inspector_dock()->widget()->findChildren<QLineEdit*>()) {
          if (line->isVisible() &&
              line->accessibleName() == QStringLiteral("Layer 2")) {
            QTest::mouseClick(line, Qt::LeftButton);
            QTest::keyClick(line, Qt::Key_A, Qt::ControlModifier);
            QTest::keyClicks(line, QStringLiteral("1.5"));
            QTest::keyClick(line, Qt::Key_Return);
            return;
          }
        }
        Miss(QStringLiteral("no Layer 2 start"));
      },
      700,
      [field_of] {
        const QString value = ToQ(field_of(bridge::FieldKind::Offsets).text);
        return Expect(value == QStringLiteral("0,1500"),
                      QStringLiteral("the starts are '%1'").arg(value));
      });
  Add(QStringLiteral("undo-offsets"), undo, 500, [field_of] {
    const QString value = ToQ(field_of(bridge::FieldKind::Offsets).text);
    return Expect(value.isEmpty(),
                  QStringLiteral("the starts are still '%1'").arg(value));
  });
  Add(
      QStringLiteral("spot-drag"),
      [this, window, add_from_list] {
        add_from_list(QStringLiteral("Camera"), QStringLiteral("Zoom here"));
        QTest::qWait(kSettleMs);
        auto* frame =
            window->inspector_dock()->widget()->findChild<SpotFrame*>();
        if (frame == nullptr) {
          Miss(QStringLiteral("no spot frame"));
          return;
        }
        window->inspector_dock()
            ->widget()
            ->findChild<QScrollArea*>()
            ->ensureWidgetVisible(frame);
        QTest::qWait(kSettleMs);
        const QPoint from(frame->width() / 2, frame->height() / 2);
        const QPoint to(frame->width() / 4, frame->height() * 3 / 4);
        Drag(frame, from, to, 6);
      },
      800,
      [field_of] {
        const bridge::FieldView spot = field_of(bridge::FieldKind::Spot);
        const double y = QJsonDocument::fromJson(ToQ(spot.extra_json).toUtf8())
                             .object()
                             .value(QStringLiteral("y"))
                             .toDouble();
        return Expect(
            std::abs(spot.number - 0.25) < 0.02 && std::abs(y - 0.75) < 0.02,
            QStringLiteral("the spot is %1, %2").arg(spot.number).arg(y));
      });
  Add(QStringLiteral("undo-spot-drag"), undo, 600, [field_of] {
    const bridge::FieldView spot = field_of(bridge::FieldKind::Spot);
    return Expect(
        spot.number == 0.5,
        QStringLiteral("one undo left the spot at %1").arg(spot.number));
  });
  Add(QStringLiteral("undo-add-zoom"), undo, 500, back);
  const auto boxes_frame = [window]() -> BoxesFrame* {
    auto* frame = window->inspector_dock()->widget()->findChild<BoxesFrame*>();
    if (frame != nullptr) {
      window->inspector_dock()
          ->widget()
          ->findChild<QScrollArea*>()
          ->ensureWidgetVisible(frame);
    }
    return frame;
  };
  const auto area_count = [] {
    for (const auto& field : App::Get().session().inspector().fields) {
      if (field.kind == bridge::FieldKind::Boxes) {
        return static_cast<int>(
            QJsonDocument::fromJson(ToQ(field.value_json).toUtf8())
                .array()
                .size());
      }
    }
    return -1;
  };
  const auto at = [](QWidget* frame, double x, double y) {
    return QPoint(static_cast<int>(frame->width() * x),
                  static_cast<int>(frame->height() * y));
  };
  Add(
      QStringLiteral("boxes-draw"),
      [this, add_from_list, boxes_frame, at] {
        add_from_list(QStringLiteral("Effects"), QStringLiteral("Redact take"));
        QTest::qWait(kSettleMs);
        BoxesFrame* frame = boxes_frame();
        if (frame == nullptr) {
          Miss(QStringLiteral("no areas frame"));
          return;
        }
        QTest::qWait(kSettleMs);
        Drag(frame, at(frame, 0.05, 0.05), at(frame, 0.3, 0.3), 6);
      },
      700,
      [area_count] {
        return Expect(area_count() == 2,
                      QStringLiteral("%1 areas, not 2").arg(area_count()));
      });
  Add(
      QStringLiteral("boxes-move"),
      [this, boxes_frame, at] {
        BoxesFrame* frame = boxes_frame();
        if (frame == nullptr) {
          Miss(QStringLiteral("no areas frame"));
          return;
        }
        Drag(frame, at(frame, 0.17, 0.17), at(frame, 0.77, 0.17), 6);
      },
      700,
      [boxes_frame] {
        BoxesFrame* frame = boxes_frame();
        if (frame == nullptr || frame->areas().size() != 2) {
          return QStringLiteral("the areas changed count");
        }
        const Area& moved = frame->areas().at(1);
        return Expect(moved.x > 0.6 && moved.x < 0.7 && moved.y < 0.1,
                      QStringLiteral("the area sits at %1, %2")
                          .arg(moved.x)
                          .arg(moved.y));
      });
  Add(
      QStringLiteral("boxes-remove"),
      [boxes_frame] {
        BoxesFrame* frame = boxes_frame();
        if (frame == nullptr || frame->areas().size() < 2) return;
        const QRectF box = frame->ToWidget(frame->areas().at(1));
        QTest::mouseClick(
            frame, Qt::LeftButton, {},
            QPoint(static_cast<int>(box.right()), static_cast<int>(box.top())));
      },
      700,
      [area_count] {
        return Expect(
            area_count() == 1,
            QStringLiteral("%1 areas after removing").arg(area_count()));
      });
  Add(
      QStringLiteral("undo-boxes"),
      [this, window] {
        for (int i = 0; i < 4; ++i) Click(window, QStringLiteral("Undo"));
      },
      600, back);
  const auto marks_frame = [window]() -> MarksFrame* {
    auto* frame = window->inspector_dock()->widget()->findChild<MarksFrame*>();
    if (frame != nullptr) {
      window->inspector_dock()
          ->widget()
          ->findChild<QScrollArea*>()
          ->ensureWidgetVisible(frame);
    }
    return frame;
  };
  const auto saved_marks = [] {
    for (const auto& field : App::Get().session().inspector().fields) {
      if (field.kind == bridge::FieldKind::Marks) {
        return QJsonDocument::fromJson(ToQ(field.value_json).toUtf8()).array();
      }
    }
    return QJsonArray();
  };
  Add(
      QStringLiteral("marks-draw-arrow"),
      [this, add_from_list, marks_frame, at] {
        add_from_list(QStringLiteral("Image"), QStringLiteral("Annotate"));
        QTest::qWait(kSettleMs);
        MarksFrame* frame = marks_frame();
        if (frame == nullptr) {
          Miss(QStringLiteral("no marks frame"));
          return;
        }
        QTest::qWait(kSettleMs);
        Drag(frame, at(frame, 0.2, 0.2), at(frame, 0.6, 0.5), 6);
      },
      700,
      [this, window, saved_marks] {
        const QJsonArray marks = saved_marks();
        QString problems;
        if (marks.size() != 1 ||
            marks.at(0).toObject().value(QStringLiteral("kind")).toString() !=
                QStringLiteral("arrow")) {
          problems += QStringLiteral(" no arrow was drawn");
        }
        if (Find(window->inspector_dock()->widget(),
                 QStringLiteral("Remove")) == nullptr) {
          problems += QStringLiteral(" the new mark's settings aren't shown");
        }
        return problems.trimmed();
      });
  Add(
      QStringLiteral("marks-step-click"),
      [this, window, marks_frame, at] {
        Click(window->inspector_dock()->widget(),
              QStringLiteral("Step number"));
        MarksFrame* frame = marks_frame();
        if (frame == nullptr) return;
        QTest::mouseClick(frame, Qt::LeftButton, {}, at(frame, 0.8, 0.8));
      },
      700,
      [saved_marks] {
        const QJsonArray marks = saved_marks();
        return Expect(
            marks.size() == 2 && marks.at(1)
                                         .toObject()
                                         .value(QStringLiteral("kind"))
                                         .toString() == QStringLiteral("step"),
            QStringLiteral("%1 marks after placing a step").arg(marks.size()));
      });
  Add(
      QStringLiteral("marks-move-arrow"),
      [this, window, marks_frame, at] {
        Click(window->inspector_dock()->widget(),
              QStringLiteral("Select and move"));
        MarksFrame* frame = marks_frame();
        if (frame == nullptr) return;
        Drag(frame, at(frame, 0.4, 0.35), at(frame, 0.5, 0.45), 6);
      },
      700,
      [saved_marks] {
        const QJsonObject arrow = saved_marks().at(0).toObject();
        const double x = arrow.value(QStringLiteral("x")).toDouble();
        return Expect(std::abs(x - 0.3) < 0.02,
                      QStringLiteral("the arrow starts at %1").arg(x));
      });
  Add(
      QStringLiteral("marks-delete-key"),
      [marks_frame] {
        MarksFrame* frame = marks_frame();
        if (frame == nullptr) return;
        frame->setFocus();
        QTest::keyClick(frame, Qt::Key_Delete);
      },
      700,
      [saved_marks] {
        const QJsonArray marks = saved_marks();
        return Expect(
            marks.size() == 1 && marks.at(0)
                                         .toObject()
                                         .value(QStringLiteral("kind"))
                                         .toString() == QStringLiteral("step"),
            QStringLiteral("%1 marks after Delete").arg(marks.size()));
      });
  Add(
      QStringLiteral("undo-marks"),
      [this, window] {
        for (int i = 0; i < 5; ++i) Click(window, QStringLiteral("Undo"));
      },
      600, back);
  const auto opacity_keys = [] {
    for (const auto& field : App::Get().session().inspector().fields) {
      if (field.kind == bridge::FieldKind::Envelope) {
        return QJsonDocument::fromJson(ToQ(field.value_json).toUtf8())
            .object()
            .value(QStringLiteral("opacity"))
            .toArray();
      }
    }
    return QJsonArray();
  };
  const auto inspector_line = [window](const QString& name) -> QLineEdit* {
    for (QLineEdit* line :
         window->inspector_dock()->widget()->findChildren<QLineEdit*>()) {
      if (line->isVisible() && line->accessibleName() == name) return line;
    }
    return nullptr;
  };
  Add(
      QStringLiteral("envelope-add-key"),
      [this, window, select_titled] {
        select_titled(QStringLiteral("Title card"));
        QTest::qWait(kSettleMs);
        QWidget* panel = window->inspector_dock()->widget();
        for (QAbstractButton* button :
             panel->findChildren<QAbstractButton*>()) {
          if (button->isVisible() &&
              button->text().startsWith(QStringLiteral("Advanced"))) {
            QTest::mouseClick(button, Qt::LeftButton);
          }
        }
        QTest::qWait(kSettleMs);
        for (QAbstractButton* button :
             panel->findChildren<QAbstractButton*>()) {
          if (button->isVisible() &&
              button->accessibleName() ==
                  QStringLiteral("Add opacity keyframe")) {
            panel->findChild<QScrollArea*>()->ensureWidgetVisible(button);
            QTest::mouseClick(button, Qt::LeftButton);
            return;
          }
        }
        Miss(QStringLiteral("no Add for the opacity track"));
      },
      700,
      [opacity_keys] {
        return Expect(
            opacity_keys().size() == 1,
            QStringLiteral("%1 opacity keys").arg(opacity_keys().size()));
      });
  Add(
      QStringLiteral("envelope-type-value"),
      [this, inspector_line] {
        QLineEdit* value = inspector_line(QStringLiteral("Value"));
        if (value == nullptr) {
          Miss(QStringLiteral("no keyframe value box"));
          return;
        }
        QTest::mouseClick(value, Qt::LeftButton);
        QTest::keyClick(value, Qt::Key_A, Qt::ControlModifier);
        QTest::keyClicks(value, QStringLiteral("50"));
        QTest::keyClick(value, Qt::Key_Return);
      },
      700,
      [opacity_keys] {
        const double value = opacity_keys()
                                 .at(0)
                                 .toObject()
                                 .value(QStringLiteral("value"))
                                 .toDouble(-1);
        return Expect(std::abs(value - 0.5) < 1e-9,
                      QStringLiteral("the key holds %1").arg(value));
      });
  Add(
      QStringLiteral("envelope-remove-key"),
      [this, window] {
        Click(window->inspector_dock()->widget(),
              QStringLiteral("Remove keyframe"));
      },
      700,
      [opacity_keys] {
        return Expect(
            opacity_keys().isEmpty(),
            QStringLiteral("%1 opacity keys left").arg(opacity_keys().size()));
      });
  Add(
      QStringLiteral("undo-envelope"),
      [this, window] {
        for (int i = 0; i < 3; ++i) Click(window, QStringLiteral("Undo"));
      },
      600,
      [opacity_keys] {
        return Expect(opacity_keys().isEmpty(),
                      QStringLiteral("%1 opacity keys after undoing")
                          .arg(opacity_keys().size()));
      });
  Add(
      QStringLiteral("camera-field-needs-the-film"),
      [add_from_list, pick_kind] {
        add_from_list(QStringLiteral("Text & overlays"),
                      QStringLiteral("3D text"));
        QTest::qWait(kSettleMs);
        for (int i = 0; i < 12 && !pick_kind(QStringLiteral("keyframes"));
             ++i) {
          QTest::qWait(kSettleMs);
        }
      },
      700,
      [this, window] {
        QAbstractButton* add = Find(window->inspector_dock()->widget(),
                                    QStringLiteral("Add key at playhead"));
        return Expect(
            add != nullptr && !add->isEnabled() &&
                add->toolTip().startsWith(QStringLiteral("Put this node")),
            QStringLiteral("no disabled Add key at playhead"));
      });
  Add(
      QStringLiteral("undo-camera"),
      [this, window, memo] {
        for (int i = 0; i < 14 && Graph().nodes.size() != memo->nodes; ++i) {
          Click(window, QStringLiteral("Undo"));
        }
      },
      600, back);
  Add(
      QStringLiteral("theme-mode"),
      [this, window, add_from_list] {
        add_from_list(QStringLiteral("Camera"), QStringLiteral("Device frame"));
        QTest::qWait(kSettleMs);
        Click(window->inspector_dock()->widget(), QStringLiteral("Light"));
      },
      700,
      [] {
        return Expect(
            FieldText(QStringLiteral("theme")) == QStringLiteral("light"),
            QStringLiteral("the theme is %1")
                .arg(FieldText(QStringLiteral("theme"))));
      });
  Add(
      QStringLiteral("theme-colours-open"),
      [this, window] {
        Click(window->inspector_dock()->widget(), QStringLiteral("Colours"));
      },
      500,
      [window] {
        int shown = 0;
        for (auto* input :
             window->inspector_dock()->widget()->findChildren<ColorInput*>()) {
          if (input->isVisible()) ++shown;
        }
        return Expect(shown > 1, QStringLiteral("%1 theme colours").arg(shown));
      });
  Add(
      QStringLiteral("theme-colour-typed"),
      [this, window] {
        for (auto* input :
             window->inspector_dock()->widget()->findChildren<ColorInput*>()) {
          if (!input->isVisible()) continue;
          auto* line = input->findChild<QLineEdit*>();
          QTest::mouseClick(line, Qt::LeftButton);
          QTest::keyClicks(line, QStringLiteral("#ff0000"));
          QTest::keyClick(line, Qt::Key_Return);
          return;
        }
        Miss(QStringLiteral("no theme colour to type in"));
      },
      700,
      [this, window] {
        QString problems;
        for (const auto& field : App::Get().session().inspector().fields) {
          if (ToQ(field.key) == QStringLiteral("theme") &&
              !ToQ(field.value_json).contains(QStringLiteral("#ff0000"))) {
            problems += QStringLiteral(" the colour wasn't saved");
          }
        }
        if (Find(window->inspector_dock()->widget(),
                 QStringLiteral("Colours (1 changed)")) == nullptr) {
          problems += QStringLiteral(" no count of changed colours");
        }
        return problems.trimmed();
      });
  Add(
      QStringLiteral("undo-theme"),
      [this, window] {
        for (int i = 0; i < 3; ++i) Click(window, QStringLiteral("Undo"));
      },
      600, back);
  const auto timeline = [window]() -> TimelineCanvas* {
    auto* panel =
        qobject_cast<TimelinePanel*>(window->timeline_dock()->widget());
    return panel != nullptr ? panel->canvas() : nullptr;
  };
  const auto bar_spans = [timeline] {
    std::vector<std::pair<double, double>> spans;
    if (TimelineCanvas* canvas = timeline();
        canvas != nullptr && canvas->view()) {
      for (const auto& bar : canvas->view()->bars)
        spans.emplace_back(bar.from, bar.to);
    }
    return spans;
  };
  Add(
      QStringLiteral("timeline-shows-the-film"),
      [window, select_titled] {
        select_titled(QStringLiteral("Export video"));
        QTest::qWait(kSettleMs);
        window->timeline_dock()->setAsCurrentTab();
      },
      900,
      [timeline] {
        TimelineCanvas* canvas = timeline();
        return Expect(canvas != nullptr && canvas->has_film() &&
                          !canvas->view()->bars.empty(),
                      QStringLiteral("the timeline shows no bars"));
      });
  Add(
      QStringLiteral("timeline-scrub-ruler"),
      [timeline] {
        TimelineCanvas* canvas = timeline();
        if (canvas == nullptr) return;
        QTest::mouseClick(canvas->viewport(), Qt::LeftButton, {},
                          QPoint(static_cast<int>(canvas->FrameX(60)), 8));
      },
      400,
      [] {
        const double frame = PlaybackClock::Instance()->frame();
        return Expect(std::abs(frame - 60) <= 1,
                      QStringLiteral("the playhead is at %1").arg(frame));
      });
  Add(
      QStringLiteral("timeline-play"),
      [this, window, memo] {
        memo->x = PlaybackClock::Instance()->frame();
        Click(window->timeline_dock()->widget(), QStringLiteral("Play  Space"));
      },
      600,
      [memo] {
        const PlaybackClock* clock = PlaybackClock::Instance();
        return Expect(clock->playing() && clock->frame() > memo->x,
                      QStringLiteral("the playhead didn't move"));
      });
  Add(
      QStringLiteral("timeline-pause-with-space"),
      [timeline] {
        TimelineCanvas* canvas = timeline();
        if (canvas == nullptr) return;
        canvas->setFocus();
        QTest::keyClick(canvas, Qt::Key_Space);
      },
      300,
      [] {
        return Expect(!PlaybackClock::Instance()->playing(),
                      QStringLiteral("playback didn't stop"));
      });
  Add(
      QStringLiteral("timeline-select-bar"),
      [timeline, memo] {
        TimelineCanvas* canvas = timeline();
        if (canvas == nullptr || canvas->view()->bars.empty()) return;
        const auto& bar = canvas->view()->bars.at(0);
        memo->id = ToQ(bar.node_id);
        const QRectF box = canvas->BarRect(ToQ(bar.key));
        QTest::mouseClick(canvas->viewport(), Qt::LeftButton, {},
                          box.center().toPoint());
      },
      600,
      [memo] {
        return Expect(ToQ(App::Get().session().selected()) == memo->id,
                      QStringLiteral("the bar's node isn't selected"));
      });
  Add(
      QStringLiteral("timeline-trim-end"),
      [this, timeline, bar_spans, memo] {
        TimelineCanvas* canvas = timeline();
        if (canvas == nullptr || canvas->view()->bars.empty()) return;
        const auto& bar = canvas->view()->bars.at(0);
        memo->key = ToQ(bar.key);
        memo->width = bar.to;
        const QRectF box = canvas->BarRect(memo->key);
        const QPoint edge(static_cast<int>(box.right()) - 2,
                          static_cast<int>(box.center().y()));
        Drag(canvas->viewport(), edge, edge - QPoint(30, 0), 6);
        Q_UNUSED(bar_spans);
      },
      800,
      [timeline, memo] {
        TimelineCanvas* canvas = timeline();
        if (canvas == nullptr) return QStringLiteral("no timeline");
        for (const auto& bar : canvas->view()->bars) {
          if (ToQ(bar.key) == memo->key) {
            return Expect(
                bar.to < memo->width,
                QStringLiteral("the bar still ends at %1").arg(bar.to));
          }
        }
        return QStringLiteral("the trimmed bar is gone");
      });
  Add(QStringLiteral("undo-trim"), undo, 600, [timeline, memo] {
    TimelineCanvas* canvas = timeline();
    if (canvas == nullptr) return QStringLiteral("no timeline");
    for (const auto& bar : canvas->view()->bars) {
      if (ToQ(bar.key) == memo->key) {
        return Expect(
            std::abs(bar.to - memo->width) < 0.5,
            QStringLiteral("one undo left the end at %1").arg(bar.to));
      }
    }
    return QStringLiteral("the bar is gone after undo");
  });
  Add(
      QStringLiteral("timeline-split-tool"),
      [timeline, bar_spans, memo] {
        TimelineCanvas* canvas = timeline();
        if (canvas == nullptr || canvas->view()->bars.empty()) return;
        memo->count = bar_spans().size();
        canvas->setFocus();
        QTest::keyClick(canvas, Qt::Key_C);
        const QRectF box = canvas->BarRect(ToQ(canvas->view()->bars.at(0).key));
        QTest::mouseClick(canvas->viewport(), Qt::LeftButton, {},
                          box.center().toPoint());
      },
      800,
      [timeline, bar_spans, memo] {
        TimelineCanvas* canvas = timeline();
        QString problems;
        if (canvas == nullptr || canvas->tool() != QStringLiteral("blade")) {
          problems += QStringLiteral(" C didn't pick the Split tool");
        }
        if (bar_spans().size() != memo->count + 1) {
          problems += QStringLiteral(" %1 bars after splitting")
                          .arg(bar_spans().size());
        }
        return problems.trimmed();
      });
  Add(
      QStringLiteral("undo-split"),
      [timeline, undo] {
        undo();
        if (TimelineCanvas* canvas = timeline()) {
          canvas->setFocus();
          QTest::keyClick(canvas, Qt::Key_V);
        }
      },
      600,
      [bar_spans, memo] {
        return Expect(
            bar_spans().size() == memo->count,
            QStringLiteral("%1 bars after undo").arg(bar_spans().size()));
      });
  Add(
      QStringLiteral("timeline-marker-add-and-rename"),
      [timeline] {
        TimelineCanvas* canvas = timeline();
        if (canvas == nullptr) return;
        canvas->setFocus();
        QTest::keyClick(canvas, Qt::Key_M);
        QTest::qWait(kSettleMs);
        if (canvas->view()->markers.empty()) return;
        const QString id = ToQ(canvas->view()->markers.at(0).id);
        QTest::mouseDClick(canvas->viewport(), Qt::LeftButton, {},
                           canvas->MarkerRect(id).center().toPoint());
        QTest::qWait(kSettleMs);
        auto* line = canvas->viewport()->findChild<QLineEdit*>(
            QStringLiteral("MarkerName"));
        if (line == nullptr) return;
        QTest::keyClick(line, Qt::Key_A, Qt::ControlModifier);
        QTest::keyClicks(line, QStringLiteral("Intro"));
        QTest::keyClick(line, Qt::Key_Return);
      },
      700,
      [timeline] {
        TimelineCanvas* canvas = timeline();
        return Expect(canvas != nullptr &&
                          canvas->view()->markers.size() == 1 &&
                          ToQ(canvas->view()->markers.at(0).label) ==
                              QStringLiteral("Intro"),
                      QStringLiteral("no marker named Intro"));
      });
  Add(
      QStringLiteral("timeline-marker-delete"),
      [timeline] {
        TimelineCanvas* canvas = timeline();
        if (canvas == nullptr || canvas->view()->markers.empty()) return;
        const QString id = ToQ(canvas->view()->markers.at(0).id);
        QTest::mouseClick(canvas->viewport(), Qt::LeftButton, {},
                          canvas->MarkerRect(id).center().toPoint());
        canvas->setFocus();
        QTest::keyClick(canvas, Qt::Key_Delete);
      },
      600,
      [timeline] {
        TimelineCanvas* canvas = timeline();
        return Expect(canvas != nullptr && canvas->view()->markers.empty(),
                      QStringLiteral("the marker is still there"));
      });
  Add(
      QStringLiteral("undo-markers"),
      [this, window] {
        for (int i = 0; i < 3; ++i) Click(window, QStringLiteral("Undo"));
      },
      600,
      [timeline] {
        TimelineCanvas* canvas = timeline();
        return Expect(canvas != nullptr && canvas->view()->markers.empty(),
                      QStringLiteral("markers left after undo"));
      });
  const auto bar_titled = [timeline](const QString& title) -> QString {
    TimelineCanvas* canvas = timeline();
    if (canvas == nullptr || canvas->view() == nullptr) return {};
    for (const auto& bar : canvas->view()->bars) {
      if (ToQ(bar.title) == title) return ToQ(bar.key);
    }
    return {};
  };
  const auto opacity_of = [] {
    for (const auto& field : App::Get().session().inspector().fields) {
      if (field.kind == bridge::FieldKind::Envelope) {
        return QJsonDocument::fromJson(ToQ(field.value_json).toUtf8())
            .object()
            .value(QStringLiteral("opacity"))
            .toArray();
      }
    }
    return QJsonArray();
  };
  Add(
      QStringLiteral("timeline-add-clip-key"),
      [window, timeline, bar_titled, memo] {
        TimelineCanvas* canvas = timeline();
        if (canvas == nullptr) return;
        for (QComboBox* combo :
             window->timeline_dock()->widget()->findChildren<QComboBox*>()) {
          if (combo->accessibleName() == QStringLiteral("Clip keyframes")) {
            combo->setFocus();
            QTest::keyClick(combo, Qt::Key_Down);
          }
        }
        memo->key = bar_titled(QStringLiteral("Title card"));
        const QRectF box = canvas->BarRect(memo->key);
        const QPoint at(static_cast<int>(box.center().x()),
                        static_cast<int>(box.top() + 12));
        QTest::mouseClick(canvas->viewport(), Qt::LeftButton, {}, at);
        QTest::mouseDClick(canvas->viewport(), Qt::LeftButton, {}, at);
      },
      700,
      [timeline, opacity_of] {
        TimelineCanvas* canvas = timeline();
        QString problems;
        if (canvas == nullptr ||
            canvas->video_track() != QStringLiteral("opacity")) {
          problems += QStringLiteral(" the clip keyframes aren't Opacity");
        }
        if (opacity_of().size() != 1) {
          problems +=
              QStringLiteral(" %1 opacity keys").arg(opacity_of().size());
        }
        return problems.trimmed();
      });
  Add(
      QStringLiteral("timeline-drag-clip-key"),
      [this, timeline, memo, opacity_of] {
        TimelineCanvas* canvas = timeline();
        if (canvas == nullptr) return;
        memo->x = opacity_of()
                      .at(0)
                      .toObject()
                      .value(QStringLiteral("value"))
                      .toDouble();
        const QPoint at = canvas->KeyPoint(memo->key, 0).toPoint();
        Drag(canvas->viewport(), at, at + QPoint(0, 12), 6);
      },
      700,
      [memo, opacity_of] {
        const double value = opacity_of()
                                 .at(0)
                                 .toObject()
                                 .value(QStringLiteral("value"))
                                 .toDouble();
        return Expect(value < memo->x,
                      QStringLiteral("the key stayed at %1").arg(value));
      });
  Add(
      QStringLiteral("timeline-remove-clip-key"),
      [timeline, memo] {
        TimelineCanvas* canvas = timeline();
        if (canvas == nullptr) return;
        const QPoint at = canvas->KeyPoint(memo->key, 0).toPoint();
        QTest::mouseClick(canvas->viewport(), Qt::LeftButton, {}, at);
        QTest::mouseDClick(canvas->viewport(), Qt::LeftButton, {}, at);
      },
      700,
      [opacity_of] {
        return Expect(
            opacity_of().isEmpty(),
            QStringLiteral("%1 opacity keys left").arg(opacity_of().size()));
      });
  Add(
      QStringLiteral("undo-clip-keys"),
      [this, window, timeline] {
        for (int i = 0; i < 3; ++i) Click(window, QStringLiteral("Undo"));
        for (QComboBox* combo :
             window->timeline_dock()->widget()->findChildren<QComboBox*>()) {
          if (combo->accessibleName() == QStringLiteral("Clip keyframes")) {
            combo->setFocus();
            QTest::keyClick(combo, Qt::Key_Up);
          }
        }
        Q_UNUSED(timeline);
      },
      600,
      [timeline, opacity_of] {
        TimelineCanvas* canvas = timeline();
        QString problems;
        if (!opacity_of().isEmpty()) {
          problems += QStringLiteral(" %1 opacity keys after undo")
                          .arg(opacity_of().size());
        }
        if (canvas == nullptr || !canvas->video_track().isEmpty()) {
          problems += QStringLiteral(" the clip keyframes are still shown");
        }
        return problems.trimmed();
      });
  Add(
      QStringLiteral("timeline-zoom-keys"),
      [timeline, memo] {
        TimelineCanvas* canvas = timeline();
        if (canvas == nullptr) return;
        memo->x = canvas->px_per_frame();
        canvas->setFocus();
        QTest::keyClick(canvas, Qt::Key_Equal);
      },
      400,
      [timeline, memo] {
        TimelineCanvas* canvas = timeline();
        return Expect(canvas != nullptr && canvas->px_per_frame() > memo->x,
                      QStringLiteral("= didn't zoom in"));
      });
  Add(
      QStringLiteral("timeline-fit-key"),
      [timeline] {
        TimelineCanvas* canvas = timeline();
        if (canvas == nullptr) return;
        QTest::keyClick(canvas, Qt::Key_Z);
      },
      400,
      [timeline, memo] {
        TimelineCanvas* canvas = timeline();
        return Expect(canvas != nullptr &&
                          std::abs(canvas->px_per_frame() - memo->x) < 1e-9,
                      QStringLiteral("Z didn't fit the film again"));
      });
  Add(
      QStringLiteral("hide-node-list"),
      [this, window] { Click(window, QStringLiteral("Hide node list")); }, 600,
      [window] {
        return Expect(
            !window->nodes_dock()->isOpen() && window->rail()->isVisible(),
            QStringLiteral("the rail didn't replace the node list"));
      });
  Add(
      QStringLiteral("rail-category-popover"),
      [this, window] {
        const auto buttons = window->rail()->findChildren<QToolButton*>();
        if (buttons.size() < 2) {
          Miss(QStringLiteral("the rail has no categories"));
          return;
        }
        QTest::mouseClick(buttons.at(1), Qt::LeftButton);
      },
      600,
      [] {
        QWidget* popup = QApplication::activePopupWidget();
        const bool open = popup != nullptr &&
                          popup->objectName() == QStringLiteral("Popover");
        return Expect(open, QStringLiteral("no category popover"));
      });
  Add(QStringLiteral("close-category-popover"), [] {
    if (QWidget* popup = QApplication::activePopupWidget()) popup->close();
  });
  Add(
      QStringLiteral("show-node-list-from-top-bar"),
      [this, window] { Click(window, QStringLiteral("Show node list")); }, 600,
      [window] {
        QString problems;
        for (QAbstractButton* button :
             window->rail()->findChildren<QAbstractButton*>()) {
          if (button->toolTip().startsWith(QStringLiteral("Show node list"))) {
            problems += QStringLiteral(" the rail repeats the show button");
          }
        }
        if (!window->nodes_dock()->isOpen()) {
          problems += QStringLiteral(" the node list didn't come back");
        }
        return problems.trimmed();
      });
  Add(
      QStringLiteral("hide-the-grid"),
      [this, canvas] { Click(canvas, QStringLiteral("Hide the grid")); }, 400,
      [canvas] {
        return Expect(!canvas->GridShown(),
                      QStringLiteral("the grid is still on"));
      });
  Add(
      QStringLiteral("show-the-grid"),
      [this, canvas] { Click(canvas, QStringLiteral("Show the grid")); }, 400,
      [canvas] {
        return Expect(canvas->GridShown(),
                      QStringLiteral("the grid stayed off"));
      });

  Add(
      QStringLiteral("canvas-touchpad-button"),
      [this, canvas] {
        Click(canvas, QStringLiteral(
                          "Touchpad: two fingers move around, pinch to zoom"));
      },
      500,
      [canvas] {
        QString problems = canvas->ToolbarProblems();
        if (ToQ(App::Get().session().settings().canvas_mode) !=
            QStringLiteral("touchpad")) {
          problems += QStringLiteral(" the setting didn't change");
        }
        return problems.trimmed();
      });
  Add(
      QStringLiteral("canvas-mouse-button"),
      [this, canvas] {
        Click(canvas, QStringLiteral(
                          "Mouse: the wheel zooms, drag the background to move "
                          "around"));
      },
      500,
      [] {
        return Expect(ToQ(App::Get().session().settings().canvas_mode) ==
                          QStringLiteral("mouse"),
                      QStringLiteral("the setting didn't go back to mouse"));
      });
  Add(
      QStringLiteral("zoom-buttons"),
      [this, canvas] {
        Click(canvas, QStringLiteral("Zoom in"));
        Click(canvas, QStringLiteral("Zoom to 100%"));
      },
      400,
      [canvas] {
        return Expect(
            std::abs(canvas->zoom() - 1.0) < 0.01,
            QStringLiteral("zoom is %1 after 100%").arg(canvas->zoom()));
      });

  Add(
      QStringLiteral("drop-a-file"),
      [view, memo] {
        memo->count = Graph().nodes.size();
        const QString file = QDir(qEnvironmentVariable("OPSIS_USER_DATA"))
                                 .filePath(QStringLiteral("drop.png"));
        QImage picture(kDropSide, kDropSide, QImage::Format_RGB32);
        picture.fill(Qt::white);
        picture.save(file);
        auto* data = new QMimeData();
        data->setUrls({QUrl::fromLocalFile(file)});
        const QPointF at(view->rect().center());
        QDragEnterEvent enter(at.toPoint(), Qt::CopyAction, data,
                              Qt::LeftButton, Qt::NoModifier);
        QApplication::sendEvent(view, &enter);
        QDropEvent drop(at, Qt::CopyAction, data, Qt::LeftButton,
                        Qt::NoModifier);
        QApplication::sendEvent(view, &drop);
        delete data;
      },
      1200,
      [memo] {
        return Expect(Graph().nodes.size() == memo->count + 1,
                      QStringLiteral("the dropped file didn't add a node"));
      });
  Add(QStringLiteral("undo-drop"), undo, 500);

  Add(
      QStringLiteral("open-settings-from-top-bar"),
      [this, window] { Click(window, QStringLiteral("Settings")); }, 700,
      [] {
        for (QWidget* top : QApplication::topLevelWidgets()) {
          if (top->isVisible() &&
              top->objectName() == QStringLiteral("SettingsWindow")) {
            return QString();
          }
        }
        return QStringLiteral("Settings didn't open");
      });
  Add(
      QStringLiteral("settings-api-switch"),
      [this] {
        QWidget* settings = nullptr;
        for (QWidget* top : QApplication::topLevelWidgets()) {
          if (top->objectName() == QStringLiteral("SettingsWindow"))
            settings = top;
        }
        if (settings == nullptr) {
          Miss(QStringLiteral("no Settings window"));
          return;
        }
        auto* pages = settings->findChild<QListWidget*>();
        for (int i = 0; i < pages->count(); ++i) {
          if (pages->item(i)->text() == QStringLiteral("Local API")) {
            QTest::mouseClick(pages->viewport(), Qt::LeftButton, {},
                              pages->visualItemRect(pages->item(i)).center());
          }
        }
        for (ToggleSwitch* toggle : settings->findChildren<ToggleSwitch*>()) {
          if (toggle->isVisible()) {
            QTest::mouseClick(toggle, Qt::LeftButton);
            break;
          }
        }
      },
      1500,
      [window] {
        const bool on = App::Get().session().bar().api_enabled;
        QString problems =
            Expect(on, QStringLiteral("the API didn't switch on"));
        if (on && window->findChild<QPushButton*>(QStringLiteral("ApiBadge")) &&
            !window->findChild<QPushButton*>(QStringLiteral("ApiBadge"))
                 ->isVisible()) {
          problems += QStringLiteral(" no API badge");
        }
        return problems;
      });
  Add(QStringLiteral("close-settings"), [this] {
    for (QWidget* top : QApplication::topLevelWidgets()) {
      if (top->isVisible() &&
          top->objectName() == QStringLiteral("SettingsWindow")) {
        Click(top, QStringLiteral("Close"));
      }
    }
  });

  Add(
      QStringLiteral("ask-to-close"),
      [this, window] { ClickLater(window, QStringLiteral("Close")); }, 900,
      [] {
        QWidget* dialog = QApplication::activeModalWidget();
        return Expect(dialog != nullptr &&
                          dialog->objectName() == QStringLiteral("CloseDialog"),
                      QStringLiteral("closing didn't ask"));
      });
  Add(
      QStringLiteral("cancel-close"),
      [this] {
        if (QWidget* dialog = QApplication::activeModalWidget()) {
          Click(dialog, QStringLiteral("Cancel"));
        }
      },
      600,
      [window] {
        return Expect(window->isVisible(), QStringLiteral("the window closed"));
      });
  Add(
      QStringLiteral("back-to-projects"),
      [this, window] { Click(window, QStringLiteral("All projects")); }, 800,
      [window] {
        return Expect(!App::Get().session().has_project() &&
                          window->start_screen()->isVisible(),
                      QStringLiteral("the start screen didn't come back"));
      });
  BuildMixer(window);
}

void Scenario::BuildMixer(MainWindow* window) {
  const auto undo = [this, window] { Click(window, QStringLiteral("Undo")); };
  const auto channel = [](const char* field) {
    const bridge::MixerView view = App::Get().session().mixer();
    if (view.channels.empty()) return QJsonValue();
    return QJsonDocument::fromJson(
               ToQ(view.channels.at(0).settings_json).toUtf8())
        .object()
        .value(QLatin1String(field));
  };
  const auto strip = [window]() -> QFrame* {
    for (auto* frame : window->mixer_dock()->widget()->findChildren<QFrame*>(
             QStringLiteral("MixerStrip"))) {
      if (frame->isVisible() && frame->property("bus").isNull()) return frame;
    }
    return nullptr;
  };
  auto memo = std::make_shared<Memo>();
  Add(QStringLiteral("pick-social-teaser"), [this, window] {
    ClickLater(window, QStringLiteral("Social teaser (9:16)"));
  });
  Add(
      QStringLiteral("create-social-teaser"),
      [this] {
        QWidget* dialog = QApplication::activeModalWidget();
        if (dialog == nullptr) {
          Miss(QStringLiteral("no dialog"));
          return;
        }
        const QList<QLineEdit*> lines = dialog->findChildren<QLineEdit*>();
        if (lines.size() < 2) {
          Miss(QStringLiteral("the dialog has no name and location fields"));
          return;
        }
        for (int i = 0; i < 2; ++i) {
          QLineEdit* line = lines.at(i);
          line->setFocus();
          QTest::keyClick(line, Qt::Key_A, Qt::ControlModifier);
          QTest::keyClicks(
              line, i == 0 ? QStringLiteral("Teaser")
                           : QDir(qEnvironmentVariable("OPSIS_USER_DATA"))
                                 .filePath(QStringLiteral("projects")));
        }
        Click(dialog, QStringLiteral("Create"));
      },
      2500,
      [] {
        return Expect(ToQ(Graph().name) == QStringLiteral("Teaser"),
                      QStringLiteral("the teaser didn't open"));
      });
  Add(
      QStringLiteral("pick-music-track"),
      [this, window] {
        GraphCanvas* canvas = window->canvas();
        const bridge::GraphView graph = Graph();
        const auto* card = CardTitled(graph, QStringLiteral("Music"));
        if (card == nullptr) {
          Miss(QStringLiteral("no Music card"));
          return;
        }
        Click(canvas, QStringLiteral("Fit to view"));
        ClickAt(canvas->viewport(), canvas->ViewPointOf(ToQ(card->id)));
        QTest::qWait(kSettleMs);
        for (QComboBox* combo :
             window->inspector_dock()->widget()->findChildren<QComboBox*>()) {
          if (combo->isVisible() && combo->count() > 1) {
            combo->setFocus();
            QTest::keyClick(combo, Qt::Key_Down);
            return;
          }
        }
        Miss(QStringLiteral("no music list"));
      },
      1500,
      [] {
        return Expect(!App::Get().session().mixer().channels.empty(),
                      QStringLiteral("picking music made no channel"));
      });
  Add(
      QStringLiteral("mixer-shows-strips"),
      [this] { ClickMenu(QStringLiteral("View"), QStringLiteral("Mixer")); },
      1500,
      [window, strip] {
        const int strips = static_cast<int>(
            window->mixer_dock()
                ->widget()
                ->findChildren<QFrame*>(QStringLiteral("MixerStrip"))
                .size());
        return Expect(strip() != nullptr && strips >= 2,
                      QStringLiteral("%1 mixer strips").arg(strips));
      });
  Add(
      QStringLiteral("mixer-fader-drag"),
      [this, strip, channel, memo] {
        QFrame* shell = strip();
        Fader* fader = shell != nullptr ? shell->findChild<Fader*>() : nullptr;
        if (fader == nullptr) {
          Miss(QStringLiteral("no channel fader"));
          return;
        }
        memo->x = channel("fader").toDouble();
        const QPoint thumb(fader->width() / 2,
                           static_cast<int>(fader->ThumbY()));
        Drag(fader, thumb, thumb + QPoint(0, -20), 6);
      },
      800,
      [channel, memo] {
        const double fader = channel("fader").toDouble();
        return Expect(fader > memo->x,
                      QStringLiteral("the fader stayed at %1").arg(fader));
      });
  Add(QStringLiteral("undo-mixer-fader"), undo, 700, [channel, memo] {
    const double fader = channel("fader").toDouble();
    return Expect(std::abs(fader - memo->x) < 1e-9,
                  QStringLiteral("one undo left the fader at %1").arg(fader));
  });
  Add(
      QStringLiteral("mixer-mute"),
      [this, strip] {
        if (QFrame* shell = strip()) Click(shell, QStringLiteral("M"));
      },
      700,
      [channel] {
        return Expect(channel("mute").toBool(),
                      QStringLiteral("the channel isn't muted"));
      });
  Add(QStringLiteral("undo-mixer-mute"), undo, 700, [channel] {
    return Expect(!channel("mute").toBool(),
                  QStringLiteral("the channel is still muted"));
  });
  Add(
      QStringLiteral("mixer-eq-popover"),
      [this, strip] {
        if (QFrame* shell = strip()) Click(shell, QStringLiteral("Equaliser"));
      },
      700,
      [] {
        QWidget* popup = QApplication::activePopupWidget();
        return Expect(popup != nullptr && popup->accessibleName() ==
                                              QStringLiteral("Equaliser"),
                      QStringLiteral("no equaliser opened"));
      });
  Add(
      QStringLiteral("mixer-eq-switch"),
      [] {
        QWidget* popup = QApplication::activePopupWidget();
        if (popup == nullptr) return;
        for (auto* toggle : popup->findChildren<ToggleSwitch*>()) {
          if (toggle->accessibleName() == QStringLiteral("Equaliser on")) {
            QTest::mouseClick(toggle, Qt::LeftButton);
          }
        }
        QTest::keyClick(popup, Qt::Key_Escape);
      },
      800,
      [channel] {
        return Expect(
            channel("eq").toObject().value(QStringLiteral("on")).toBool(),
            QStringLiteral("the equaliser didn't turn on"));
      });
  Add(QStringLiteral("undo-mixer-eq"), undo, 700, [channel] {
    return Expect(
        !channel("eq").toObject().value(QStringLiteral("on")).toBool(),
        QStringLiteral("the equaliser is still on"));
  });
  Add(
      QStringLiteral("mixer-levels"), [] {}, 6000,
      [] {
        const bridge::MixerView view = App::Get().session().mixer();
        if (view.channels.empty()) return QStringLiteral("no channels");
        const bridge::MeterView level = App::Get().session().meter(
            ToRust(QStringLiteral("ch:") + ToQ(view.channels.at(0).id)), 30);
        return Expect(level.has && !view.stale,
                      QStringLiteral("no levels after the mix rendered"));
      });
  Add(
      QStringLiteral("mix-stem-decoded"), [] {}, 2000,
      [] {
        return Expect(App::Get().session().audio_ready(),
                      QStringLiteral("the mix wasn't decoded for playback"));
      });
  Add(
      QStringLiteral("timeline-waveform"),
      [this, window] {
        ClickMenu(QStringLiteral("View"), QStringLiteral("Timeline"));
        auto* panel =
            qobject_cast<TimelinePanel*>(window->timeline_dock()->widget());
        if (panel != nullptr && panel->canvas()->view() != nullptr &&
            !panel->canvas()->view()->audio.empty()) {
          const auto& sound = panel->canvas()->view()->audio.at(0);
          App::Get().session().wave_columns(sound.key, 1, 0, 8);
        }
      },
      4000,
      [window] {
        auto* panel =
            qobject_cast<TimelinePanel*>(window->timeline_dock()->widget());
        if (panel == nullptr || panel->canvas()->view() == nullptr ||
            panel->canvas()->view()->audio.empty()) {
          return QStringLiteral("no sound bar on the timeline");
        }
        const auto& sound = panel->canvas()->view()->audio.at(0);
        const rust::Vec<float> columns =
            App::Get().session().wave_columns(sound.key, 1, 0, 8);
        return Expect(columns.size() == 16,
                      QStringLiteral("the music has no waveform yet"));
      });
}

}
