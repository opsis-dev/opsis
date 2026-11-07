#include <QApplication>
#include <QElapsedTimer>
#include <QFileDialog>
#include <QJsonDocument>
#include <QJsonObject>
#include <QLabel>
#include <QLineEdit>
#include <QListWidget>
#include <QPointer>
#include <QTimer>
#include <QtTest/QTest>
#include <memory>

#include "app.h"
#include "file_player.h"
#include "inspector_panel.h"
#include "live_view.h"
#include "main_window.h"
#include "opsis-qt/src/bridge.rs.h"
#include "replay_dialog.h"
#include "scenario.h"
#include "settings_window.h"
#include "start_screen.h"
#include "stream_window.h"

namespace opsis {

namespace {

constexpr int kEngineMs = 60'000;
constexpr int kViewMs = 30'000;
constexpr int kInspectorMs = 15'000;
constexpr int kHoverMs = 600;
constexpr int kRetryMs = 2000;
constexpr int kLiveMs = 90'000;
constexpr int kRunMs = 240'000;
constexpr int kDialogMs = 20'000;
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

QWidget* Dialog(const QString& name) {
  for (QWidget* top : QApplication::topLevelWidgets()) {
    if (top->isVisible() && top->objectName() == name) return top;
    if (auto* found = top->findChild<QWidget*>(name);
        found != nullptr && found->isVisible()) {
      return found;
    }
  }
  return nullptr;
}

bool ReplayPlays() {
  QWidget* dialog = Dialog(QStringLiteral("ReplayDialog"));
  auto* player = dialog == nullptr ? nullptr : dialog->findChild<FilePlayer*>();
  if (player == nullptr) return false;
  const bridge::PlayerView view =
      App::Get().session().player_view(player->id());
  return view.ready && view.has_video && view.serial > 0;
}

int ListCount(const QString& dialog_name, const QString& list_name) {
  QWidget* dialog = Dialog(dialog_name);
  auto* list =
      dialog == nullptr ? nullptr : dialog->findChild<QListWidget*>(list_name);
  return list == nullptr ? -1 : list->count();
}

void CloseDialog(const QString& name) {
  if (QWidget* dialog = Dialog(name)) dialog->close();
}

SettingsWindow* OpenSettingsWindow() {
  for (QWidget* top : QApplication::topLevelWidgets()) {
    if (auto* found = qobject_cast<SettingsWindow*>(top);
        found != nullptr && found->isVisible()) {
      return found;
    }
  }
  return nullptr;
}

QString EngineState(const QWidget* root) {
  auto* state = root == nullptr
                    ? nullptr
                    : root->findChild<QLabel*>(QStringLiteral("EngineState"));
  return state != nullptr && state->isVisible() ? state->text() : QString();
}

}

void Scenario::BuildRuns() {
  const QString projects = qEnvironmentVariable("OPSIS_QT_PROJECTS");
  const QString page = qEnvironmentVariable("OPSIS_QT_PAGE");
  const QString browser = qEnvironmentVariable("OPSIS_QT_BROWSER");
  Add(
      QStringLiteral("setup-shows-without-an-engine"), [] {}, 300,
      [this] {
        auto* setup =
            window_->findChild<QWidget*>(QStringLiteral("SetupScreen"));
        return Expect(setup != nullptr && setup->isVisible() &&
                          EngineState(setup) == QStringLiteral("Not set up"),
                      QStringLiteral("the setup screen didn't show"));
      });
  Add(
      QStringLiteral("skip-setup"),
      [this] { Click(window_, QStringLiteral("Skip for now")); }, 600,
      [this] {
        return Expect(window_->start_screen()->isVisible() &&
                          App::Get().session().settings().setup_skipped,
                      QStringLiteral("skipping didn't reach the start screen"));
      });
  Add(
      QStringLiteral("settings-offers-the-download"),
      [this] { window_->OpenSettings()->ShowPage(SettingsWindow::kEngine); },
      800,
      [this] {
        SettingsWindow* settings = OpenSettingsWindow();
        return Expect(
            EngineState(settings) == QStringLiteral("Not set up") &&
                Find(settings, QStringLiteral("Download engine")) != nullptr,
            QStringLiteral("the Engine page doesn't offer the download"));
      });
  Add(
      QStringLiteral("settings-choose-the-browser-asks-for-a-file"),
      [this] { Click(OpenSettingsWindow(), QStringLiteral("Choose file…")); },
      900,
      [] {
        return Expect(qobject_cast<QFileDialog*>(
                          QApplication::activeModalWidget()) != nullptr,
                      QStringLiteral("no file dialog opened"));
      });
  Until(
      QStringLiteral("settings-pick-the-browser"),
      [browser] {
        auto* dialog =
            qobject_cast<QFileDialog*>(QApplication::activeModalWidget());
        if (dialog == nullptr) return;
        auto* name =
            dialog->findChild<QLineEdit*>(QStringLiteral("fileNameEdit"));
        if (browser.isEmpty() || name == nullptr) {
          dialog->reject();
          return;
        }
        name->setFocus();
        QTest::keyClicks(name, browser);
        QTest::keyClick(name, Qt::Key_Return);
      },
      kEngineMs, [] { return !App::Get().session().engine_missing(); },
      [] {
        return Expect(!App::Get().session().engine_missing(),
                      QStringLiteral("the chosen recording browser isn't "
                                     "usable"));
      });
  Add(
      QStringLiteral("settings-shows-the-engine"),
      [this] { window_->OpenSettings()->ShowPage(SettingsWindow::kEngine); },
      800,
      [] {
        SettingsWindow* settings = nullptr;
        for (QWidget* top : QApplication::topLevelWidgets()) {
          if (auto* found = qobject_cast<SettingsWindow*>(top))
            settings = found;
        }
        auto* state =
            settings == nullptr
                ? nullptr
                : settings->findChild<QLabel*>(QStringLiteral("EngineState"));
        const bridge::EngineView engine = App::Get().session().engine_view();
        const bool shown = state != nullptr && state->isVisible() &&
                           state->text() == QStringLiteral("Set up") &&
                           !engine.path.empty();
        if (settings != nullptr) settings->close();
        return Expect(shown, QStringLiteral("the Engine page doesn't show the "
                                            "engine as set up"));
      });
  BuildPorts();
  Until(
      QStringLiteral("open-todomvc-on-the-test-page"),
      [projects, page] {
        auto& session = App::Get().session();
        session.open_project(ToRust(projects + QStringLiteral("/todomvc")));
        const QJsonObject patch{{QStringLiteral("url"), page}};
        session.set_params(ToRust(QStringLiteral("open")),
                           ToRust(QString::fromUtf8(QJsonDocument(patch).toJson(
                               QJsonDocument::Compact))));
        rust::Vec<rust::String> ids;
        ids.push_back(ToRust(QStringLiteral("complete")));
        session.select(std::move(ids));
      },
      kInspectorMs,
      [this] { return Find(window_, QStringLiteral("Pick")) != nullptr; },
      [this, page] {
        return Expect(
            !page.isEmpty() && Find(window_, QStringLiteral("Pick")) != nullptr,
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

  Until(
      QStringLiteral("live-view-shows-the-recording"),
      [] {
        App::Get().session().run_node(ToRust(QStringLiteral("record")), false);
      },
      kLiveMs,
      [this] {
        LivePip* pip = window_->live_pip();
        return pip != nullptr && pip->isVisible() &&
               pip->surface()->HasPicture();
      },
      [this] {
        LivePip* pip = window_->live_pip();
        const QWidget* surface = pip == nullptr ? nullptr : pip->surface();
        return Expect(pip != nullptr && pip->isVisible() &&
                          pip->surface()->HasPicture(),
                      QStringLiteral("the live view never showed the "
                                     "recording")) +
               Expect(
                   surface != nullptr &&
                       std::abs(surface->height() -
                                surface->heightForWidth(surface->width())) <= 2,
                   QStringLiteral(" the picture is %1 px tall, not its "
                                  "shape's %2")
                       .arg(surface == nullptr ? 0 : surface->height())
                       .arg(surface == nullptr
                                ? 0
                                : surface->heightForWidth(surface->width())));
      });
  Until(
      QStringLiteral("live-window-opens-from-the-picture"),
      [this] {
        LivePip* pip = window_->live_pip();
        if (pip == nullptr) return;
        QTest::mouseClick(pip->surface(), Qt::LeftButton, {},
                          pip->surface()->rect().center());
      },
      kDialogMs, [] { return LiveWindow::IsOpen(); },
      [this] {
        return Expect(LiveWindow::IsOpen() && !window_->live_pip()->isVisible(),
                      QStringLiteral("the Live window didn't take over from "
                                     "the picture in picture"));
      });
  Until(
      QStringLiteral("record-run-finishes"),
      [] {
        if (QWidget* live = Dialog(QStringLiteral("LiveWindow"))) live->close();
      },
      kRunMs, [] { return !App::Get().session().bar().running; },
      [] {
        const bridge::RunPanelView view = App::Get().session().run_panel();
        const bridge::ReplayView replay = App::Get().session().replay(
            ToRust(QStringLiteral("record")), ToRust(QStringLiteral("replay")));
        return Expect(
            !view.running && view.failure.empty() && replay.can_replay &&
                !replay.video_file.empty(),
            QStringLiteral("the take failed: %1").arg(ToQ(view.failure)));
      });
  Until(
      QStringLiteral("replay-plays-the-take"),
      [] {
        App::Get().session().show_replay(ToRust(QStringLiteral("record")),
                                         ToRust(QStringLiteral("replay")));
      },
      kDialogMs, [] { return ReplayPlays(); },
      [] {
        const int actions = ListCount(QStringLiteral("ReplayDialog"),
                                      QStringLiteral("ReplayActions"));
        return Expect(ReplayPlays() && actions > 0,
                      QStringLiteral("the Replay didn't play the take with its "
                                     "actions (%1 actions)")
                          .arg(actions));
      });
  Until(
      QStringLiteral("trace-lists-the-actions"),
      [] {
        CloseDialog(QStringLiteral("ReplayDialog"));
        App::Get().session().show_replay(ToRust(QStringLiteral("record")),
                                         ToRust(QStringLiteral("trace")));
      },
      kDialogMs,
      [] {
        return ListCount(QStringLiteral("ReplayDialog"),
                         QStringLiteral("TraceActions")) > 0;
      },
      [] {
        const int actions = ListCount(QStringLiteral("ReplayDialog"),
                                      QStringLiteral("TraceActions"));
        QWidget* dialog = Dialog(QStringLiteral("ReplayDialog"));
        auto* shot =
            dialog == nullptr
                ? nullptr
                : dialog->findChild<TraceShot*>(QStringLiteral("TraceShot"));
        CloseDialog(QStringLiteral("ReplayDialog"));
        return Expect(actions > 0 && shot != nullptr && shot->HasPicture(),
                      QStringLiteral("the trace shows %1 actions and %2")
                          .arg(actions)
                          .arg(shot != nullptr && shot->HasPicture()
                                   ? QStringLiteral("a screenshot")
                                   : QStringLiteral("no screenshot")));
      });
  BuildTerminalRuns();
}

}
