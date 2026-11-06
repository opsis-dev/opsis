#include "scenario.h"

#include <QAbstractButton>
#include <QAction>
#include <QApplication>
#include <QDir>
#include <QElapsedTimer>
#include <QFile>
#include <QImage>
#include <QLabel>
#include <QLineEdit>
#include <QScrollArea>
#include <QTextCursor>
#include <QTimer>
#include <QToolButton>
#include <QUrl>
#include <QtTest/QTest>
#include <algorithm>
#include <cmath>
#include <memory>

#include "app.h"
#include "graph_canvas.h"
#include "inspector_panel.h"
#include "main_window.h"
#include "node_list.h"
#include "opsis-qt/src/bridge.rs.h"
#include "settings_window.h"
#include "top_bar.h"
#include "var_text.h"
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
    dir = QDir::homePath() + QStringLiteral("/.cache/opsis-qt-shots/") + name;
  }
  QDir shots(dir);
  if (dir != QStringLiteral("off")) shots.mkpath(QStringLiteral("."));
  new Scenario(window, name, shots);
}

Scenario::Scenario(MainWindow* window, const QString& name, const QDir& shots)
    : QObject(window), window_(window), name_(name), shots_(shots) {
  shooting_ = qEnvironmentVariable("OPSIS_QT_SHOTS") != QStringLiteral("off");
  if (name == QStringLiteral("bench")) {
    const int count = qEnvironmentVariableIntValue("OPSIS_QT_BENCH_NODES");
    BuildBench(count > 0 ? count : 200);
  } else {
    BuildTour();
  }
  QTimer::singleShot(1200, this, &Scenario::Next);
}

void Scenario::Add(const QString& name, std::function<void()> action,
                   int wait_ms, std::function<QString()> check) {
  steps_.push_back(Step{name, std::move(action), wait_ms, std::move(check)});
}

void Scenario::Next() {
  if (index_ >= steps_.size()) {
    App::Get().Log(QStringLiteral("scenario %1 done, rss %2, checks %3 "
                                  "passed, %4 failed")
                       .arg(name_, Rss())
                       .arg(checked_ - failed_)
                       .arg(failed_));
    QCoreApplication::exit(failed_ == 0 ? 0 : 1);
    return;
  }
  const Step& step = steps_.at(index_);
  App::Get().Log(QStringLiteral("step %1: %2").arg(index_ + 1).arg(step.name));
  step.action();
  const int wait = step.wait_ms;
  const QString name = step.name;
  const std::function<QString()> check = step.check;
  const std::size_t number = ++index_;
  QTimer::singleShot(wait, this, [this, name, number, check] {
    if (check || !misses_.isEmpty() || !Overflow().isEmpty()) {
      QStringList found = misses_;
      misses_.clear();
      const QString overflow = Overflow();
      if (!overflow.isEmpty()) found << overflow;
      if (check) {
        const QString own = check();
        if (!own.isEmpty()) found << own;
      }
      const QString problem = found.join(QStringLiteral("; "));
      ++checked_;
      if (problem.isEmpty()) {
        App::Get().Log(QStringLiteral("check %1: ok").arg(name));
      } else {
        ++failed_;
        App::Get().Log(
            QStringLiteral("check %1: FAILED %2").arg(name, problem));
      }
    }
    Shoot(
        QStringLiteral("%1-%2").arg(number, 2, 10, QLatin1Char('0')).arg(name));
    Next();
  });
}

QString Scenario::Overflow() {
  QStringList wide;
  for (QWidget* top : QApplication::topLevelWidgets()) {
    if (!top->isVisible()) continue;
    for (QScrollArea* scroll : top->findChildren<QScrollArea*>()) {
      if (!scroll->isVisible() || scroll->widget() == nullptr ||
          scroll->horizontalScrollBarPolicy() != Qt::ScrollBarAlwaysOff) {
        continue;
      }
      const int over = scroll->widget()->width() - scroll->viewport()->width();
      if (over > 1) {
        QWidget* widest = nullptr;
        for (QWidget* child : scroll->widget()->findChildren<QWidget*>()) {
          if (!child->isVisible() || child->layout() != nullptr) continue;
          if (widest == nullptr || child->minimumSizeHint().width() >
                                       widest->minimumSizeHint().width()) {
            widest = child;
          }
        }
        QString name;
        if (widest != nullptr) {
          name = QString::fromLatin1(widest->metaObject()->className());
          QString text = widest->accessibleName();
          if (auto* label = qobject_cast<QLabel*>(widest)) text = label->text();
          if (auto* button = qobject_cast<QAbstractButton*>(widest)) {
            text = button->text();
          }
          if (!text.isEmpty())
            name += QStringLiteral(" '%1'").arg(text.left(40));
          name +=
              QStringLiteral(" (%1 px)").arg(widest->minimumSizeHint().width());
        }
        wide << QStringLiteral(
                    "the panel is %1 px wider than its dock, widest %2")
                    .arg(over)
                    .arg(name);
      }
    }
  }
  return wide.join(QStringLiteral("; "));
}

