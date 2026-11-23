#include <kddockwidgets/LayoutSaver.h>
#include <kddockwidgets/core/DockWidget.h>
#include <kddockwidgets/qtwidgets/views/Separator.h>

#include <QApplication>
#include <QComboBox>
#include <QDialog>
#include <QDir>
#include <QEnterEvent>
#include <QFile>
#include <QFontInfo>
#include <QGraphicsItem>
#include <QGraphicsView>
#include <QImage>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QKeyEvent>
#include <QLabel>
#include <QLineEdit>
#include <QPlainTextEdit>
#include <QPointer>
#include <QPushButton>
#include <QSet>
#include <QTimer>
#include <QToolButton>
#include <QTreeWidget>
#include <QtTest/QTest>
#include <algorithm>
#include <cmath>
#include <memory>

#include "app.h"
#include "color_picker.h"
#include "event_hub.h"
#include "gradient_field.h"
#include "graph_canvas.h"
#include "main_window.h"
#include "node_list.h"
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

QString SavedFloating(const QString& dock) {
  const QByteArray saved = ToQ(App::Get().session().load_layout()).toUtf8();
  const QJsonArray floating = QJsonDocument::fromJson(saved)
                                  .object()
                                  .value("floatingWindows")
                                  .toArray();
  const bool named = QJsonDocument(floating)
                         .toJson(QJsonDocument::Compact)
                         .contains(dock.toUtf8());
  return QStringLiteral(
             "the saved layout has %1 bytes and %2 floating windows, %3 naming "
             "%4")
      .arg(saved.size())
      .arg(floating.size())
      .arg(named ? QStringLiteral("one") : QStringLiteral("none"))
      .arg(dock);
}

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

QString WidgetName(const QWidget* widget) {
  if (widget == nullptr) return QStringLiteral("nothing");
  return widget->objectName().isEmpty()
             ? QString::fromUtf8(widget->metaObject()->className())
             : widget->objectName();
}

struct CardCase {
  QString node_type;
  QString special;
  bridge::PortShape shape;
};

QWidget* ShownHoverCard() {
  for (QWidget* top : QApplication::topLevelWidgets()) {
    if (top->isVisible() && top->objectName() == QStringLiteral("HoverCard")) {
      return top;
    }
  }
  return nullptr;
}

QString HoverCardProblem(const CardCase& want) {
  const auto palette = App::Get().session().palette();
  const auto entry = std::find_if(
      palette.begin(), palette.end(),
      [&](const auto& e) { return ToQ(e.node_type) == want.node_type; });
  if (entry == palette.end()) {
    return QStringLiteral("%1 isn't in the node list").arg(want.node_type);
  }
  QWidget* card = ShownHoverCard();
  if (card == nullptr) {
    return QStringLiteral("hovering %1 showed no card").arg(want.node_type);
  }
  QStringList texts;
  int badge = 0;
  for (QLabel* label : card->findChildren<QLabel*>()) {
    texts << label->text();
    if (!label->pixmap().isNull()) {
      badge = static_cast<int>(label->pixmap().deviceIndependentSize().width());
    }
  }
  QStringList missing;
  int ports = 0;
  bool special = false;
  for (const auto* side : {&entry->inputs, &entry->outputs}) {
    for (const auto& port : *side) {
      ++ports;
      for (const QString& text : {ToQ(port.label), ToQ(port.note)}) {
        if (!text.isEmpty() && !texts.contains(text)) missing << text;
      }
      special |= ToQ(port.label) == want.special && port.shape == want.shape;
    }
  }
  const auto dots =
      card->findChildren<QWidget*>(QStringLiteral("PortDot")).size();
  const bool old_parts = texts.contains(ToQ(entry->category)) ||
                         texts.join(' ').contains(QChar(0x2192));
  return Expect(missing.isEmpty() && texts.contains(ToQ(entry->title)),
                QStringLiteral("the %1 card lacks %2")
                    .arg(want.node_type, missing.join(QStringLiteral(", ")))) +
         Expect(dots == ports, QStringLiteral(" it draws %1 dots for %2 ports")
                                   .arg(dots)
                                   .arg(ports)) +
         Expect(special, QStringLiteral(" %1 isn't drawn as its own kind")
                             .arg(want.special)) +
         Expect(!old_parts,
                QStringLiteral(" it still shows the group name or an arrow")) +
         Expect(badge == 40,
                QStringLiteral(" its icon is %1 px, not 40").arg(badge));
}

