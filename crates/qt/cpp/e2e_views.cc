#include <kddockwidgets/LayoutSaver.h>
#include <kddockwidgets/core/DockWidget.h>

#include <QApplication>
#include <QComboBox>
#include <QDialog>
#include <QDir>
#include <QFontInfo>
#include <QGraphicsItem>
#include <QGraphicsView>
#include <QImage>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QKeyEvent>
#include <QPlainTextEdit>
#include <QPushButton>
#include <QSet>
#include <QTimer>
#include <QToolButton>
#include <QtTest/QTest>
#include <algorithm>
#include <cmath>
#include <memory>

#include "app.h"
#include "color_picker.h"
#include "gradient_field.h"
#include "graph_canvas.h"
#include "main_window.h"
#include "opsis-qt/src/bridge.rs.h"
#include "scenario.h"
#include "script_panel.h"
#include "start_screen.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr int kProjectMs = 20'000;
constexpr int kSwatches = 14;
constexpr int kBarWidth = 300;

QString Expect(bool ok, const QString& what) { return ok ? QString() : what; }

QColor IconInk(const QAbstractButton* button) {
  const QImage icon = button->icon()
                          .pixmap(button->iconSize())
                          .toImage()
                          .convertToFormat(QImage::Format_ARGB32);
  for (int y = 0; y < icon.height(); ++y) {
    for (int x = 0; x < icon.width(); ++x) {
      const QColor pixel = icon.pixelColor(x, y);
      if (pixel.alpha() == 255) return pixel;
    }
  }
  return {};
}

QSet<QString> SelectedCards(GraphCanvas* canvas) {
  QSet<QString> ids;
  for (QGraphicsItem* item : canvas->scene()->selectedItems()) {
    if (auto* card = dynamic_cast<CardItem*>(item)) ids.insert(card->id());
  }
  return ids;
}

void Select(const QStringList& ids) {
  rust::Vec<rust::String> chosen;
  for (const QString& id : ids) chosen.push_back(ToRust(id));
  App::Get().session().select(std::move(chosen));
}

void ChooseStart(const QString& choice) {
  App::Get().session().patch_settings(
      ToRust(QStringLiteral(R"({"app":{"onStart":"%1"}})").arg(choice)));
}

QJsonValue Without(const QJsonValue& value, const QString& name) {
  if (value.isArray()) {
    QJsonArray kept;
    for (const QJsonValue& item : value.toArray()) {
      if (item.toString() == name ||
          item.toObject().value("uniqueName").toString() == name) {
        continue;
      }
      kept.append(Without(item, name));
    }
    return kept;
  }
  if (value.isObject()) {
    QJsonObject kept;
    const QJsonObject object = value.toObject();
    for (auto it = object.begin(); it != object.end(); ++it) {
      kept.insert(it.key(), Without(it.value(), name));
    }
    return kept;
  }
  return value;
}

bool HasCard(const QString& id, const QString& title) {
  for (const auto& card : App::Get().session().graph().nodes) {
    if (ToQ(card.id) == id && (title.isEmpty() || ToQ(card.title) == title)) {
      return true;
    }
  }
  return false;
}

QString Overlaps(const QWidget* parent) {
  QList<QWidget*> parts;
  for (QObject* child : parent->children()) {
    auto* part = qobject_cast<QWidget*>(child);
    if (part != nullptr && part->isVisible()) parts << part;
  }
  for (int i = 0; i < parts.size(); ++i) {
    for (int j = i + 1; j < parts.size(); ++j) {
      if (parts.at(i)->geometry().intersects(parts.at(j)->geometry())) {
        return QStringLiteral("%1 covers %2")
            .arg(parts.at(j)->accessibleName(), parts.at(i)->accessibleName());
      }
    }
  }
  return {};
}

}

