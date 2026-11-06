#include "scenario.h"

#include <QApplication>
#include <QElapsedTimer>
#include <QFile>
#include <QLineEdit>
#include <QTimer>
#include <QtTest/QTest>

#include "app.h"
#include "graph_canvas.h"
#include "inspector_panel.h"
#include "main_window.h"
#include "opsis-qt/src/bridge.rs.h"
#include "settings_window.h"
#include "widgets.h"

namespace opsis {

namespace {

QString Rss() {
  QFile status(QStringLiteral("/proc/self/status"));
  if (!status.open(QIODevice::ReadOnly)) return QStringLiteral("unknown");
  for (const QByteArray& line : status.readAll().split('\n')) {
    if (line.startsWith("VmRSS:")) {
      return QString::fromLatin1(line.mid(6).trimmed());
    }
  }
  return QStringLiteral("unknown");
}

}

void Scenario::MaybeStart(MainWindow* window) {
  const QString name = qEnvironmentVariable("OPSIS_QT_SCENARIO");
  if (name.isEmpty()) return;
  const QString platform = QApplication::platformName();
  if (platform != QStringLiteral("xcb") &&
      platform != QStringLiteral("offscreen")) {
    App::Get().Log(QStringLiteral("scenarios run only in a virtual display; "
                                  "this is %1, so quitting")
                       .arg(platform));
    QTimer::singleShot(0, qApp, &QApplication::quit);
    return;
  }
  QString dir = qEnvironmentVariable("OPSIS_QT_SHOTS");
  if (dir.isEmpty()) {
    dir = QDir::homePath() +
          QStringLiteral("/.cache/opsis-qt-shots/") +
          name;
  }
  QDir shots(dir);
  shots.mkpath(QStringLiteral("."));
  new Scenario(window, name, shots);
}

Scenario::Scenario(MainWindow* window, const QString& name, const QDir& shots)
    : QObject(window), window_(window), name_(name), shots_(shots) {
  if (name == QStringLiteral("bench")) {
    const int count = qEnvironmentVariableIntValue("OPSIS_QT_BENCH_NODES");
    BuildBench(count > 0 ? count : 200);
  } else {
    BuildTour();
  }
  QTimer::singleShot(1200, this, &Scenario::Next);
}

void Scenario::Add(const QString& name, std::function<void()> action,
                   int wait_ms) {
  steps_.push_back(Step{name, std::move(action), wait_ms});
}

void Scenario::Next() {
  if (index_ >= steps_.size()) {
    App::Get().Log(QStringLiteral("scenario %1 done, rss %2").arg(name_, Rss()));
    QApplication::quit();
    return;
  }
  const Step& step = steps_[index_];
  App::Get().Log(QStringLiteral("step %1: %2").arg(index_ + 1).arg(step.name));
  step.action();
  const int wait = step.wait_ms;
  const QString name = step.name;
  const std::size_t number = ++index_;
  QTimer::singleShot(wait, this, [this, name, number] {
    Shoot(QStringLiteral("%1-%2").arg(number, 2, 10, QLatin1Char('0')).arg(name));
    Next();
  });
}

void Scenario::Shoot(const QString& name) {
  int count = 0;
  for (QWidget* widget : QApplication::topLevelWidgets()) {
    if (!widget->isVisible() || !widget->isWindow()) continue;
    if (widget->windowType() == Qt::ToolTip || widget->windowType() == Qt::Popup) continue;
    if (widget->width() < 40 || widget->height() < 40) continue;
    const QString suffix = count == 0 ? QString()
                                      : QStringLiteral("-%1").arg(widget->objectName().isEmpty()
                                                                      ? QString::number(count)
                                                                      : widget->objectName());
    widget->grab().save(shots_.filePath(name + suffix + QStringLiteral(".png")));
    ++count;
  }
}

void Scenario::Drag(QWidget* target, QPoint from, QPoint to, int steps) {
  QTest::mousePress(target, Qt::LeftButton, {}, from);
  for (int i = 1; i <= steps; ++i) {
    const QPoint at = from + (to - from) * i / steps;
    QMouseEvent move(QEvent::MouseMove, QPointF(at), target->mapToGlobal(QPointF(at)),
                     Qt::NoButton, Qt::LeftButton, Qt::NoModifier);
    QApplication::sendEvent(target, &move);
  }
  QTest::mouseRelease(target, Qt::LeftButton, {}, to);
}

void Scenario::BuildTour() {
  MainWindow* window = window_;
  GraphCanvas* canvas = window->canvas();
  auto first_card = [] {
    const bridge::GraphView graph = App::Get().session().graph();
    for (const auto& card : graph.nodes) {
      if (card.kind == bridge::CardKind::Card && !card.inputs.empty()) {
        return ToQ(card.id);
      }
    }
    return graph.nodes.empty() ? QString() : ToQ(graph.nodes[0].id);
  };
  auto shared_edge = std::make_shared<bridge::EdgeView>();
  Add(QStringLiteral("start-screen"), [] {});
  Add(QStringLiteral("open-example"), [window] {
    for (const auto& entry : App::Get().session().examples()) {
      if (ToQ(entry.dir).endsWith(QStringLiteral("todomvc"))) {
        window->OpenExample(ToQ(entry.dir));
        return;
      }
    }
  }, 900);
  Add(QStringLiteral("select-card"), [canvas, first_card] {
    QTest::mouseClick(canvas->viewport(), Qt::LeftButton, {},
                      canvas->ViewPointOf(first_card()));
  });
  Add(QStringLiteral("detail-at-100"), [canvas, first_card] {
    canvas->resetTransform();
    const QPoint at = canvas->ViewPointOf(first_card());
    canvas->centerOn(canvas->mapToScene(at) + QPointF(260, 120));
    emit canvas->ZoomChanged(canvas->zoom());
  });
  Add(QStringLiteral("drag-card"), [this, canvas, first_card] {
    const QPoint from = canvas->ViewPointOf(first_card());
    Drag(canvas->viewport(), from, from + QPoint(120, 90));
  }, 600);
  Add(QStringLiteral("undo-drag"), [] { App::Get().session().undo(); }, 600);
  Add(QStringLiteral("remove-edge"), [shared_edge] {
    const bridge::GraphView graph = App::Get().session().graph();
    if (graph.edges.empty()) return;
    *shared_edge = graph.edges[0];
    rust::Vec<rust::String> edges;
    edges.push_back(shared_edge->id);
    App::Get().session().remove({}, std::move(edges));
  }, 600);
  Add(QStringLiteral("reconnect-by-drag"), [this, canvas, shared_edge] {
    if (shared_edge->id.empty()) return;
    const QPoint from = canvas->ViewPointOfPort(ToQ(shared_edge->source),
                                                ToQ(shared_edge->source_handle), true);
    const QPoint to = canvas->ViewPointOfPort(ToQ(shared_edge->target),
                                              ToQ(shared_edge->target_handle), false);
    Drag(canvas->viewport(), from, to, 20);
  }, 600);
  Add(QStringLiteral("refused-connection"), [this, canvas] {
    bridge::Session& session = App::Get().session();
    const bridge::GraphView graph = session.graph();
    for (const auto& source : graph.nodes) {
      for (const auto& out : source.outputs) {
        for (const auto& target : graph.nodes) {
          if (target.id == source.id) continue;
          for (const auto& in : target.inputs) {
            const rust::String problem =
                session.connection_problem(source.id, out.id, target.id, in.id);
            if (problem.empty()) continue;
            Drag(canvas->viewport(),
                 canvas->ViewPointOfPort(ToQ(source.id), ToQ(out.id), true),
                 canvas->ViewPointOfPort(ToQ(target.id), ToQ(in.id), false), 16);
            return;
          }
        }
      }
    }
  }, 500);
  Add(QStringLiteral("edit-label"), [window] {
    auto* inspector = window->inspector_dock()->widget();
    const QList<QLineEdit*> lines = inspector->findChildren<QLineEdit*>();
    if (lines.isEmpty()) return;
    QLineEdit* name = lines.first();
    name->setFocus();
    name->selectAll();
    QTest::keyClicks(name, QStringLiteral("Open the shop"));
    QTest::keyClick(name, Qt::Key_Return);
    name->clearFocus();
  }, 700);
  Add(QStringLiteral("edit-pause"), [window] {
    for (QLineEdit* line :
         window->inspector_dock()->widget()->findChildren<QLineEdit*>()) {
      if (line->validator() == nullptr) continue;
      line->setFocus();
      line->selectAll();
      QTest::keyClicks(line, QStringLiteral("2.5"));
      QTest::keyClick(line, Qt::Key_Return);
      line->clearFocus();
      return;
    }
  }, 700);
  Add(QStringLiteral("zoom-out"), [canvas] {
    canvas->ZoomBy(0.6, canvas->viewport()->rect().center());
  });
  Add(QStringLiteral("pop-out-inspector"), [window] {
    window->inspector_dock()->setFloating(true);
  }, 900);
  Add(QStringLiteral("dock-back-inspector"), [window] {
    window->inspector_dock()->setFloating(false);
  }, 900);
  Add(QStringLiteral("toggle-nodes-panel"), [window] {
    window->activateWindow();
    const bool active = QTest::qWaitForWindowActive(window, 2000);
    App::Get().Log(QStringLiteral("main window active: %1, nodes open before: %2")
                       .arg(active)
                       .arg(window->nodes_dock()->isOpen()));
    QTest::keyClick(window, Qt::Key_B, Qt::ControlModifier);
    App::Get().Log(QStringLiteral("nodes open after Ctrl+B: %1")
                       .arg(window->nodes_dock()->isOpen()));
  });
  Add(QStringLiteral("restore-nodes-panel"), [window] {
    QTest::keyClick(window, Qt::Key_B, Qt::ControlModifier);
  });
  Add(QStringLiteral("settings-general"), [window] { window->OpenSettings(); }, 700);
  Add(QStringLiteral("settings-projects"), [window] { window->OpenSettings()->ShowPage(1); });
  Add(QStringLiteral("settings-editor-touchpad"), [window] {
    SettingsWindow* settings = window->OpenSettings();
    settings->ShowPage(2);
    for (QPushButton* button : settings->findChildren<QPushButton*>()) {
      if (button->isVisible() && button->text() == QStringLiteral("Touchpad")) {
        QTest::mouseClick(button, Qt::LeftButton);
      }
    }
  });
  Add(QStringLiteral("settings-api-switch"), [window] {
    SettingsWindow* settings = window->OpenSettings();
    settings->ShowPage(3);
    for (ToggleSwitch* toggle : settings->findChildren<ToggleSwitch*>()) {
      if (toggle->isVisible()) QTest::mouseClick(toggle, Qt::LeftButton);
    }
  }, 600);
  Add(QStringLiteral("settings-about"), [window] {
    window->OpenSettings()->ShowPage(4);
  });
  Add(QStringLiteral("close-settings"), [window] { window->OpenSettings()->close(); });
  Add(QStringLiteral("close-project"), [window] { window->CloseProject(); }, 800);
}

void Scenario::BuildBench(int count) {
  MainWindow* window = window_;
  GraphCanvas* canvas = window->canvas();
  Add(QStringLiteral("generate"), [window, count] {
    QElapsedTimer timer;
    timer.start();
    window->Generate(count);
    App::Get().Log(QStringLiteral("generated %1 nodes in %2 ms, rss %3")
                       .arg(count)
                       .arg(timer.elapsed())
                       .arg(Rss()));
  }, 1500);
  Add(QStringLiteral("fit"), [canvas] {
    canvas->FitAll();
    App::Get().Log(QStringLiteral("zoomed out: ") + canvas->Measure(120));
  }, 600);
  Add(QStringLiteral("detail"), [canvas] {
    canvas->resetTransform();
    canvas->centerOn(800, 500);
    App::Get().Log(QStringLiteral("at 100%: ") + canvas->Measure(120));
  }, 600);
  Add(QStringLiteral("drag-one"), [this, canvas] {
    const bridge::GraphView graph = App::Get().session().graph();
    const QString id = ToQ(graph.nodes[0].id);
    canvas->centerOn(graph.nodes[0].x + 128, graph.nodes[0].y + 60);
    QElapsedTimer timer;
    timer.start();
    const QPoint from = canvas->ViewPointOf(id);
    Drag(canvas->viewport(), from, from + QPoint(60, 40), 30);
    App::Get().Log(QStringLiteral("drag of 30 moves plus commit: %1 ms").arg(timer.elapsed()));
  }, 900);
  Add(QStringLiteral("undo"), [] {
    QElapsedTimer timer;
    timer.start();
    App::Get().session().undo();
    App::Get().Log(QStringLiteral("undo call: %1 ms, rss %2").arg(timer.elapsed()).arg(Rss()));
  }, 900);
}

}
