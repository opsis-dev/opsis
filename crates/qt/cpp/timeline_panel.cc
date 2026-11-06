#include "timeline_panel.h"

#include <QHBoxLayout>
#include <QVBoxLayout>
#include <array>

#include "app.h"
#include "event_hub.h"
#include "opsis-qt/src/bridge.rs.h"
#include "playback_clock.h"
#include "timeline_canvas.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr double kZoomStep = 1.5;
constexpr int kIcon = 16;
constexpr int kKeysWidth = 144;
constexpr int kSoundKeysWidth = 112;

struct ToolSpec {
  const char* id;
  const char* icon;
  const char* label;
  const char* key;
  const char* help;
};

constexpr std::array<ToolSpec, 6> kTools = {{
    {"select", "mouse-pointer-2", "Select", "V",
     "Move clips and drag their edges to trim. Hold Shift to move later "
     "clips too."},
    {"ripple", "arrow-right-from-line", "Ripple", "B",
     "Moves and trims push the clips after them."},
    {"roll", "arrow-left-right", "Roll", "N",
     "Drag the cut between two clips. The total length stays the same."},
    {"slip", "gallery-horizontal", "Slip", "Y",
     "Shows a different part of a clip without changing where it sits."},
    {"blade", "scissors", "Split", "C", "Click a clip to cut it in two."},
    {"rate", "gauge", "Speed", "R",
     "Drag a clip's edge to speed it up or slow it down."},
}};

QWidget* EmptyState(QWidget* parent) {
  auto* box = new QWidget(parent);
  auto* column = new QVBoxLayout(box);
  column->addStretch(1);
  auto* icon = new QLabel(box);
  icon->setPixmap(App::Get().Glyph(QStringLiteral("square-chart-gantt"),
                                   App::Get().Token("muted-foreground"), 28));
  icon->setAlignment(Qt::AlignCenter);
  column->addWidget(icon);
  auto* title = MakeLabel(QStringLiteral("No film to show yet"), nullptr, box);
  title->setAlignment(Qt::AlignCenter);
  column->addWidget(title);
  auto* about = MakeLabel(
      QStringLiteral(
          "Connect clips to an export, or select a node that makes a clip."),
      "muted", box);
  about->setAlignment(Qt::AlignCenter);
  about->setWordWrap(true);
  column->addWidget(about);
  column->addStretch(2);
  return box;
}

}

TimelinePanel::TimelinePanel(QWidget* parent)
    : QWidget(parent),
      stack_(new QStackedWidget(this)),
      canvas_(new TimelineCanvas()),
      play_(MakeIconButton(QStringLiteral("play"),
                           QStringLiteral("Play  Space"), nullptr, kIcon)),
      clock_(MakeButton(QStringLiteral("0:00 / 0:00"), "ghost")),
      snap_(
          MakeIconButton(QStringLiteral("magnet"), QString(), nullptr, kIcon)),
      video_keys_(new Combo()),
      audio_keys_(new Combo()),
      range_(new QWidget()),
      range_text_(new QLabel()),
      source_(new QLabel()) {
  setObjectName(QStringLiteral("Panel"));
  setAttribute(Qt::WA_StyledBackground);
  auto* layout = new QVBoxLayout(this);
  layout->setContentsMargins(0, 0, 0, 0);
  layout->setSpacing(0);
  auto* editor = new QWidget(this);
  auto* column = new QVBoxLayout(editor);
  column->setContentsMargins(0, 0, 0, 0);
  column->setSpacing(0);
  auto* bar = new QWidget(editor);
  bar->setObjectName(QStringLiteral("TimelineToolbar"));
  bar->setAttribute(Qt::WA_StyledBackground);
  auto* row = new QHBoxLayout(bar);
  row->setContentsMargins(8, 4, 8, 4);
  row->setSpacing(4);
  play_->setParent(bar);
  row->addWidget(play_);
  clock_->setParent(bar);
  clock_->setObjectName(QStringLiteral("TimelineClock"));
  clock_->setToolTip(QStringLiteral("Switch between seconds and timecode"));
  row->addWidget(clock_);
  auto* rule = new QFrame(bar);
  rule->setProperty("role", "vrule");
  rule->setFixedSize(1, 18);
  row->addWidget(rule);
  for (const ToolSpec& spec : kTools) {
    auto* button = MakeIconButton(
        QLatin1String(spec.icon),
        QStringLiteral("%1  %2\n%3")
            .arg(QLatin1String(spec.label), QLatin1String(spec.key),
                 QLatin1String(spec.help)),
        bar, kIcon);
    button->setAccessibleName(QLatin1String(spec.label));
    button->setCheckable(true);
    button->setProperty("tool", QLatin1String(spec.id));
    tools_.addButton(button);
    row->addWidget(button);
  }
  tools_.setExclusive(true);
  snap_->setParent(bar);
  snap_->setCheckable(true);
  snap_->setAccessibleName(QStringLiteral("Snap"));
  snap_->setToolTip(QStringLiteral(
      "Snap to the playhead, clip edges, markers, actions and beats. Hold Alt "
      "to turn it off while dragging."));
  row->addWidget(snap_);
  video_keys_->setParent(bar);
  video_keys_->setAccessibleName(QStringLiteral("Clip keyframes"));
  video_keys_->setToolTip(QStringLiteral("Clip keyframes"));
  video_keys_->setFixedWidth(kKeysWidth);
  audio_keys_->setParent(bar);
  audio_keys_->setAccessibleName(QStringLiteral("Sound keyframes"));
  audio_keys_->setToolTip(QStringLiteral("Sound keyframes"));
  audio_keys_->setFixedWidth(kSoundKeysWidth);
  row->addWidget(video_keys_);
  row->addWidget(audio_keys_);
  range_->setParent(bar);
  range_->setObjectName(QStringLiteral("RangeBadge"));
  range_->setAttribute(Qt::WA_StyledBackground);
  auto* range_row = new QHBoxLayout(range_);
  range_row->setContentsMargins(6, 0, 2, 0);
  range_row->setSpacing(2);
  range_row->addWidget(range_text_);
  auto* clear = MakeIconButton(QStringLiteral("x"),
                               QStringLiteral("Clear the loop"), range_, 12);
  range_row->addWidget(clear);
  row->addWidget(range_);
  row->addStretch(1);
  source_->setParent(bar);
  source_->setObjectName(QStringLiteral("SourceBadge"));
  row->addWidget(source_);
  auto* out = MakeIconButton(QStringLiteral("minus"),
                             QStringLiteral("Zoom out  −"), bar, kIcon);
  auto* in = MakeIconButton(QStringLiteral("plus"),
                            QStringLiteral("Zoom in  ="), bar, kIcon);
  auto* fit = MakeIconButton(QStringLiteral("maximize"),
                             QStringLiteral("Fit the film  Z"), bar, kIcon);
  row->addWidget(out);
  row->addWidget(in);
  row->addWidget(fit);
  column->addWidget(bar);
  column->addWidget(canvas_, 1);
  stack_->addWidget(EmptyState(stack_));
  stack_->addWidget(editor);
  layout->addWidget(stack_);

  connect(play_, &QToolButton::clicked, this,
          [] { PlaybackClock::Instance()->Toggle(); });
  connect(clock_, &QPushButton::clicked, this,
          [this] { canvas_->SetTimecode(!canvas_->timecode()); });
  connect(&tools_, &QButtonGroup::buttonClicked, this,
          [this](QAbstractButton* button) {
            canvas_->SetTool(button->property("tool").toString());
            canvas_->setFocus();
          });
  connect(snap_, &QToolButton::toggled, this,
          [this](bool on) { canvas_->SetSnapping(on); });
  connect(video_keys_, &QComboBox::activated, this, [this](int) {
    canvas_->SetVideoTrack(video_keys_->currentData().toString());
  });
  connect(audio_keys_, &QComboBox::activated, this, [this](int) {
    canvas_->SetAudioTrack(audio_keys_->currentData().toString());
  });
  connect(clear, &QToolButton::clicked, this,
          [] { PlaybackClock::Instance()->ClearRange(); });
  connect(out, &QToolButton::clicked, this,
          [this] { canvas_->ZoomBy(1 / kZoomStep); });
  connect(in, &QToolButton::clicked, this,
          [this] { canvas_->ZoomBy(kZoomStep); });
  connect(fit, &QToolButton::clicked, this, [this] { canvas_->Fit(); });
  connect(canvas_, &TimelineCanvas::StateChanged, this,
          &TimelinePanel::SyncBar);
  connect(PlaybackClock::Instance(), &PlaybackClock::Changed, this,
          &TimelinePanel::SyncBar);
  for (auto signal : {&EventHub::CompiledChanged, &EventHub::DocumentChanged,
                      &EventHub::SelectionChanged, &EventHub::ProjectChanged}) {
    connect(EventHub::Instance(), signal, this, &TimelinePanel::Reload);
  }
  Reload();
}