void Scenario::BuildReopen() {
  const QDir projects(qEnvironmentVariable("OPSIS_QT_PROJECTS"));
  const QStringList made =
      projects.entryList(QDir::Dirs | QDir::NoDotAndDotDot);
  Add(
      QStringLiteral("reopen-past-setup"),
      [this] {
        if (Find(window_, QStringLiteral("Skip for now")) != nullptr) {
          Click(window_, QStringLiteral("Skip for now"));
        }
      },
      500,
      [this] {
        return Expect(window_->start_screen()->isVisible(),
                      QStringLiteral("the start screen isn't showing"));
      });
  if (made.isEmpty()) {
    Until(
        QStringLiteral("reopen-create-a-project"),
        [projects] {
          App::Get().session().create_project(ToRust(QStringLiteral("Reopen")),
                                              ToRust(QStringLiteral("todomvc")),
                                              ToRust(projects.absolutePath()));
        },
        kProjectMs, [] { return App::Get().session().has_project(); },
        [] { return QString(); });
    Add(
        QStringLiteral("reopen-pop-the-timeline-out"),
        [this] { window_->timeline_dock()->setFloating(true); }, 500,
        [this] {
          return Expect(window_->timeline_dock()->isFloating(),
                        QStringLiteral("the timeline didn't pop out"));
        });
    Add(
        QStringLiteral("reopen-save-a-layout-from-before-the-script-tab"),
        [] {
          KDDockWidgets::LayoutSaver saver;
          const QJsonValue old =
              Without(QJsonDocument::fromJson(saver.serializeLayout()).object(),
                      QStringLiteral("script"));
          App::Get().session().save_layout(ToRust(QString::fromUtf8(
              QJsonDocument(old.toObject()).toJson(QJsonDocument::Compact))));
        },
        200,
        [] {
          return Expect(
              !ToQ(App::Get().session().load_layout())
                   .contains(QStringLiteral("\"script\"")),
              QStringLiteral("the saved layout still names the script tab"));
        });
    Add(
        QStringLiteral("reopen-close-the-project"),
        [] { App::Get().session().close_project(); }, 500,
        [] {
          return Expect(!App::Get().session().has_project(),
                        QStringLiteral("the project is still open"));
        });
    return;
  }
  Until(
      QStringLiteral("reopen-open-the-project-again"),
      [projects, made] {
        App::Get().session().open_project(
            ToRust(projects.absoluteFilePath(made.first())));
      },
      kProjectMs, [] { return App::Get().session().has_project(); },
      [] { return QString(); });
  Add(
      QStringLiteral("reopen-the-timeline-is-still-popped-out"), [] {}, 500,
      [this] {
        return Expect(window_->timeline_dock()->isFloating() &&
                          window_->timeline_dock()->isOpen(),
                      QStringLiteral("the timeline came back docked"));
      });
  Add(
      QStringLiteral("reopen-the-script-tab-joins-the-bottom-group"), [] {},
      300,
      [this] {
        auto* script = window_->script_dock();
        return Expect(script->isOpen() && !script->isFloating() &&
                          script->dockWidget()->isTabbed(),
                      QStringLiteral("the script tab came back on its own"));
      });
}

