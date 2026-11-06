#include <QAction>
#include <QApplication>
#include <QDir>
#include <QDropEvent>
#include <QImage>
#include <QLabel>
#include <QLineEdit>
#include <QListWidget>
#include <QMenu>
#include <QMenuBar>
#include <QMimeData>
#include <QPointer>
#include <QPushButton>
#include <QScrollBar>
#include <QTextCursor>
#include <QTimer>
#include <QToolButton>
#include <QTreeWidget>
#include <QTreeWidgetItemIterator>
#include <QUrl>
#include <QtTest/QTest>
#include <cmath>
#include <memory>

#include "app.h"
#include "graph_canvas.h"
#include "main_window.h"
#include "node_list.h"
#include "opsis-qt/src/bridge.rs.h"
#include "scenario.h"
#include "start_screen.h"
#include "top_bar.h"
#include "var_text.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr int kDropSide = 64;
constexpr int kNudgeX = 120;
constexpr int kNudgeY = 60;
constexpr int kGrow = 60;
constexpr double kNear = 1.0;
constexpr int kSettleMs = 300;
constexpr int kEventuallyMs = 2000;
constexpr int kControlsWidth = 220;
constexpr double kBottomShareLow = 0.3;
constexpr double kBottomShareHigh = 0.5;

struct Memo {
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
        auto* search = list->findChild<QLineEdit*>();
        QTest::keyClick(search, Qt::Key_A, Qt::ControlModifier);
        QTest::keyClick(search, Qt::Key_Backspace);
        return problems.trimmed();
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
      [this, window] {
        const bridge::ProjectView project = App::Get().session().project();
        const QString problem =
            Expect(project.width == project.height,
                   QStringLiteral("the canvas is %1 × %2, not square")
                       .arg(project.width)
                       .arg(project.height));
        Click(window->inspector_dock()->widget(), QStringLiteral("16:9"));
        return problem;
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
  Add(QStringLiteral("undo-typing"), undo, 500);

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
  Add(QStringLiteral("undo-rename"), undo, 500);
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
        if (popup != nullptr) popup->close();
        return Expect(open, QStringLiteral("no category popover"));
      });
  Add(
      QStringLiteral("show-node-list-from-rail"),
      [this, window] {
        Click(window->rail(), QStringLiteral("Show node list"));
      },
      600,
      [window] {
        return Expect(window->nodes_dock()->isOpen(),
                      QStringLiteral("the node list didn't come back"));
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
}

}
