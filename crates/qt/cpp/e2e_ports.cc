#include <QApplication>
#include <QComboBox>
#include <QDir>
#include <QDirIterator>
#include <QFile>
#include <QGraphicsItem>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QLabel>
#include <QPainterPath>
#include <QtTest/QTest>
#include <algorithm>
#include <cmath>
#include <map>
#include <memory>
#include <vector>

#include "app.h"
#include "file_player.h"
#include "graph_canvas.h"
#include "main_window.h"
#include "marks_field.h"
#include "opsis-qt/src/bridge.rs.h"
#include "playback_clock.h"
#include "scenario.h"
#include "start_screen.h"
#include "timeline_canvas.h"
#include "timeline_panel.h"
#include "var_text.h"

namespace opsis {

namespace {

constexpr int kOpenMs = 20'000;
constexpr int kRunMs = 400'000;
constexpr int kPlayMs = 20'000;
constexpr int kVerticalRun = 8;
constexpr double kNear = 0.02;
constexpr int kThemes = 300;

const std::array<const char*, 6> kExamples = {"todomvc",       "feature-launch",
                                              "changelog",     "onboarding",
                                              "social-teaser", "before-after"};

QString Expect(bool ok, const QString& what) { return ok ? QString() : what; }

QString Projects() { return qEnvironmentVariable("OPSIS_QT_PROJECTS"); }

bool Running() { return App::Get().session().bar().running; }

void Patch(const QString& node, const QJsonObject& params) {
  App::Get().session().set_params(
      ToRust(node), ToRust(QString::fromUtf8(
                        QJsonDocument(params).toJson(QJsonDocument::Compact))));
}

void Select(const QString& node) {
  rust::Vec<rust::String> ids;
  ids.push_back(ToRust(node));
  App::Get().session().select(std::move(ids));
}

const bridge::CardView* CardOf(const bridge::GraphView& graph,
                               const QString& id) {
  for (const auto& card : graph.nodes) {
    if (ToQ(card.id) == id) return &card;
  }
  return nullptr;
}

QString CardText(const QString& id) {
  const bridge::GraphView graph = App::Get().session().graph();
  const bridge::CardView* card = CardOf(graph, id);
  if (card == nullptr) return {};
  QStringList parts{ToQ(card->title), ToQ(card->text), ToQ(card->badge)};
  for (const auto& row : card->rows) parts << ToQ(row.text);
  return parts.join(QLatin1Char(' '));
}

QString Positions() {
  QStringList parts;
  for (const auto& card : App::Get().session().graph().nodes) {
    parts
        << QStringLiteral("%1:%2,%3").arg(ToQ(card.id)).arg(card.x).arg(card.y);
  }
  parts.sort();
  return parts.join(QLatin1Char(';'));
}

QString ErrorIssues() {
  QStringList errors;
  for (const auto& issue : App::Get().session().project().issues) {
    if (issue.error) errors << ToQ(issue.message);
  }
  return errors.join(QStringLiteral("; "));
}

QJsonValue FieldValue(const QString& key) {
  for (const auto& field : App::Get().session().inspector().fields) {
    if (ToQ(field.key) == key) {
      return QJsonDocument::fromJson("[" + ToQ(field.value_json).toUtf8() + "]")
          .array()
          .at(0);
    }
  }
  return {};
}

QWidget* FieldWidget(MainWindow* window, const QString& label) {
  QWidget* panel = window->inspector_dock()->widget();
  for (QLabel* text : panel->findChildren<QLabel*>()) {
    if (text->isVisible() && text->text() == label) return text->parentWidget();
  }
  return nullptr;
}

std::vector<std::pair<double, std::pair<double, double>>> VerticalRuns(
    const QPainterPath& path) {
  std::vector<std::pair<double, std::pair<double, double>>> runs;
  for (int i = 1; i < path.elementCount(); ++i) {
    const QPainterPath::Element a = path.elementAt(i - 1);
    const QPainterPath::Element b = path.elementAt(i);
    if (std::abs(a.x - b.x) < 0.5 && std::abs(a.y - b.y) > kVerticalRun) {
      runs.push_back({a.x, {std::min(a.y, b.y), std::max(a.y, b.y)}});
    }
  }
  return runs;
}

QString Aurora() {
  return ToQ(App::Get().session().file_path(
      ToRust(QStringLiteral("app:backdrops/aurora.jpg"))));
}

QString FindFile(const QString& dir, const QString& name) {
  QDirIterator files(dir, {name}, QDir::Files | QDir::Hidden,
                     QDirIterator::Subdirectories);
  return files.hasNext() ? files.next() : QString();
}

}

void Scenario::BuildPorts() {
  MainWindow* window = window_;
  Add(
      QStringLiteral("start-screen-lists-actions-and-templates"), [] {}, 300,
      [window] {
        QStringList shown;
        for (QLabel* label : window->start_screen()->findChildren<QLabel*>()) {
          if (label->isVisible()) shown << label->text();
        }
        QStringList missing;
        for (const QString& want :
             {QStringLiteral("New project…"), QStringLiteral("Changelog clip"),
              QStringLiteral("Sample: to-do app")}) {
          bool found = shown.contains(want);
          for (QAbstractButton* button :
               window->start_screen()->findChildren<QAbstractButton*>()) {
            found = found || (button->isVisible() && button->text() == want);
          }
          if (!found) missing << want;
        }
        return Expect(missing.isEmpty() &&
                          !App::Get().session().engine_view().version.empty(),
                      QStringLiteral("the start screen misses %1")
                          .arg(missing.join(", ")));
      });
  for (const char* example : kExamples) {
    const QString name = QString::fromLatin1(example);
    Until(
        QStringLiteral("example-%1-opens-whole").arg(name),
        [name] {
          App::Get().session().open_project(
              ToRust(Projects() + QLatin1Char('/') + name));
        },
        kOpenMs,
        [window] {
          return window->canvas()->card_count() ==
                     static_cast<int>(
                         App::Get().session().graph().nodes.size()) &&
                 window->canvas()->card_count() > 0;
        },
        [window, name] {
          const int nodes =
              static_cast<int>(App::Get().session().graph().nodes.size());
          const QString errors = ErrorIssues();
          return Expect(
              window->canvas()->card_count() == nodes && errors.isEmpty(),
              QStringLiteral("%1 shows %2 of %3 nodes; problems: %4")
                  .arg(name)
                  .arg(window->canvas()->card_count())
                  .arg(nodes)
                  .arg(errors));
        });
  }
  auto before = std::make_shared<QString>();
  Until(
      QStringLiteral("tidy-up-lays-out-a-third-clip-apart"),
      [this, window, before] {
        App::Get().session().open_project(
            ToRust(Projects() + QStringLiteral("/feature-launch")));
        *before = Positions();
        Click(window, QStringLiteral("Tidy up layout"));
      },
      kOpenMs, [before] { return Positions() != *before; },
      [window] {
        const bridge::GraphView graph = App::Get().session().graph();
        bool third = false;
        for (const auto& edge : graph.edges) {
          const QString handle = ToQ(edge.target_handle);
          third = third || (handle.startsWith(QStringLiteral("in")) &&
                            handle.mid(2).toInt() >= 3);
        }
        QStringList touching;
        for (const auto& a : graph.nodes) {
          if (!a.parent.empty()) continue;
          for (const auto& b : graph.nodes) {
            if (!b.parent.empty() || ToQ(a.id) >= ToQ(b.id)) continue;
            const bool apart = a.x + a.width <= b.x || b.x + b.width <= a.x ||
                               a.y + a.height <= b.y || b.y + b.height <= a.y;
            if (!apart) touching << ToQ(a.id) + QLatin1Char('/') + ToQ(b.id);
          }
        }
        std::map<QString,
                 std::vector<std::pair<
                     QString, std::pair<double, std::pair<double, double>>>>>
            by_target;
        for (const auto& edge : graph.edges) {
          for (QGraphicsItem* item : window->canvas()->scene()->items()) {
            auto* drawn = dynamic_cast<EdgeItem*>(item);
            if (drawn == nullptr || drawn->id() != ToQ(edge.id)) continue;
            for (const auto& run : VerticalRuns(drawn->path())) {
              by_target[ToQ(edge.target)].emplace_back(ToQ(edge.id), run);
            }
          }
        }
        QStringList stacked;
        for (const auto& [target, runs] : by_target) {
          for (const auto& [a_id, a] : runs) {
            for (const auto& [b_id, b] : runs) {
              if (a_id >= b_id) continue;
              const bool overlap = a.second.first < b.second.second &&
                                   b.second.first < a.second.second;
              if (overlap && std::abs(a.first - b.first) <= 0.5) {
                stacked << a_id + QLatin1Char('/') + b_id;
              }
            }
          }
        }
        return Expect(third && touching.isEmpty() && stacked.isEmpty(),
                      QStringLiteral("third clip %1; cards on top of each "
                                     "other: %2; edge runs on top of each "
                                     "other: %3")
                          .arg(third)
                          .arg(touching.join(", "), stacked.join(", ")));
      });
  Until(
      QStringLiteral("undo-the-tidy-up"), [] { App::Get().session().undo(); },
      kOpenMs, [before] { return Positions() == *before; },
      [before] {
        return Expect(Positions() == *before,
                      QStringLiteral("undo didn't put the nodes back"));
      });
  Until(
      QStringLiteral("timeline-marks-nodes-and-bars-at-the-playhead"),
      [window] {
        App::Get().session().open_project(
            ToRust(Projects() + QStringLiteral("/changelog")));
        App::Get().session().select({});
        PlaybackClock::Instance()->Seek(0);
        window->timeline_dock()->show();
      },
      kOpenMs,
      [window] {
        for (const QString& id : window->canvas()->CardIds()) {
          if (CardItem* card = window->canvas()->Card(id);
              card && card->active()) {
            return true;
          }
        }
        return false;
      },
      [window] {
        auto* panel =
            qobject_cast<TimelinePanel*>(window->timeline_dock()->widget());
        const bridge::TimelineView* view =
            panel == nullptr ? nullptr : panel->canvas()->view();
        const double frame = PlaybackClock::Instance()->frame();
        bool lit = false;
        if (view != nullptr) {
          for (const auto& bar : view->bars) {
            lit = lit || (frame >= bar.from && frame < bar.to);
          }
        }
        return Expect(lit, QStringLiteral("no timeline bar is under the "
                                          "playhead"));
      });
  auto clock = std::make_shared<double>(0);
  Until(
      QStringLiteral("timeline-plays-on-its-own-clock"),
      [this, window, clock] {
        *clock = PlaybackClock::Instance()->frame();
        auto* panel =
            qobject_cast<TimelinePanel*>(window->timeline_dock()->widget());
        if (panel != nullptr) Click(panel, QStringLiteral("Play"));
      },
      kPlayMs,
      [clock] { return PlaybackClock::Instance()->frame() > *clock + 5; },
      [this, window] {
        auto* panel =
            qobject_cast<TimelinePanel*>(window->timeline_dock()->widget());
        if (panel != nullptr) Click(panel, QStringLiteral("Pause"));
        return Expect(
            panel != nullptr && Find(panel, QStringLiteral("Play")) != nullptr,
            QStringLiteral("the timeline didn't pause"));
      });
  auto shot = std::make_shared<QStringList>();
  Until(
      QStringLiteral("annotate-shows-the-picture"),
      [shot] {
        const QString dir = Projects() + QStringLiteral("/changelog");
        QFile::copy(Aurora(), dir + QStringLiteral("/assets/shot.jpg"));
        auto& session = App::Get().session();
        const QString media = ToQ(
            session.add_node(ToRust(QStringLiteral("media.import")), 0, 1400));
        const QString notes = ToQ(session.add_node(
            ToRust(QStringLiteral("image.annotate")), 320, 1400));
        *shot = {media, notes};
        Patch(media,
              {{QStringLiteral("path"), QStringLiteral("assets/shot.jpg")}});
        session.connect(ToRust(media), ToRust(QStringLiteral("media")),
                        ToRust(notes), ToRust(QStringLiteral("image")));
        Select(notes);
      },
      kOpenMs,
      [window] {
        auto* frame =
            window->inspector_dock()->widget()->findChild<MarksFrame*>();
        return frame != nullptr && frame->isVisible() &&
               frame->PictureWidth() > 0;
      },
      [window] {
        return Expect(
            window->inspector_dock()->widget()->findChild<MarksFrame*>() !=
                nullptr,
            QStringLiteral("no marks picture in the Inspector"));
      });
  Until(
      QStringLiteral("annotate-draws-a-box-as-one-mark"),
      [this, window] {
        QWidget* panel = window->inspector_dock()->widget();
        Click(panel, QStringLiteral("Box"));
        auto* frame = panel->findChild<MarksFrame*>();
        if (frame == nullptr) return;
        Drag(frame, frame->ToWidget(0.25, 0.25).toPoint(),
             frame->ToWidget(0.6, 0.7).toPoint());
      },
      kOpenMs,
      [] { return FieldValue(QStringLiteral("marks")).toArray().size() == 1; },
      [shot] {
        const QJsonArray marks = FieldValue(QStringLiteral("marks")).toArray();
        const QJsonObject box = marks.at(0).toObject();
        const bool placed =
            marks.size() == 1 &&
            box.value(QStringLiteral("kind")).toString() ==
                QStringLiteral("rectangle") &&
            std::abs(box.value(QStringLiteral("x")).toDouble() - 0.25) <
                kNear &&
            std::abs(box.value(QStringLiteral("y2")).toDouble() - 0.7) < kNear;
        rust::Vec<rust::String> remove;
        for (const QString& node : *shot) remove.push_back(ToRust(node));
        App::Get().session().remove(std::move(remove), {});
        return Expect(placed,
                      QStringLiteral("the marks are %1")
                          .arg(QString::fromUtf8(QJsonDocument(marks).toJson(
                              QJsonDocument::Compact))));
      });
  auto made = std::make_shared<QString>();
  Until(
      QStringLiteral("inspector-plays-a-made-sound"),
      [made] {
        auto& session = App::Get().session();
        *made =
            ToQ(session.add_node(ToRust(QStringLiteral("audio.sfx")), 0, 1600));
        Patch(*made, {{QStringLiteral("source"), QStringLiteral("made")},
                      {QStringLiteral("sound"), QStringLiteral("ding")}});
        session.run_node(ToRust(*made), false);
      },
      kRunMs,
      [made, window] {
        if (Running()) return false;
        Select(*made);
        auto* player =
            window->inspector_dock()->widget()->findChild<FilePlayer*>();
        return player != nullptr &&
               App::Get().session().player_view(player->id()).ready;
      },
      [this, window] {
        auto* player =
            window->inspector_dock()->widget()->findChild<FilePlayer*>();
        if (player != nullptr) Click(player, QStringLiteral("Play"));
        return Expect(player != nullptr, QStringLiteral("the made sound has no "
                                                        "player"));
      });
  Until(
      QStringLiteral("the-made-sound-plays"), [] {}, kPlayMs,
      [window] {
        auto* player =
            window->inspector_dock()->widget()->findChild<FilePlayer*>();
        return player != nullptr &&
               App::Get().session().player_view(player->id()).position > 0.2;
      },
      [window, made] {
        auto* player =
            window->inspector_dock()->widget()->findChild<FilePlayer*>();
        const bool played =
            player != nullptr &&
            App::Get().session().player_view(player->id()).position > 0.2;
        if (player != nullptr) App::Get().session().player_pause(player->id());
        App::Get().session().delete_node(ToRust(*made));
        return Expect(played, QStringLiteral("the made sound never played"));
      });
  auto split = std::make_shared<QStringList>();
  Until(
      QStringLiteral("typing-waits-to-mark-changed"),
      [split] {
        const QString dir = Projects() + QStringLiteral("/changelog");
        QFile::copy(Aurora(), dir + QStringLiteral("/assets/pair.jpg"));
        auto& session = App::Get().session();
        const QString pair = ToQ(
            session.add_node(ToRust(QStringLiteral("media.import")), 0, 1900));
        const QString node = ToQ(session.add_node(
            ToRust(QStringLiteral("image.beforeAfter")), 320, 1900));
        *split = {pair, node};
        Patch(pair,
              {{QStringLiteral("path"), QStringLiteral("assets/pair.jpg")}});
        for (const char* handle : {"image", "after"}) {
          session.connect(ToRust(pair), ToRust(QStringLiteral("media")),
                          ToRust(node), ToRust(QString::fromLatin1(handle)));
        }
        session.run_node(ToRust(node), false);
      },
      kRunMs,
      [split, window] {
        if (Running() || split->size() != 2) return false;
        Select(split->at(1));
        return FieldWidget(window, QStringLiteral("Before label")) != nullptr &&
               !CardText(split->at(1)).contains(QStringLiteral("Changed"));
      },
      [split, window] {
        QWidget* field = FieldWidget(window, QStringLiteral("Before label"));
        auto* text =
            field == nullptr ? nullptr : field->findChild<VarSurface*>();
        bool focused = false;
        if (text != nullptr) {
          window->activateWindow();
          window->raise();
          (void)QTest::qWaitForWindowActive(window);
          QTest::mouseClick(text->viewport(), Qt::LeftButton, {},
                            text->viewport()->rect().center());
          focused = text->hasFocus();
          text->moveCursor(QTextCursor::Start);
          QTest::keyClicks(text, QStringLiteral("X"));
        }
        return Expect(
            text != nullptr && focused,
            QStringLiteral("the Before label field didn't take focus"));
      });
  Until(
      QStringLiteral("the-card-shows-typing-at-once"), [] {}, kOpenMs,
      [split] {
        return CardText(split->at(1)).contains(QStringLiteral("“XBefore”"));
      },
      [split] {
        const QString card = CardText(split->at(1));
        return Expect(
            card.contains(QStringLiteral("“XBefore”")) &&
                !card.contains(QStringLiteral("Changed")),
            QStringLiteral("while typing the card reads '%1'").arg(card));
      });
  Until(
      QStringLiteral("leaving-the-field-marks-changed"),
      [window] { window->canvas()->setFocus(); }, kOpenMs,
      [split] {
        return CardText(split->at(1)).contains(QStringLiteral("Changed"));
      },
      [split] {
        const bool changed =
            CardText(split->at(1)).contains(QStringLiteral("Changed"));
        rust::Vec<rust::String> remove;
        for (const QString& node : *split) remove.push_back(ToRust(node));
        App::Get().session().remove(std::move(remove), {});
        return Expect(changed,
                      QStringLiteral("leaving the field didn't mark the "
                                     "node changed"));
      });
  Until(
      QStringLiteral("changelog-exports"),
      [] {
        App::Get().session().open_project(
            ToRust(Projects() + QStringLiteral("/changelog")));
        App::Get().session().run_node(ToRust(QStringLiteral("render")), false);
      },
      kRunMs, [] { return !Running(); },
      [] {
        const QString film = FindFile(Projects() + QStringLiteral("/changelog"),
                                      QStringLiteral("changelog-clip.mp4"));
        return Expect(!film.isEmpty() && QFile(film).size() > 0 &&
                          App::Get().session().run_panel().failure.empty(),
                      QStringLiteral("the changelog didn't export"));
      });
  Until(
      QStringLiteral("run-panel-plays-the-export"),
      [this] {
        App::Get().session().select({});
        ClickMenu(QStringLiteral("View"), QStringLiteral("Run"));
      },
      kPlayMs,
      [window] {
        for (FilePlayer* player : window->findChildren<FilePlayer*>()) {
          const bridge::PlayerView view =
              App::Get().session().player_view(player->id());
          if (player->isVisible() && view.ready && view.duration > 0 &&
              view.has_video) {
            return true;
          }
        }
        return false;
      },
      [] { return QString(); });
  Until(
      QStringLiteral("terminal-look-lists-the-themes"),
      [] {
        App::Get().session().open_project(
            ToRust(Projects() + QStringLiteral("/terminal")));
        Select(QStringLiteral("look"));
      },
      kOpenMs,
      [window] {
        QWidget* field = FieldWidget(window, QStringLiteral("Theme"));
        auto* combo =
            field == nullptr ? nullptr : field->findChild<QComboBox*>();
        return combo != nullptr && combo->count() > kThemes;
      },
      [window] {
        QWidget* field = FieldWidget(window, QStringLiteral("Theme"));
        auto* combo =
            field == nullptr ? nullptr : field->findChild<QComboBox*>();
        return Expect(
            combo != nullptr && combo->count() > kThemes &&
                combo->findText(QStringLiteral("Catppuccin Mocha")) >= 0,
            QStringLiteral("the theme list has %1 themes")
                .arg(combo == nullptr ? 0 : combo->count()));
      });
}

}
