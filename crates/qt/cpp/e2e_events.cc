#include <QApplication>
#include <QFile>
#include <QJsonDocument>
#include <QJsonObject>
#include <QPointer>
#include <QString>
#include <QTimer>
#include <memory>

#include "app.h"
#include "event_hub.h"
#include "inspector_panel.h"
#include "main_window.h"
#include "mixer_panel.h"
#include "opsis-qt/src/bridge.rs.h"
#include "scenario.h"

namespace opsis {

namespace {

constexpr int kBurst = 10;
constexpr int kSaveMs = 1500;
constexpr int kDiskMs = 10'000;

QString Expect(bool ok, const QString& what) { return ok ? QString() : what; }

struct EventsTest {
  QString shown;
  QString other;
  QPointer<QWidget> content;
  int builds = 0;
  int dispatched = 0;
};

InspectorPanel* Inspector(MainWindow* window) {
  return qobject_cast<InspectorPanel*>(window->inspector_dock()->widget());
}

MixerPanel* Mixer(MainWindow* window) {
  return qobject_cast<MixerPanel*>(window->mixer_dock()->widget());
}

}

void Scenario::BuildEvents(MainWindow* window) {
  auto test = std::make_shared<EventsTest>();
  const auto document = static_cast<std::uint8_t>(bridge::EventKind::Document);
  Add(
      QStringLiteral("select-a-node-for-events"),
      [test] {
        const bridge::GraphView graph = App::Get().session().graph();
        if (graph.nodes.size() < 2) return;
        test->shown = ToQ(graph.nodes.at(0).id);
        test->other = ToQ(graph.nodes.at(1).id);
        rust::Vec<rust::String> ids;
        ids.push_back(ToRust(test->shown));
        App::Get().session().select(std::move(ids));
      },
      800,
      [test, window] {
        InspectorPanel* inspector = Inspector(window);
        if (inspector == nullptr || test->other.isEmpty()) {
          return QStringLiteral("no Inspector or too few nodes");
        }
        test->content = inspector->content();
        test->builds = inspector->builds();
        return Expect(App::Get().session().selected() == ToRust(test->shown),
                      QStringLiteral("the node wasn't selected"));
      });
  Add(
      QStringLiteral("another-nodes-edit-keeps-the-inspector"),
      [test] {
        App::Get().session().rename_node(ToRust(test->other),
                                         ToRust(QStringLiteral("Elsewhere")));
      },
      1200,
      [test, window] {
        InspectorPanel* inspector = Inspector(window);
        return Expect(
            !test->content.isNull() &&
                inspector->content() == test->content.data() &&
                inspector->builds() == test->builds,
            QStringLiteral("editing another node rebuilt the Inspector "
                           "(%1 builds)")
                .arg(inspector->builds() - test->builds));
      });
  Add(
      QStringLiteral("a-burst-of-edits-dispatches-once"),
      [test, document] {
        test->dispatched = EventHub::Instance()->dispatched(document);
        for (int n = 0; n < kBurst; ++n) {
          App::Get().session().rename_node(
              ToRust(test->other), ToRust(QStringLiteral("Burst %1").arg(n)));
        }
      },
      1200,
      [test, document] {
        const int dispatched =
            EventHub::Instance()->dispatched(document) - test->dispatched;
        return Expect(dispatched > 0 && dispatched < kBurst / 2,
                      QStringLiteral("%1 edits dispatched %2 times")
                          .arg(kBurst)
                          .arg(dispatched));
      });
  Add(
      QStringLiteral("undo-the-burst"),
      [test] {
        test->builds = 0;
        for (int n = 0; n <= kBurst; ++n) {
          if (App::Get().session().undo()) test->builds += 1;
        }
      },
      800,
      [test] {
        const bridge::GraphView graph = App::Get().session().graph();
        for (const auto& card : graph.nodes) {
          if (ToQ(card.id) == test->other) {
            return Expect(
                ToQ(card.title) != QStringLiteral("Elsewhere") &&
                    !ToQ(card.title).startsWith(QStringLiteral("Burst")),
                QStringLiteral("undo didn't bring the name back: "
                               "'%1' after %2 undos")
                    .arg(ToQ(card.title))
                    .arg(test->builds));
          }
        }
        return QStringLiteral("the other node is gone");
      });
  Add(
      QStringLiteral("unchanged-media-keeps-the-mixer"),
      [test, window] {
        MixerPanel* mixer = Mixer(window);
        test->builds = mixer == nullptr ? 0 : mixer->builds();
        EventHub::Instance()->Post(
            static_cast<std::uint8_t>(bridge::EventKind::Media), {}, true);
      },
      400,
      [test, window] {
        MixerPanel* mixer = Mixer(window);
        return Expect(mixer != nullptr && mixer->builds() == test->builds,
                      QStringLiteral("the Mixer rebuilt with nothing new"));
      });
}

void Scenario::BuildDisk() {
  const auto file = [] {
    return ToQ(App::Get().session().graph().dir) +
           QStringLiteral("/project.opsis.json");
  };
  const auto rewrite = [file](const QString& name) {
    QFile project(file());
    if (!project.open(QIODevice::ReadOnly)) return;
    QJsonObject json = QJsonDocument::fromJson(project.readAll()).object();
    project.close();
    json.insert(QStringLiteral("name"), name);
    QFile written(file() + QStringLiteral(".outside"));
    if (!written.open(QIODevice::WriteOnly)) return;
    written.write(QJsonDocument(json).toJson());
    written.close();
    QFile::remove(file());
    QFile::rename(file() + QStringLiteral(".outside"), file());
  };
  auto saved = std::make_shared<QString>();
  Until(
      QStringLiteral("disk-change-loads-a-saved-project"),
      [saved, rewrite] {
        *saved = ToQ(App::Get().session().graph().name);
        QTimer::singleShot(
            kSaveMs, [rewrite] { rewrite(QStringLiteral("Edited outside")); });
      },
      kDiskMs,
      [] {
        return App::Get().session().graph().name ==
               ToRust(QStringLiteral("Edited outside"));
      },
      [] {
        return Expect(App::Get().session().graph().name ==
                          ToRust(QStringLiteral("Edited outside")),
                      QStringLiteral("the outside change wasn't loaded"));
      });
  Add(
      QStringLiteral("undo-the-outside-change"),
      [] { App::Get().session().undo(); }, 400,
      [saved] {
        return Expect(ToQ(App::Get().session().graph().name) == *saved,
                      QStringLiteral("undo didn't bring the project back"));
      });
  Until(
      QStringLiteral("disk-change-asks-with-unsaved-edits"),
      [rewrite] {
        const QJsonObject rename{
            {QStringLiteral("op"), QStringLiteral("rename")},
            {QStringLiteral("name"), QStringLiteral("Mine")}};
        App::Get().session().project_edit(ToRust(QString::fromUtf8(
            QJsonDocument(rename).toJson(QJsonDocument::Compact))));
        rewrite(QStringLiteral("Theirs"));
      },
      kDiskMs,
      [] {
        for (QWidget* top : QApplication::topLevelWidgets()) {
          if (top->findChild<QWidget*>(QStringLiteral("DiskChangeDialog")))
            return true;
        }
        return false;
      },
      [this] {
        QWidget* dialog = nullptr;
        for (QWidget* top : QApplication::topLevelWidgets()) {
          if (auto* found = top->findChild<QWidget*>(
                  QStringLiteral("DiskChangeDialog"))) {
            dialog = found;
          }
        }
        if (dialog != nullptr) Click(dialog, QStringLiteral("Keep mine"));
        return Expect(dialog != nullptr, QStringLiteral("the outside change "
                                                        "didn't ask"));
      });
  Until(
      QStringLiteral("keep-mine-saves-over-the-file"), [] {}, kDiskMs,
      [file] {
        QFile project(file());
        return project.open(QIODevice::ReadOnly) &&
               QJsonDocument::fromJson(project.readAll())
                       .object()
                       .value(QStringLiteral("name"))
                       .toString() == QStringLiteral("Mine");
      },
      [saved, file] {
        QFile project(file());
        const bool mine = project.open(QIODevice::ReadOnly) &&
                          QJsonDocument::fromJson(project.readAll())
                                  .object()
                                  .value(QStringLiteral("name"))
                                  .toString() == QStringLiteral("Mine");
        const QJsonObject back{{QStringLiteral("op"), QStringLiteral("rename")},
                               {QStringLiteral("name"), *saved}};
        App::Get().session().project_edit(ToRust(QString::fromUtf8(
            QJsonDocument(back).toJson(QJsonDocument::Compact))));
        return Expect(mine, QStringLiteral("keeping mine didn't save over the "
                                           "file"));
      });
}

}