void TimelinePanel::Reload() {
  canvas_->Reload();
  stack_->setCurrentIndex(canvas_->has_film() ? 1 : 0);
  const bridge::TimelineView* view = canvas_->view();
  if (view == nullptr || view->video_tracks.empty() ||
      video_keys_->count() > 0) {
    return;
  }
  video_keys_->addItem(QStringLiteral("No keyframes"), QString());
  for (const auto& choice : view->video_tracks) {
    video_keys_->addItem(ToQ(choice.label), ToQ(choice.value));
  }
  for (const auto& choice : view->audio_tracks) {
    audio_keys_->addItem(ToQ(choice.label), ToQ(choice.value));
  }
  audio_keys_->setCurrentIndex(
      std::max(0, audio_keys_->findData(canvas_->audio_track())));
}

void TimelinePanel::SyncBar() {
  const PlaybackClock* clock = PlaybackClock::Instance();
  const bool playing = clock->playing();
  play_->setIcon(App::Get().Icon(
      playing ? QStringLiteral("pause") : QStringLiteral("play"),
      App::Get().Token("foreground"), kIcon));
  play_->setToolTip(playing ? QStringLiteral("Pause  Space")
                            : QStringLiteral("Play  Space"));
  play_->setAccessibleName(playing ? QStringLiteral("Pause")
                                   : QStringLiteral("Play"));
  clock_->setText(canvas_->TimeText(clock->frame()) + QStringLiteral(" / ") +
                  canvas_->TimeText(clock->duration()));
  for (QAbstractButton* button : tools_.buttons()) {
    const bool on = button->property("tool").toString() == canvas_->tool();
    if (button->isChecked() != on) button->setChecked(on);
  }
  if (snap_->isChecked() != canvas_->snapping()) {
    snap_->setChecked(canvas_->snapping());
  }
  const bool looped = clock->in_point() >= 0 || clock->out_point() >= 0;
  range_->setVisible(looped);
  if (looped) {
    const double from = std::max(0.0, clock->in_point());
    const double to =
        clock->out_point() >= 0 ? clock->out_point() : clock->duration();
    range_text_->setText(canvas_->TimeText(from) + QStringLiteral("–") +
                         canvas_->TimeText(to));
  }
  const bridge::TimelineView* view = canvas_->view();
  const QString source = view != nullptr ? ToQ(view->source_label) : QString();
  source_->setText(source);
  source_->setVisible(!source.isEmpty());
}

}
