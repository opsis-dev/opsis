#include <QApplication>
#include <QFileInfo>
#include <QLabel>
#include <QMenu>
#include <QPointer>
#include <QToolButton>
#include <QVBoxLayout>
#include <QtTest/QTest>
#include <cmath>
#include <memory>

#include "app.h"
#include "file_player.h"
#include "opsis-qt/src/bridge.rs.h"
#include "scenario.h"

namespace opsis {

namespace {

constexpr int kHostWidth = 420;
constexpr int kHostHeight = 380;
constexpr int kNarrowWidth = 240;
constexpr double kClipSeconds = 3;
constexpr double kSeekShare = 0.75;

QString Expect(bool ok, const QString& what) { return ok ? QString() : what; }

struct PlayerTest {
  QPointer<QWidget> host;
  QPointer<FilePlayer> player;
  std::uint64_t serial = 0;
  std::uint64_t id = 0;

  bridge::PlayerView View() const {
    return App::Get().session().player_view(player.isNull() ? 0 : player->id());
  }

  void Show(const QString& file, bool video) {
    delete host;
    auto* window = new QWidget(nullptr);
    window->setObjectName(QStringLiteral("PlayerTest"));
    window->resize(kHostWidth, kHostHeight);
    auto* layout = new QVBoxLayout(window);
    player = new FilePlayer(file, QFileInfo(file).fileName(), video, window);
    id = player->id();
    layout->addWidget(player);
    layout->addStretch(1);
    window->show();
    host = window;
  }
};

}

void Scenario::BuildPlayer() {
  const QString fixtures = qEnvironmentVariable("OPSIS_QT_FIXTURES");
  auto test = std::make_shared<PlayerTest>();
  const QString clip = fixtures + QStringLiteral("/clip.mp4");
  Add(
      QStringLiteral("player-video-ready"),
      [test, clip] { test->Show(clip, true); }, 2500,
      [test, fixtures] {
        if (fixtures.isEmpty()) return QStringLiteral("no test clip was made");
        const bridge::PlayerView view = test->View();
        return Expect(view.ready && view.has_video && view.has_audio &&
                          std::abs(view.duration - kClipSeconds) < 0.2 &&
                          view.serial > 0 && view.width == 320,
                      QStringLiteral("the clip didn't open with a picture"));
      });
  Add(
      QStringLiteral("player-sound-decoded"), [] {}, 3000,
      [test] {
        return Expect(test->View().sound_ready,
                      QStringLiteral("the clip's sound wasn't decoded"));
      });
  Add(
      QStringLiteral("player-play"),
      [this, test] { Click(test->host, QStringLiteral("Play")); }, 1500,
      [test] {
        const bridge::PlayerView view = test->View();
        test->serial = view.serial;
        return Expect(view.playing && view.position > 0.6,
                      QStringLiteral("the clip didn't play"));
      });
  Add(
      QStringLiteral("player-pause"),
      [this, test] { Click(test->host, QStringLiteral("Pause")); }, 500,
      [test] {
        const bridge::PlayerView view = test->View();
        return Expect(!view.playing && view.serial > 1,
                      QStringLiteral("the clip didn't pause after showing "
                                     "new pictures"));
      });
  Add(
      QStringLiteral("player-paused-shares-its-frame"), [] {}, 200,
      [test] {
        const auto* surface = test->host->findChild<PlayerSurface*>();
        const rust::Box<bridge::FrameRef> frame =
            App::Get().session().player_frame(test->id);
        const bool shared = surface != nullptr && frame->width() > 0 &&
                            surface->FrameBits() == frame->bytes().data();
        return Expect(shared && !test->player->Ticking(),
                      QStringLiteral("the paused player copied its picture "
                                     "or kept ticking"));
      });
  Add(
      QStringLiteral("player-seek"),
      [this, test] {
        auto* seek = test->host->findChild<SeekBar*>();
        test->serial = test->View().serial;
        if (seek == nullptr) {
          Miss(QStringLiteral("no seek bar"));
          return;
        }
        ClickAt(seek, QPoint(static_cast<int>(seek->width() * kSeekShare),
                             seek->height() / 2));
      },
      1500,
      [test] {
        const bridge::PlayerView view = test->View();
        return Expect(view.position > 2.0 && view.position < 2.5 &&
                          view.serial != test->serial,
                      QStringLiteral("seeking didn't move the clip and its "
                                     "picture (at %1)")
                          .arg(view.position));
      });
  Add(QStringLiteral("player-speed-menu"), [this, test] {
    ClickLater(test->host, QStringLiteral("Playback speed"));
  });
  Add(
      QStringLiteral("player-speed"),
      [this] {
        auto* menu = qobject_cast<QMenu*>(QApplication::activePopupWidget());
        if (menu == nullptr) {
          Miss(QStringLiteral("no speed menu"));
          return;
        }
        for (QAction* action : menu->actions()) {
          if (action->text() == QStringLiteral("2×")) {
            QTest::mouseClick(menu, Qt::LeftButton, {},
                              menu->actionGeometry(action).center());
            return;
          }
        }
        menu->close();
        Miss(QStringLiteral("no 2× in the speed menu"));
      },
      450,
      [test] {
        auto* speed =
            test->host->findChild<QToolButton*>(QStringLiteral("PlayerSpeed"));
        return Expect(speed != nullptr && speed->text() == QStringLiteral("2×"),
                      QStringLiteral("the speed didn't change"));
      });
  Add(
      QStringLiteral("player-plays-to-the-end"),
      [this, test] { Click(test->host, QStringLiteral("Play")); }, 1500,
      [test] {
        const bridge::PlayerView view = test->View();
        return Expect(
            !view.playing && std::abs(view.position - view.duration) < 0.05,
            QStringLiteral("the clip didn't stop at its end (at %1)")
                .arg(view.position));
      });
  Add(
      QStringLiteral("player-mute"),
      [this, test] { Click(test->host, QStringLiteral("Mute")); }, 300,
      [this, test] {
        return Expect(Find(test->host, QStringLiteral("Unmute")) != nullptr,
                      QStringLiteral("the clip didn't mute"));
      });
  Add(
      QStringLiteral("player-full-screen"),
      [this, test] { Click(test->host, QStringLiteral("Full screen")); }, 800,
      [test] {
        return Expect(!test->player.isNull() && test->player->isFullScreen(),
                      QStringLiteral("the clip didn't go full screen"));
      });
  Add(
      QStringLiteral("player-leave-full-screen"),
      [test] { QTest::keyClick(test->player, Qt::Key_Escape); }, 600,
      [test] {
        return Expect(!test->player.isNull() && !test->player->isWindow() &&
                          test->player->parentWidget() == test->host.data() &&
                          test->player->isVisible(),
                      QStringLiteral("the clip didn't return to its place"));
      });
  Add(
      QStringLiteral("player-narrow"),
      [test] { test->host->resize(kNarrowWidth, kHostHeight); }, 300,
      [test] {
        return Expect(test->player->minimumSizeHint().width() <= kNarrowWidth,
                      QStringLiteral("the player is %1 px wide at least")
                          .arg(test->player->minimumSizeHint().width()));
      });
  Add(
      QStringLiteral("player-audio"),
      [test] {
        test->Show(ToQ(App::Get().session().file_path(
                       ToRust(QStringLiteral("app:music/calm-piano.m4a")))),
                   false);
      },
      3000,
      [test] {
        const bridge::PlayerView view = test->View();
        const auto* surface = test->host->findChild<PlayerSurface*>();
        return Expect(view.ready && view.has_audio && !view.has_video &&
                          view.sound_ready && view.duration > 1 &&
                          surface == nullptr,
                      QStringLiteral("the music didn't open as sound only"));
      });
  Add(
      QStringLiteral("player-audio-play"),
      [this, test] { Click(test->host, QStringLiteral("Play")); }, 1200,
      [test] {
        const bridge::PlayerView view = test->View();
        return Expect(view.playing && view.position > 0.5,
                      QStringLiteral("the music didn't play"));
      });
  Add(
      QStringLiteral("player-missing-file"),
      [test, fixtures] {
        test->Show(fixtures + QStringLiteral("/gone.mp4"), true);
      },
      1200,
      [test] {
        const auto* error =
            test->host->findChild<QWidget*>(QStringLiteral("PlayerError"));
        bool said = false;
        for (const QLabel* label : test->host->findChildren<QLabel*>()) {
          said = said ||
                 (label->isVisible() &&
                  label->text() == QStringLiteral("Couldn't play this file"));
        }
        return Expect(error != nullptr && error->isVisible() && said,
                      QStringLiteral("a missing file didn't say so"));
      });
  Add(
      QStringLiteral("image-preview-enlarge"),
      [test] {
        delete test->host;
        auto* window = new QWidget(nullptr);
        window->resize(kHostWidth, kHostHeight);
        auto* layout = new QVBoxLayout(window);
        auto* preview = new ImagePreview(
            ToQ(App::Get().session().file_path(
                ToRust(QStringLiteral("app:placeholders/launch-earth.jpg")))),
            QStringLiteral("Earth"), kNarrowWidth, window);
        layout->addWidget(preview);
        window->show();
        test->host = window;
        QTest::mouseClick(preview, Qt::LeftButton, {},
                          preview->rect().center());
      },
      800,
      [] {
        QWidget* dialog = QApplication::activeModalWidget();
        const bool shown =
            dialog != nullptr &&
            dialog->objectName() == QStringLiteral("ImagePreviewDialog");
        if (dialog != nullptr) dialog->close();
        return Expect(shown, QStringLiteral("the picture didn't enlarge"));
      });
  Add(
      QStringLiteral("player-closed"), [test] { delete test->host; }, 300,
      [test] {
        return Expect(App::Get().session().player_view(test->id).failed,
                      QStringLiteral("closing the player left it open"));
      });
}

}