QRect InWindow(QWidget* widget, QWidget* window) {
  return {widget->mapTo(window, QPoint()), widget->size()};
}

QString BottomBetweenSides(MainWindow* window) {
  auto* bottom = window->timeline_dock();
  auto* nodes = window->nodes_dock();
  auto* inspector = window->inspector_dock();
  if (!bottom->isVisible() || !nodes->isVisible() || !inspector->isVisible()) {
    return QStringLiteral("the timeline, node list or Inspector isn't shown");
  }
  const QRect under = InWindow(bottom, window);
  const QRect left = InWindow(nodes, window);
  const QRect right = InWindow(inspector, window);
  return Expect(under.left() > left.right() && under.right() < right.left(),
                QStringLiteral("the bottom bar spans x %1 to %2 while the "
                               "node list ends at %3 and the Inspector "
                               "starts at %4")
                    .arg(under.left())
                    .arg(under.right())
                    .arg(left.right())
                    .arg(right.left()));
}

QColor MiddleColour(QWidget* widget) {
  const QImage shot = widget->grab().toImage();
  return shot.pixelColor(shot.width() / 2, shot.height() / 2);
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
          const QJsonValue old = Without(
              Without(QJsonDocument::fromJson(saver.serializeLayout()).object(),
                      QStringLiteral("script")),
              QStringLiteral("agent"));
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
        auto* timeline = window_->timeline_dock();
        return Expect(timeline->isFloating() && timeline->isOpen(),
                      QStringLiteral("the timeline came back docked; %1")
                          .arg(SavedFloating(timeline->uniqueName())));
      });
  Add(
      QStringLiteral("reopen-the-agent-tab-joins-the-inspector"), [] {}, 300,
      [this] {
        auto* agent = window_->agent_dock();
        return Expect(agent->isOpen() && !agent->isFloating() &&
                          agent->dockWidget()->isTabbed(),
                      QStringLiteral("the agent tab came back on its own"));
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

void Scenario::BuildAgent() {
  const QString projects = qEnvironmentVariable("OPSIS_QT_PROJECTS");
  Add(
      QStringLiteral("agent-past-setup"),
      [this] {
        if (Find(window_, QStringLiteral("Skip for now")) != nullptr) {
          Click(window_, QStringLiteral("Skip for now"));
        }
      },
      500, [] { return QString(); });
  Until(
      QStringLiteral("agent-create-a-project"),
      [projects] {
        App::Get().session().patch_settings(
            ToRust(QStringLiteral("{\"assistant\":{\"enabled\":true}}")));
        App::Get().session().create_project(ToRust(QStringLiteral("Agent")),
                                            ToRust(QStringLiteral("todomvc")),
                                            ToRust(projects));
      },
      kProjectMs, [] { return App::Get().session().has_project(); },
      [] { return QString(); });
  Until(
      QStringLiteral("agent-the-thread-list"),
      [this] {
        window_->agent_dock()->setAsCurrentTab();
        App::Get().session().agent_refresh();
      },
      kProjectMs,
      [this] {
        auto* list = window_->agent_dock()->findChild<QWidget*>(
            QStringLiteral("AgentThreads"));
        return list != nullptr && list->isVisible();
      },
      [] { return QString(); });
  Until(
      QStringLiteral("agent-a-new-thread"),
      [] { App::Get().session().agent_new_thread(); }, kProjectMs,
      [] {
        const bridge::AgentPanelView view = App::Get().session().agent_view();
        return view.live && ToQ(view.status) == QStringLiteral("ready");
      },
      [this] {
        auto* page = window_->agent_dock()->findChild<QWidget*>(
            QStringLiteral("AgentThreadPage"));
        return Expect(page != nullptr && page->isVisible(),
                      QStringLiteral("the new thread didn't open"));
      });
  Until(
      QStringLiteral("agent-asks-to-write-a-file"),
      [] {
        App::Get().session().agent_prompt(ToRust(QStringLiteral(
            "Write hello into notes.txt and tell me what you did")));
      },
      kProjectMs,
      [] { return !App::Get().session().agent_view().asks.empty(); },
      [this] {
        auto* allow = window_->agent_dock()->findChild<QPushButton*>(
            QStringLiteral("AgentAnswer"));
        return Expect(allow != nullptr,
                      QStringLiteral("the approval line has no answers"));
      });
  Until(
      QStringLiteral("agent-answered"),
      [] {
        const bridge::AgentPanelView view = App::Get().session().agent_view();
        const auto& ask = view.asks.front();
        for (const auto& option : ask.options) {
          if (ToQ(option.kind) == QStringLiteral("allowOnce")) {
            App::Get().session().agent_answer(ask.id, option.id);
          }
        }
      },
      kProjectMs,
      [] {
        const bridge::AgentPanelView view = App::Get().session().agent_view();
        return ToQ(view.status) == QStringLiteral("ready") &&
               view.entries.size() >= 3;
      },
      [] { return QString(); });
  Add(
      QStringLiteral("agent-the-tool-opens"),
      [] {
        const bridge::AgentPanelView view = App::Get().session().agent_view();
        for (const auto& entry : view.entries) {
          if (ToQ(entry.kind) == QStringLiteral("tools")) {
            App::Get().session().agent_toggle(entry.tools.front().id);
          }
        }
      },
      500,
      [this] {
        auto* detail = window_->agent_dock()->findChild<QLabel*>(
            QStringLiteral("AgentDetail"));
        return Expect(detail != nullptr && !detail->text().isEmpty(),
                      QStringLiteral("the tool's output didn't open"));
      });
  Add(
      QStringLiteral("agent-back-to-the-threads"),
      [] { App::Get().session().agent_back(); }, 500,
      [this] {
        auto* row = window_->agent_dock()->findChild<QPushButton*>(
            QStringLiteral("AgentThread"));
        return Expect(row != nullptr && row->accessibleName().startsWith(
                                            QStringLiteral("Write hello")),
                      QStringLiteral("the thread list lacks the thread"));
      });
  Add(
      QStringLiteral("agent-an-ended-thread"),
      [] {
        App::Get().session().agent_stop();
        const bridge::AgentPanelView view = App::Get().session().agent_view();
        App::Get().session().agent_open(view.threads.front().id);
      },
      500,
      [] {
        const bridge::AgentPanelView view = App::Get().session().agent_view();
        return Expect(!view.live && !view.entries.empty(),
                      QStringLiteral("the ended thread shows no messages"));
      });
  Until(
      QStringLiteral("agent-the-thread-goes-on"),
      [] {
        const bridge::AgentPanelView view = App::Get().session().agent_view();
        App::Get().session().agent_continue(view.thread,
                                            ToRust(QStringLiteral("Thanks")));
      },
      kProjectMs,
      [] {
        const bridge::AgentPanelView view = App::Get().session().agent_view();
        int users = 0;
        for (const auto& entry : view.entries) {
          if (ToQ(entry.kind) == QStringLiteral("user")) ++users;
        }
        return view.live && ToQ(view.status) == QStringLiteral("ready") &&
               users == 2;
      },
      [] { return QString(); });
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
  for (const CardCase& each :
       {CardCase{QStringLiteral("timeline.sequence"), QStringLiteral("Clips"),
                 bridge::PortShape::Numbered},
        CardCase{QStringLiteral("media.concat"), QStringLiteral("Files"),
                 bridge::PortShape::Numbered},
        CardCase{QStringLiteral("audio.mix"), QStringLiteral("Sounds"),
                 bridge::PortShape::Many}}) {
    Add(
        QStringLiteral("views-the-hover-card-names-the-ports-of-%1")
            .arg(each.node_type),
        [this, each] {
          auto* list = qobject_cast<NodeList*>(window_->nodes_dock()->widget());
          QTreeWidgetItem* row = list->RowOf(each.node_type);
          auto* tree = list->findChild<QTreeWidget*>();
          if (row == nullptr || tree == nullptr) return;
          tree->scrollToItem(row);
          emit tree->itemEntered(row, 0);
        },
        900, [each] { return HoverCardProblem(each); });
  }
  auto moving = std::make_shared<QString>();
  Until(
      QStringLiteral("views-the-hover-card-plays-the-node-motion"),
      [this, moving] {
        for (const auto& entry : App::Get().session().palette()) {
          if (!entry.motion.empty()) {
            *moving = ToQ(entry.node_type);
            break;
          }
        }
        auto* list = qobject_cast<NodeList*>(window_->nodes_dock()->widget());
        QTreeWidgetItem* row = list->RowOf(*moving);
        auto* tree = list->findChild<QTreeWidget*>();
        if (row == nullptr || tree == nullptr) return;
        tree->scrollToItem(row);
        emit tree->itemEntered(row, 0);
      },
      2000, [] { return ShownHoverCard() != nullptr; },
      [this, moving] {
        auto* list = qobject_cast<NodeList*>(window_->nodes_dock()->widget());
        const double elapsed =
            list->looks().MotionElapsed(NodeList::CardLook(*moving));
        return Expect(
            !moving->isEmpty() && elapsed >= 0,
            QStringLiteral("the card for %1 shows a still icon").arg(*moving));
      });
  Add(
      QStringLiteral("views-the-hover-card-goes-when-the-pointer-leaves"),
      [this] {
        auto* list = qobject_cast<NodeList*>(window_->nodes_dock()->widget());
        QEvent leave(QEvent::Leave);
        QCoreApplication::sendEvent(list->findChild<QTreeWidget*>(), &leave);
      },
      200,
      [] {
        return Expect(ShownHoverCard() == nullptr,
                      QStringLiteral("the hover card stays after the pointer "
                                     "leaves the list"));
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
  auto focus_while_running = std::make_shared<QPointer<QWidget>>();
  Until(
      QStringLiteral("views-running-from-the-button-keeps-focus-in-the-editor"),
      [this, focus_while_running] {
        ScriptPanel* panel = window_->script_panel();
        panel->editor()->setPlainText(
            QStringLiteral("opsis.sleep(0.5)\nprint(\"focus\")\n"));
        auto* run = panel->findChild<QPushButton*>(QStringLiteral("ScriptRun"));
        run->setFocus();
        QTest::mouseClick(run, Qt::LeftButton);
        QTest::qWait(250);
        *focus_while_running = panel->window()->focusWidget();
      },
      kProjectMs,
      [] {
        const bridge::ScriptView view = App::Get().session().script_view();
        return !view.running && ToQ(view.outcome) == QStringLiteral("ok") &&
               !view.output.empty() &&
               ToQ(view.output.back().text) == QStringLiteral("focus");
      },
      [this, focus_while_running] {
        QTest::qWait(200);
        ScriptPanel* panel = window_->script_panel();
        QWidget* during = focus_while_running->data();
        QWidget* after = panel->window()->focusWidget();
        return Expect(during == panel->editor() && after == panel->editor(),
                      QStringLiteral("focus is on %1 while the script runs and "
                                     "%2 after")
                          .arg(WidgetName(during), WidgetName(after)));
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
      QStringLiteral("views-the-script-list-stays-still"),
      [this] {
        quiet_from_ = EventHub::Instance()->dispatched(
            static_cast<std::uint8_t>(bridge::EventKind::Script));
      },
      1500,
      [this] {
        const int now = EventHub::Instance()->dispatched(
            static_cast<std::uint8_t>(bridge::EventKind::Script));
        return Expect(now == quiet_from_,
                      QStringLiteral("the idle script panel rebuilt %1 times")
                          .arg(now - quiet_from_));
      });
  Until(
      QStringLiteral("views-a-script-is-deleted-to-the-trash"),
      [] {
        App::Get().session().script_new();
        QTest::qWait(200);
        const bridge::ScriptView view = App::Get().session().script_view();
        App::Get().session().script_delete(view.dir, view.open);
      },
      kProjectMs,
      [] {
        const bridge::ScriptView view = App::Get().session().script_view();
        return std::none_of(
            view.files.begin(), view.files.end(), [](const auto& f) {
              return ToQ(f) == QStringLiteral("scripts/script.lua");
            });
      },
      [] {
        const bridge::ScriptView view = App::Get().session().script_view();
#ifdef Q_OS_LINUX
        const QString trashed =
            QDir(qEnvironmentVariable("XDG_DATA_HOME"))
                .filePath(QStringLiteral("Trash/files/script.lua"));
        const bool in_trash = QFile::exists(trashed);
#else
        const bool in_trash = !QFile::exists(
            QDir(ToQ(view.dir)).filePath(QStringLiteral("scripts/script.lua")));
#endif
        return Expect(
            ToQ(view.open) == QStringLiteral("scripts/titles.lua") && in_trash,
            QStringLiteral("after deleting, %1 is open and the trash "
                           "%2 it")
                .arg(ToQ(view.open), in_trash ? QStringLiteral("has")
                                              : QStringLiteral("lacks")));
      });
  Add(
      QStringLiteral("views-the-agent-tab-sits-with-the-inspector"), [] {}, 100,
      [this] {
        auto* agent = window_->agent_dock();
        return Expect(agent != nullptr && agent->isOpen() &&
                          agent->dockWidget()->isTabbed() &&
                          window_->inspector_dock()->dockWidget()->isTabbed(),
                      QStringLiteral("the agent tab isn't with the inspector"));
      });
  Add(
      QStringLiteral("views-the-agent-panel-opens-on-its-threads"),
      [this] { window_->agent_dock()->setAsCurrentTab(); }, 300,
      [this] {
        auto* list = window_->agent_dock()->findChild<QWidget*>(
            QStringLiteral("AgentThreads"));
        auto* search = window_->agent_dock()->findChild<QLineEdit*>(
            QStringLiteral("AgentSearch"));
        return Expect(list != nullptr && list->isVisible() && search != nullptr,
                      QStringLiteral("the agent panel isn't on its threads"));
      });
  Add(
      QStringLiteral("views-the-agent-panel-asks-for-the-assistant"), [] {},
      100,
      [this] {
        auto* on = window_->agent_dock()->findChild<QPushButton*>(
            QStringLiteral("AgentTurnOn"));
        const bool off = !App::Get().session().settings().assistant_enabled;
        return Expect(
            (on != nullptr && on->isVisible()) == off,
            QStringLiteral("with the assistant %1 the panel %2 "
                           "offers to turn it on")
                .arg(off ? QStringLiteral("off") : QStringLiteral("on"),
                     on != nullptr && on->isVisible()
                         ? QString()
                         : QStringLiteral("never")));
      });
  Add(
      QStringLiteral("views-the-inspector-tab-comes-back"),
      [this] { window_->inspector_dock()->setAsCurrentTab(); }, 100,
      [this] {
        return Expect(window_->inspector_dock()->dockWidget()->isCurrentTab(),
                      QStringLiteral("the inspector isn't the current tab"));
      });
  Add(
      QStringLiteral("views-panels-have-wide-splitters"), [] {}, 100,
      [this] {
        int thin = 0;
        int seen = 0;
        int handled = 0;
        const QList<QWidget*> handles =
            window_->findChildren<QWidget*>(QStringLiteral("SplitterHandle"));
        for (auto* separator :
             window_->findChildren<KDDockWidgets::QtWidgets::Separator*>()) {
          if (!separator->isVisible()) continue;
          ++seen;
          if (std::min(separator->width(), separator->height()) < 6) ++thin;
          const bool across = separator->width() >= separator->height();
          const QSize box = across ? QSize(16, 12) : QSize(12, 16);
          const QPoint middle = separator->geometry().center();
          handled += static_cast<int>(
              std::any_of(handles.begin(), handles.end(), [&](QWidget* h) {
                return h->isVisible() &&
                       h->parentWidget() == separator->parentWidget() &&
                       h->size() == box &&
                       (h->geometry().center() - middle).manhattanLength() <= 1;
              }));
        }
        return Expect(seen > 0 && thin == 0,
                      QStringLiteral("%1 of %2 splitters are thinner than 6 px")
                          .arg(thin)
                          .arg(seen)) +
               Expect(handled == seen,
                      QStringLiteral(" %1 of %2 splitters have a centred "
                                     "12 by 16 handle")
                          .arg(handled)
                          .arg(seen));
      });
  Add(
      QStringLiteral("views-a-splitter-is-plain-again-once-the-mouse-leaves"),
      [] {}, 100,
      [this] {
        KDDockWidgets::QtWidgets::Separator* separator = nullptr;
        for (auto* each :
             window_->findChildren<KDDockWidgets::QtWidgets::Separator*>()) {
          if (each->isVisible()) separator = each;
        }
        if (separator == nullptr) {
          return Expect(false, QStringLiteral("no splitter is shown"));
        }
        const QPointF middle = QRectF(separator->rect()).center();
        const QColor before = MiddleColour(separator);
        QEnterEvent enter(middle, middle,
                          QPointF(separator->mapToGlobal(middle.toPoint())));
        QCoreApplication::sendEvent(separator, &enter);
        const QColor lit = MiddleColour(separator);
        QEvent leave(QEvent::Leave);
        QCoreApplication::sendEvent(separator, &leave);
        const QColor after = MiddleColour(separator);
        return Expect(lit != before && after == before,
                      QStringLiteral("the splitter's line is %1, then %2 on "
                                     "hover and %3 after")
                          .arg(before.name(), lit.name(), after.name()));
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
        window_->script_panel()->window()->resize(1920, 1080);
        QTest::qWait(300);
        const bool full = split->orientation() == Qt::Vertical;
        int small_buttons = 0;
        for (QToolButton* button :
             window_->script_panel()->window()->findChildren<QToolButton*>(
                 QStringLiteral("DockTitleButton"))) {
          if (button->isVisible() && button->width() < 24) ++small_buttons;
        }
        window_->script_dock()->setFloating(false);
        return Expect(tall && wide && full,
                      QStringLiteral("the split didn't turn (tall %1, wide "
                                     "%2, full screen %3)")
                          .arg(tall)
                          .arg(wide)
                          .arg(full)) +
               Expect(small_buttons == 0,
                      QStringLiteral(" %1 title buttons are smaller than 24 px")
                          .arg(small_buttons));
      });
  Add(
      QStringLiteral("views-a-closed-popped-out-panel-docks-back"),
      [this] {
        window_->script_dock()->setFloating(true);
        QTest::qWait(200);
        window_->script_panel()->window()->close();
      },
      500,
      [this] {
        auto* script = window_->script_dock();
        return Expect(
            script->isOpen() && !script->isFloating() &&
                script->dockWidget()->isTabbed(),
            QStringLiteral("after closing its window the script panel "
                           "is %1")
                .arg(script->isOpen() ? QStringLiteral("not docked")
                                      : QStringLiteral("closed")));
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
      [this] {
        QTest::qWait(200);
        return Expect(App::Get().session().has_project(),
                      QStringLiteral("starting with the last project opened "
                                     "nothing")) +
               BottomBetweenSides(window_);
      });
  Add(
      QStringLiteral("views-the-bottom-bar-comes-back-between-the-sidebars"),
      [this] {
        window_->RunCommand(QStringLiteral("view:toggle-bottom"));
        QTest::qWait(100);
        window_->RunCommand(QStringLiteral("view:toggle-bottom"));
      },
      300, [this] { return BottomBetweenSides(window_); });
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
