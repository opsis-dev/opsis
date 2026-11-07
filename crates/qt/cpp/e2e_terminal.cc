#include <QApplication>
#include <QDir>
#include <QDirIterator>
#include <QFile>
#include <QJsonDocument>
#include <QJsonObject>
#include <QtTest/QTest>
#include <memory>

#include "app.h"
#include "file_player.h"
#include "main_window.h"
#include "opsis-qt/src/bridge.rs.h"
#include "scenario.h"
#include "terminal_window.h"

namespace opsis {

namespace {

constexpr int kRunMs = 400'000;
constexpr int kWindowMs = 60'000;
constexpr int kDialogMs = 20'000;

QString Expect(bool ok, const QString& what) { return ok ? QString() : what; }

QString FindFile(const QString& dir, const QString& name) {
  QDirIterator files(dir, {name}, QDir::Files | QDir::Hidden,
                     QDirIterator::Subdirectories);
  return files.hasNext() ? files.next() : QString();
}

QString Read(const QString& file) {
  QFile opened(file);
  return opened.open(QIODevice::ReadOnly) ? QString::fromUtf8(opened.readAll())
                                          : QString();
}

TerminalWindow* OpenTerminal() {
  for (QWidget* top : QApplication::topLevelWidgets()) {
    if (auto* window = qobject_cast<TerminalWindow*>(top);
        window != nullptr && window->isVisible()) {
      return window;
    }
  }
  return nullptr;
}

bool Running() { return App::Get().session().bar().running; }

void Patch(const QString& node, const QJsonObject& params) {
  App::Get().session().set_params(
      ToRust(node), ToRust(QString::fromUtf8(
                        QJsonDocument(params).toJson(QJsonDocument::Compact))));
}

}

void Scenario::BuildTerminalRuns() {
  const QString dir = qEnvironmentVariable("OPSIS_QT_PROJECTS") +
                      QStringLiteral("/terminal");
  Until(
      QStringLiteral("terminal-example-records-and-exports"),
      [dir] {
        App::Get().session().open_project(ToRust(dir));
        App::Get().session().run_node(ToRust(QStringLiteral("render")), false);
      },
      kRunMs, [] { return !Running(); },
      [dir] {
        const bridge::RunPanelView view = App::Get().session().run_panel();
        const QString cast = FindFile(dir, QStringLiteral("take.cast"));
        const QString preview = FindFile(dir, QStringLiteral("preview.mp4"));
        const QString film = FindFile(dir, QStringLiteral("sample-terminal.mp4"));
        return Expect(view.failure.empty() && !cast.isEmpty() &&
                          Read(cast).contains(QStringLiteral(
                              "Opsis records real terminals")) &&
                          !preview.isEmpty() && QFile(preview).size() > 0 &&
                          !film.isEmpty() && QFile(film).size() > 0,
                      QStringLiteral("the terminal example didn't record and "
                                     "export: %1")
                          .arg(ToQ(view.failure)));
      });
  Until(
      QStringLiteral("terminal-take-plays-in-replay"),
      [] {
        App::Get().session().show_replay(ToRust(QStringLiteral("rec")),
                                         ToRust(QStringLiteral("replay")));
      },
      kDialogMs,
      [] {
        for (QWidget* top : QApplication::topLevelWidgets()) {
          auto* dialog = top->findChild<QWidget*>(QStringLiteral("ReplayDialog"));
          auto* player = dialog == nullptr ? nullptr : dialog->findChild<FilePlayer*>();
          if (player != nullptr &&
              App::Get().session().player_view(player->id()).serial > 0) {
            return true;
          }
        }
        return false;
      },
      [] {
        const bridge::ReplayView replay = App::Get().session().replay(
            ToRust(QStringLiteral("rec")), ToRust(QStringLiteral("replay")));
        for (QWidget* top : QApplication::topLevelWidgets()) {
          if (auto* dialog = top->findChild<QWidget*>(QStringLiteral("ReplayDialog"))) {
            dialog->close();
          }
        }
        return Expect(ToQ(replay.video_file).endsWith(QStringLiteral("preview.mp4")),
                      QStringLiteral("the Replay plays '%1', not the take's "
                                     "preview")
                          .arg(ToQ(replay.video_file)));
      });
  auto nodes = std::make_shared<QStringList>();
  Until(
      QStringLiteral("live-tape-opens-a-terminal-window"),
      [dir, nodes] {
        QFile::remove(dir + QStringLiteral("/live.tape"));
        auto& session = App::Get().session();
        const QString tape = ToQ(session.add_node(ToRust(QStringLiteral("terminal.playTape")), 2400, 0));
        const QString take = ToQ(session.add_node(ToRust(QStringLiteral("camera.take")), 2700, 0));
        const QString film = ToQ(session.add_node(ToRust(QStringLiteral("output.render")), 3000, 0));
        *nodes = {tape, take, film};
        Patch(tape, {{QStringLiteral("file"), QStringLiteral("project:live.tape")}});
        Patch(film, {{QStringLiteral("fileName"), QStringLiteral("tape film")}});
        session.connect(ToRust(tape), ToRust(QStringLiteral("recording")), ToRust(take),
                        ToRust(QStringLiteral("recording")));
        session.connect(ToRust(tape), ToRust(QStringLiteral("style")), ToRust(take),
                        ToRust(QStringLiteral("style")));
        session.connect(ToRust(take), ToRust(QStringLiteral("clip")), ToRust(film),
                        ToRust(QStringLiteral("clip")));
        session.run_node(ToRust(tape), false);
      },
      kWindowMs,
      [] {
        TerminalWindow* window = OpenTerminal();
        return window != nullptr && window->canvas()->HasPicture();
      },
      [] {
        return Expect(OpenTerminal() != nullptr,
                      QStringLiteral("no terminal window opened"));
      });
  auto before = std::make_shared<std::uint64_t>(0);
  Until(
      QStringLiteral("typing-shows-in-the-terminal"),
      [before] {
        TerminalWindow* window = OpenTerminal();
        if (window == nullptr) return;
        *before = window->canvas()->pictures();
        window->canvas()->setFocus();
        QTest::keyClicks(window->canvas(), QStringLiteral("echo live-tape"));
        QTest::keyClick(window->canvas(), Qt::Key_Return);
      },
      kWindowMs,
      [before] {
        TerminalWindow* window = OpenTerminal();
        return window != nullptr && window->canvas()->pictures() > *before;
      },
      [before] {
        TerminalWindow* window = OpenTerminal();
        return Expect(window != nullptr && window->canvas()->pictures() > *before,
                      QStringLiteral("the terminal never showed what was "
                                     "typed"));
      });
  Until(
      QStringLiteral("save-the-tape"),
      [this] {
        if (TerminalWindow* window = OpenTerminal()) {
          Click(window, QStringLiteral("Save tape"));
        }
      },
      kRunMs, [] { return OpenTerminal() == nullptr && !Running(); },
      [dir] {
        const QString tape = Read(dir + QStringLiteral("/live.tape"));
        return Expect(tape.contains(QStringLiteral("Type \"echo live-tape\"")),
                      QStringLiteral("the tape didn't keep what was typed: %1")
                          .arg(tape.left(200)));
      });
  Until(
      QStringLiteral("tape-plays-into-a-film"),
      [nodes] {
        if (nodes->size() == 3) {
          App::Get().session().run_node(ToRust(nodes->at(2)), false);
        }
      },
      kRunMs, [] { return !Running(); },
      [dir, nodes] {
        const QString film = FindFile(dir, QStringLiteral("tape-film.mp4"));
        bool cast = false;
        QDirIterator casts(dir, {QStringLiteral("take.cast")},
                           QDir::Files | QDir::Hidden, QDirIterator::Subdirectories);
        while (casts.hasNext()) {
          cast = cast || Read(casts.next()).contains(QStringLiteral("live-tape"));
        }
        rust::Vec<rust::String> remove;
        for (const QString& node : *nodes) remove.push_back(ToRust(node));
        App::Get().session().remove(std::move(remove), {});
        QFile::remove(dir + QStringLiteral("/live.tape"));
        return Expect(!film.isEmpty() && QFile(film).size() > 0 && cast,
                      QStringLiteral("the tape didn't play into a film"));
      });
}

}
