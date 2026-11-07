#include <QPointer>
#include <QString>
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
              ToRust(test->other),
              ToRust(QStringLiteral("Burst %1").arg(n)));
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
            return Expect(ToQ(card.title) != QStringLiteral("Elsewhere") &&
                              !ToQ(card.title).startsWith(
                                  QStringLiteral("Burst")),
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

}
