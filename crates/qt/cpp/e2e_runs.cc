#include <QApplication>
#include <QElapsedTimer>
#include <QJsonDocument>
#include <QJsonObject>
#include <QPointer>
#include <QTimer>
#include <QtTest/QTest>
#include <memory>

#include "app.h"
#include "inspector_panel.h"
#include "main_window.h"
#include "opsis-qt/src/bridge.rs.h"
#include "scenario.h"
#include "stream_window.h"

namespace opsis {

namespace {

constexpr int kEngineMs = 60'000;
constexpr int kViewMs = 30'000;
constexpr int kInspectorMs = 15'000;
constexpr int kHoverMs = 600;
constexpr int kRetryMs = 2000;
constexpr double kButtonX = 200;
constexpr double kButtonY = 130;

QString Expect(bool ok, const QString& what) { return ok ? QString() : what; }

StreamWindow* OpenView() {
  for (QWidget* top : QApplication::topLevelWidgets()) {
    if (auto* window = qobject_cast<StreamWindow*>(top);
        window != nullptr && window->isVisible()) {
      return window;
    }
  }
  return nullptr;
}

QString FieldText(const QString& key) {
  const bridge::InspectorView view = App::Get().session().inspector();
  for (const auto& field : view.fields) {
    if (ToQ(field.key) == key) return ToQ(field.text);
  }
  return {};
}

}

void Scenario::BuildRuns() {
  const QString projects = qEnvironmentVariable("OPSIS_QT_PROJECTS");
  const QString page = qEnvironmentVariable("OPSIS_QT_PAGE");
  Until(
      QStringLiteral("engine-ready"), [] {}, kEngineMs,
      [] { return !App::Get().session().engine_missing(); },
      [] {
        return Expect(!App::Get().session().engine_missing(),
                      QStringLiteral("the seeded recording browser isn't "
                                     "usable"));
      });
  Until(
      QStringLiteral("open-todomvc-on-the-test-page"),
      [projects, page] {
        auto& session = App::Get().session();
        session.open_project(ToRust(projects + QStringLiteral("/todomvc")));
        const QJsonObject patch{{QStringLiteral("url"), page}};
        session.set_params(
            ToRust(QStringLiteral("open")),
            ToRust(QString::fromUtf8(
                QJsonDocument(patch).toJson(QJsonDocument::Compact))));
        rust::Vec<rust::String> ids;
        ids.push_back(ToRust(QStringLiteral("complete")));
        session.select(std::move(ids));
      },
      kInspectorMs,
      [this] { return Find(window_, QStringLiteral("Pick")) != nullptr; },
      [this, page] {
        return Expect(!page.isEmpty() &&
                          Find(window_, QStringLiteral("Pick")) != nullptr,
                      QStringLiteral("the click action shows no Pick button"));
      });
  Until(
      QStringLiteral("pick-opens-a-streamed-view"),
      [this] { Click(window_, QStringLiteral("Pick")); }, kViewMs,
      [] {
        StreamWindow* view = OpenView();
        return view != nullptr && view->canvas()->HasPicture();
      },
      [] {
        StreamWindow* view = OpenView();
        if (view == nullptr) return QStringLiteral("no streamed view opened");
        const QSize page = view->canvas()->page();
        return Expect(view->canvas()->HasPicture() && page.width() > kButtonX &&
                          page.height() > kButtonY,
                      QStringLiteral("the streamed page is %1×%2")
                          .arg(page.width())
                          .arg(page.height()));
      });
  auto clicked = std::make_shared<QElapsedTimer>();
  Until(
      QStringLiteral("pick-the-button-on-the-page"),
      [clicked] { clicked->invalidate(); }, kViewMs,
      [clicked] {
        if (FieldText(QStringLiteral("target")).contains("Save draft")) {
          return true;
        }
        StreamWindow* view = OpenView();
        if (view == nullptr ||
            (clicked->isValid() && clicked->elapsed() < kRetryMs)) {
          return false;
        }
        clicked->start();
        QPointer<StreamCanvas> canvas = view->canvas();
        const QPoint at =
            canvas->FromPage(QPointF(kButtonX, kButtonY)).toPoint();
        QTest::mouseMove(canvas, at);
        QTimer::singleShot(kHoverMs, canvas, [canvas, at] {
          if (canvas.isNull()) return;
          QTest::mouseClick(canvas, Qt::LeftButton, {}, at);
        });
        return false;
      },
      [] {
        return Expect(
            FieldText(QStringLiteral("target")).contains("Save draft") &&
                OpenView() == nullptr,
            QStringLiteral("the pick gave '%1'")
                .arg(FieldText(QStringLiteral("target"))));
      });
}

}