void Scenario::BuildViews() {
  const QString projects = qEnvironmentVariable("OPSIS_QT_PROJECTS");
  GraphCanvas* canvas = window_->canvas();
  Add(
      QStringLiteral("views-skip-setup"),
      [this] { Click(window_, QStringLiteral("Skip for now")); }, 500,
      [this] {
        return Expect(window_->start_screen()->isVisible(),
                      QStringLiteral("skipping setup didn't show the start "
                                     "screen"));
      });
  Until(
      QStringLiteral("views-create-a-project"),
      [projects] {
        App::Get().session().create_project(ToRust(QStringLiteral("Views")),
                                            ToRust(QStringLiteral("todomvc")),
                                            ToRust(projects));
      },
      kProjectMs,
      [] {
        return App::Get().session().has_project() &&
               !App::Get().session().graph().nodes.empty();
      },
      [canvas] {
        const auto cards = App::Get().session().graph().nodes.size();
        return Expect(cards > 0 && canvas->isVisible(),
                      QStringLiteral("the canvas shows %1 cards").arg(cards));
      });
  Add(
      QStringLiteral("views-canvas-toggles-show-their-state"), [] {}, 100,
      [this, canvas] {
        QAbstractButton* grid = Find(canvas, QStringLiteral("Hide the grid"));
        QAbstractButton* map = Find(canvas, QStringLiteral("Hide the minimap"));
        if (map == nullptr) {
          map = Find(canvas, QStringLiteral("Show the minimap"));
        }
        const bool carded = grid != nullptr && map != nullptr &&
                            grid->parentWidget()->objectName() ==
                                QStringLiteral("CanvasToolbar") &&
                            map->parentWidget()->objectName() ==
                                QStringLiteral("CanvasToolbar");
        return Expect(carded, QStringLiteral("the grid and minimap buttons "
                                             "aren't in toolbar cards")) +
               Expect(grid != nullptr &&
                          grid->isChecked() == canvas->GridShown() &&
                          map != nullptr &&
                          map->isChecked() == canvas->minimap_visible(),
                      QStringLiteral(" a toggle doesn't show its state"));
      });
  Add(
      QStringLiteral("views-canvas-mirrors-the-selection"),
      [] {
        Select({QStringLiteral("open"), QStringLiteral("todo1")});
        QTest::qWait(100);
        Select({QStringLiteral("todo2")});
      },
      300,
      [canvas] {
        const QSet<QString> shown = SelectedCards(canvas);
        return Expect(shown == QSet<QString>{QStringLiteral("todo2")},
                      QStringLiteral("the canvas shows %1 selected")
                          .arg(QStringList(shown.values()).join(", ")));
      });
  Add(
      QStringLiteral("views-inspector-follows-the-selection"), [] {}, 300,
      [] {
        return Expect(
            ToQ(App::Get().session().inspector().id) == QStringLiteral("todo2"),
            QStringLiteral("the Inspector shows '%1'")
                .arg(ToQ(App::Get().session().inspector().id)));
      });
  Add(
      QStringLiteral("views-colour-picker-fits-its-swatches"),
      [this] {
        QStringList swatches;
        for (int i = 0; i < kSwatches; ++i) {
          swatches << QColor::fromHsv(i * 360 / kSwatches, 200, 220).name();
        }
        ColorPicker::Open(window_, QStringLiteral("#3b82f6"),
                          QStringLiteral("#000000"), swatches,
                          [](const QString&) {});
      },
      400,
      [] {
        auto* picker =
            qobject_cast<ColorPicker*>(QApplication::activePopupWidget());
        if (picker == nullptr) return QStringLiteral("no colour picker opened");
        const QString overlap = Overlaps(picker);
        picker->close();
        return Expect(overlap.isEmpty(),
                      QStringLiteral("in the picker, %1").arg(overlap));
      });
  Add(
      QStringLiteral("views-gradient-stops-sit-on-the-bar"), [] {}, 100,
      [] {
        auto bar = std::make_unique<StopBar>();
        bar->resize(kBarWidth, bar->sizeHint().height());
        bar->SetStops({{QStringLiteral("#000000"), 0.0},
                       {QStringLiteral("#ffffff"), 100.0}},
                      true, 0);
        const double middle = bar->Track().center().y();
        return Expect(std::abs(bar->HandleOf(0).y() - middle) < 0.5 &&
                          std::abs(bar->HandleOf(1).y() - middle) < 0.5,
                      QStringLiteral("the stops sit at %1, the bar's middle "
                                     "at %2")
                          .arg(bar->HandleOf(0).y())
                          .arg(middle));
      });
  Add(
      QStringLiteral("views-button-icon-follows-its-style"), [] {}, 100,
      [] {
        std::unique_ptr<QPushButton> button(MakeButton(
            QStringLiteral("Download"), "primary", QStringLiteral("download")));
        const QColor primary = IconInk(button.get());
        SetButtonVariant(button.get(), "outline", QStringLiteral("download"));
        const QColor outline = IconInk(button.get());
        App& app = App::Get();
        return Expect(
            primary == app.Token(QStringLiteral("primary-foreground")) &&
                outline == app.Token(QStringLiteral("foreground")),
            QStringLiteral("the icon is %1 then %2")
                .arg(primary.name(), outline.name()));
      });
  Add(
      QStringLiteral("views-script-is-a-tab-in-the-bottom-group"),
      [this] { window_->RunCommand(QStringLiteral("view:focus-script")); }, 300,
      [this] {
        auto* script = window_->script_dock();
        return Expect(script->isOpen() && !script->isFloating() &&
                          script->dockWidget()->isTabbed() &&
                          script->dockWidget()->isCurrentTab(),
                      QStringLiteral("the script tab isn't in the bottom "
                                     "group"));
      });
  Until(
      QStringLiteral("views-a-script-adds-a-node"),
      [this] {
        App::Get().session().script_new();
        QTest::qWait(200);
        ScriptPanel* panel = window_->script_panel();
        panel->editor()->setPlainText(QStringLiteral(
            "opsis.nodes.add { type = \"browser.wait\", id = \"scripted\" "
            "}\n"));
        panel->Run();
      },
      kProjectMs, [] { return HasCard(QStringLiteral("scripted"), QString()); },
      [] {
        return Expect(HasCard(QStringLiteral("scripted"), QString()),
                      QStringLiteral("the script's node isn't on the canvas"));
      });
  Until(
      QStringLiteral("views-a-failed-script-marks-its-line-and-undoes"),
      [this] {
        ScriptPanel* panel = window_->script_panel();
        panel->editor()->setPlainText(QStringLiteral(
            "opsis.nodes.scripted.label = \"Changed\"\n\nerror(\"broken\")\n"));
        panel->Run();
      },
      kProjectMs,
      [] {
        const bridge::ScriptView view = App::Get().session().script_view();
        return !view.running && ToQ(view.outcome) == QStringLiteral("failed");
      },
      [this] {
        QTest::qWait(200);
        const bridge::ScriptView view = App::Get().session().script_view();
        const int line =
            window_->script_panel()->editor()->textCursor().blockNumber() + 1;
        return Expect(view.failure_line == 3 && line == 3,
                      QStringLiteral("the failure marks line %1 of %2")
                          .arg(line)
                          .arg(view.failure_line)) +
               Expect(view.rolled_back && !HasCard(QStringLiteral("scripted"),
                                                   QStringLiteral("Changed")),
                      QStringLiteral(" the failed script's edit stayed"));
      });
  Until(
      QStringLiteral("views-script-text-is-monospace-and-keeps-its-spaces"),
      [this] {
        ScriptPanel* panel = window_->script_panel();
        panel->editor()->setPlainText(
            QStringLiteral("print(\"  indented\", \"tabbed\")\n"));
        panel->Run();
      },
      kProjectMs,
      [] {
        const bridge::ScriptView view = App::Get().session().script_view();
        return !view.running && ToQ(view.outcome) == QStringLiteral("ok");
      },
      [this] {
        QTest::qWait(200);
        ScriptPanel* panel = window_->script_panel();
        auto* output =
            panel->findChild<QPlainTextEdit*>(QStringLiteral("ScriptOutput"));
        const QString shown =
            output == nullptr ? QString() : output->toPlainText();
        const QFontInfo font(panel->editor()->font());
        return Expect(font.fixedPitch(),
                      QStringLiteral("the script is drawn in %1")
                          .arg(font.family())) +
               Expect(shown.contains(QStringLiteral("  indented\ttabbed")),
                      QStringLiteral(" the output reads '%1'").arg(shown));
      });
  Add(
      QStringLiteral("views-the-code-theme-and-size-restyle-the-script"),
      [] {
        App::Get().session().patch_settings(ToRust(
            QStringLiteral(R"({"script":{"theme":"gruvbox","fontSize":16}})")));
      },
      400,
      [this] {
        ScriptPanel* panel = window_->script_panel();
        const bridge::ScriptLookView look = App::Get().session().script_look();
        auto* theme =
            panel->findChild<QComboBox*>(QStringLiteral("ScriptTheme"));
        const QFontInfo font(panel->editor()->font());
        const bool styled =
            panel->editor()->styleSheet().contains(ToQ(look.background));
        return Expect(
            ToQ(look.theme) == QStringLiteral("gruvbox") && styled &&
                font.pixelSize() == 16 && theme != nullptr &&
                theme->currentData().toString() == QStringLiteral("gruvbox"),
            QStringLiteral("the script shows theme %1 at %2 px")
                .arg(ToQ(look.theme))
                .arg(font.pixelSize()));
      });
  Add(
      QStringLiteral("views-a-script-is-renamed"),
      [] {
        const bridge::ScriptView view = App::Get().session().script_view();
        App::Get().session().script_rename(view.dir, view.open,
                                           ToRust(QStringLiteral("titles")));
      },
      400,
      [this] {
        const bridge::ScriptView view = App::Get().session().script_view();
        const bool old_gone = std::none_of(
            view.files.begin(), view.files.end(), [](const auto& f) {
              return ToQ(f) == QStringLiteral("scripts/script.lua");
            });
        auto* files = window_->script_panel()->findChild<QComboBox*>(
            QStringLiteral("ScriptFiles"));
        return Expect(
            ToQ(view.open) == QStringLiteral("scripts/titles.lua") &&
                old_gone && files != nullptr &&
                files->currentText() == QStringLiteral("scripts/titles.lua"),
            QStringLiteral("the open script is %1").arg(ToQ(view.open)));
      });
  Add(
      QStringLiteral("views-the-script-split-turns-with-the-panel"),
      [this] {
        window_->script_dock()->setFloating(true);
        QTest::qWait(100);
        window_->script_panel()->window()->resize(400, 800);
      },
      400,
      [this] {
        QSplitter* split = window_->script_panel()->split();
        const bool tall = split->orientation() == Qt::Vertical;
        window_->script_panel()->window()->resize(1200, 300);
        QTest::qWait(300);
        const bool wide = split->orientation() == Qt::Horizontal;
        int small_buttons = 0;
        for (QToolButton* button :
             window_->script_panel()->window()->findChildren<QToolButton*>(
                 QStringLiteral("DockTitleButton"))) {
          if (button->isVisible() && button->width() < 24) ++small_buttons;
        }
        window_->script_dock()->setFloating(false);
        return Expect(tall && wide,
                      QStringLiteral("the split didn't turn (tall %1, wide %2)")
                          .arg(tall)
                          .arg(wide)) +
               Expect(small_buttons == 0,
                      QStringLiteral(" %1 title buttons are smaller than 24 px")
                          .arg(small_buttons));
      });
  Until(
      QStringLiteral("views-start-reopens-the-last-project"),
      [this] {
        ChooseStart(QStringLiteral("last-project"));
        App::Get().session().close_project();
        QTimer::singleShot(0, window_, [this] { window_->StartAsChosen(); });
      },
      kProjectMs,
      [] {
        return App::Get().session().has_project() &&
               !App::Get().session().graph().nodes.empty();
      },
      [] {
        return Expect(App::Get().session().has_project(),
                      QStringLiteral("starting with the last project opened "
                                     "nothing"));
      });
  Until(
      QStringLiteral("views-start-asks-for-a-new-project"),
      [this] {
        ChooseStart(QStringLiteral("new-project"));
        App::Get().session().close_project();
        QTimer::singleShot(0, window_, [this] { window_->StartAsChosen(); });
      },
      kProjectMs,
      [] {
        const QWidget* dialog = QApplication::activeModalWidget();
        return dialog != nullptr &&
               dialog->objectName() == QStringLiteral("NewProjectDialog");
      },
      [] {
        auto* dialog =
            qobject_cast<QDialog*>(QApplication::activeModalWidget());
        const bool asked =
            dialog != nullptr &&
            dialog->objectName() == QStringLiteral("NewProjectDialog");
        if (dialog != nullptr) dialog->reject();
        ChooseStart(QStringLiteral("start-screen"));
        return Expect(asked, QStringLiteral("starting with a new project "
                                            "didn't open New project"));
      });
}

}