void Scenario::Shoot(const QString& name) {
  if (!shooting_) return;
  int count = 0;
  QWidgetList windows = QApplication::topLevelWidgets();
  std::stable_partition(windows.begin(), windows.end(), [](QWidget* widget) {
    return widget->objectName() == QStringLiteral("Window");
  });
  for (QWidget* widget : windows) {
    if (!widget->isVisible() || !widget->isWindow()) continue;
    const bool popover = widget->objectName() == QStringLiteral("Popover");
    const bool card = widget->objectName() == QStringLiteral("HoverCard");
    if ((widget->windowType() == Qt::ToolTip && !card) ||
        (widget->windowType() == Qt::Popup && !popover))
      continue;
    if (widget->width() < 40 || widget->height() < 40) continue;
    const QString suffix =
        count == 0 ? QString()
                   : QStringLiteral("-%1").arg(widget->objectName().isEmpty()
                                                   ? QString::number(count)
                                                   : widget->objectName());
    widget->grab().save(
        shots_.filePath(name + suffix + QStringLiteral(".png")));
    ++count;
  }
}

void Scenario::Drag(QWidget* target, QPoint from, QPoint to, int steps) {
  QTest::mousePress(target, Qt::LeftButton, {}, from);
  for (int i = 1; i <= steps; ++i) {
    const QPoint at = from + (to - from) * i / steps;
    QMouseEvent move(QEvent::MouseMove, QPointF(at),
                     target->mapToGlobal(QPointF(at)), Qt::NoButton,
                     Qt::LeftButton, Qt::NoModifier);
    QApplication::sendEvent(target, &move);
  }
  QTest::mouseRelease(target, Qt::LeftButton, {}, to);
}

void Scenario::BuildBench(int count) {
  MainWindow* window = window_;
  GraphCanvas* canvas = window->canvas();
  Add(
      QStringLiteral("generate"),
      [window, count] {
        QElapsedTimer timer;
        timer.start();
        window->Generate(count);
        App::Get().Log(QStringLiteral("generated %1 nodes in %2 ms, rss %3")
                           .arg(count)
                           .arg(timer.elapsed())
                           .arg(Rss()));
      },
      1500);
  Add(
      QStringLiteral("fit"),
      [canvas] {
        canvas->FitAll();
        App::Get().Log(QStringLiteral("zoomed out: ") + canvas->Measure(120));
      },
      600);
  Add(
      QStringLiteral("detail"),
      [canvas] {
        canvas->resetTransform();
        canvas->centerOn(800, 500);
        App::Get().Log(QStringLiteral("at 100%: ") + canvas->Measure(120));
      },
      600);
  Add(
      QStringLiteral("drag-one"),
      [this, canvas] {
        const bridge::GraphView graph = App::Get().session().graph();
        const bridge::CardView& first = graph.nodes.front();
        const QString id = ToQ(first.id);
        canvas->centerOn(first.x + 128, first.y + 60);
        QElapsedTimer timer;
        timer.start();
        const QPoint from = canvas->ViewPointOf(id);
        Drag(canvas->viewport(), from, from + QPoint(60, 40), 30);
        App::Get().Log(QStringLiteral("drag of 30 moves plus commit: %1 ms")
                           .arg(timer.elapsed()));
      },
      900);
  Add(
      QStringLiteral("undo"),
      [] {
        QElapsedTimer timer;
        timer.start();
        App::Get().session().undo();
        App::Get().Log(QStringLiteral("undo call: %1 ms, rss %2")
                           .arg(timer.elapsed())
                           .arg(Rss()));
      },
      900);
}
}
