#include "run_panel.h"

#include <QDateTime>
#include <QFileInfo>
#include <QHBoxLayout>
#include <QLabel>
#include <QProgressBar>
#include <QPushButton>
#include <QScrollBar>
#include <QSplitter>
#include <QVBoxLayout>
#include <array>
#include <cmath>

#include "app.h"
#include "event_hub.h"
#include "file_player.h"
#include "opsis-qt/src/bridge.rs.h"
#include "replay_dialog.h"
#include "top_bar.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr int kIcon = 16;
constexpr int kPreviewWidth = 220;
constexpr int kNodesWidth = 220;
constexpr int kOutputsWidth = 264;

struct StatusLook {
  const char* state;
  const char* icon;
  const char* word;
  const char* tone;
};

constexpr std::array<StatusLook, 10> kLooks = {{
    {"idle", "circle", "Not run", "muted-foreground"},
    {"queued", "circle", "Waiting", "muted-foreground"},
    {"running", "", "Running", "foreground"},
    {"done", "circle-check", "Done", "success"},
    {"cached", "circle-check", "Up to date", "muted-foreground"},
    {"skipped", "circle-slash", "Skipped", "muted-foreground"},
    {"error", "circle-x", "Failed", "destructive"},
    {"failedBefore", "circle-x", "Failed last time", "destructive"},
    {"fromLastRun", "history", "From last run", "muted-foreground"},
    {"changed", "refresh-cw", "Changed since the last run", "muted-foreground"},
}};

QWidget* StatusIcon(const QString& state, QWidget* parent) {
  if (state == QStringLiteral("running")) {
    auto* spinner = new Spinner(parent);
    spinner->setToolTip(QStringLiteral("Running"));
    spinner->setAccessibleName(QStringLiteral("Running"));
    return spinner;
  }
  for (const StatusLook& look : kLooks) {
    if (state != QLatin1String(look.state)) continue;
    auto* icon = new QLabel(parent);
    QColor tone = App::Get().Token(QLatin1String(look.tone));
    if (state == QStringLiteral("failedBefore")) tone.setAlphaF(0.6F);
    icon->setPixmap(App::Get().Glyph(QLatin1String(look.icon), tone, kIcon));
    icon->setToolTip(QLatin1String(look.word));
    icon->setAccessibleName(QLatin1String(look.word));
    icon->setFixedSize(kIcon, kIcon);
    return icon;
  }
  return new QWidget(parent);
}

void Select(const QString& id) {
  rust::Vec<rust::String> ids;
  ids.push_back(ToRust(id));
  App::Get().session().select(std::move(ids));
}

QWidget* NodeColumn(const bridge::RunPanelView& view, QWidget* parent) {
  auto* box = new QWidget(parent);
  auto* column = new QVBoxLayout(box);
  column->setContentsMargins(8, 8, 8, 8);
  column->setSpacing(2);
  auto* header = new QHBoxLayout();
  auto* title = MakeLabel(ToQ(view.title), nullptr, box);
  QFont font = title->font();
  font.setWeight(QFont::Medium);
  title->setFont(font);
  header->addWidget(title, 1);
  if (view.running) {
    auto* stop = MakeButton(QStringLiteral("Stop"), "outline",
                            QStringLiteral("square"), box);
    QObject::connect(stop, &QPushButton::clicked, box, [] {
      try {
        App::Get().session().menu_command(ToRust(QStringLiteral("run:cancel")));
      } catch (const rust::Error& error) {
        App::Get().ShowError(QString::fromUtf8(error.what()));
      }
    });
    header->addWidget(stop);
  }
  column->addLayout(header);
  if (!view.failure.empty()) {
    QString text;
    if (view.failed.empty()) {
      text = ToQ(view.failure).toHtmlEscaped();
    } else {
      QStringList links;
      for (const auto& node : view.failed) {
        links << QStringLiteral("<a href=\"%1\">%2</a>")
                     .arg(ToQ(node.value).toHtmlEscaped(),
                          ToQ(node.label).toHtmlEscaped());
      }
      text = view.failed.size() == 1
                 ? QStringLiteral("%1 failed").arg(links.first())
                 : QStringLiteral("%1 nodes failed: %2")
                       .arg(view.failed.size())
                       .arg(links.join(QStringLiteral(", ")));
      if (view.skipped > 0)
        text += QStringLiteral(", %1 skipped").arg(view.skipped);
    }
    auto* failure = MakeLabel(text, "error", box);
    failure->setObjectName(QStringLiteral("RunFailure"));
    failure->setWordWrap(true);
    failure->setTextFormat(Qt::RichText);
    QObject::connect(failure, &QLabel::linkActivated, box, &Select);
    column->addWidget(failure);
  }
  if (view.rows.empty()) {
    auto* empty =
        MakeLabel(QStringLiteral("Press Run to see how each node is doing."),
                  "muted", box);
    empty->setWordWrap(true);
    column->addWidget(empty);
  }
  for (const auto& row : view.rows) {
    auto* button = new QPushButton(box);
    button->setObjectName(QStringLiteral("RunRow"));
    button->setProperty("selected", row.selected);
    button->setAccessibleName(ToQ(row.name));
    auto* rows = new QVBoxLayout(button);
    rows->setContentsMargins(8, 4, 8, 4);
    rows->setSpacing(2);
    auto* line = new QHBoxLayout();
    line->setSpacing(8);
    line->addWidget(StatusIcon(ToQ(row.state), button));
    auto* name = MakeLabel(ToQ(row.name), nullptr, button);
    line->addWidget(name, 1);
    rows->addLayout(line);
    if (row.progress >= 0) {
      auto* progress_row = new QHBoxLayout();
      progress_row->setContentsMargins(24, 0, 0, 0);
      auto* bar = new QProgressBar(button);
      bar->setRange(0, 100);
      bar->setValue(static_cast<int>(std::lround(row.progress * 100)));
      bar->setTextVisible(false);
      bar->setFixedHeight(4);
      progress_row->addWidget(bar, 1);
      progress_row->addWidget(
          MakeLabel(QStringLiteral("%1%").arg(std::lround(row.progress * 100)),
                    "muted", button));
      rows->addLayout(progress_row);
    }
    button->setMinimumHeight(rows->sizeHint().height());
    const QString id = ToQ(row.id);
    QObject::connect(button, &QPushButton::clicked, box, [id] { Select(id); });
    column->addWidget(button);
  }
  column->addStretch(1);
  return box;
}

QWidget* OutputsColumn(const bridge::RunPanelView& view, QWidget* parent) {
  auto* box = new QWidget(parent);
  auto* column = new QVBoxLayout(box);
  column->setContentsMargins(12, 12, 12, 12);
  column->setSpacing(10);
  auto* header = new QHBoxLayout();
  auto* title = MakeLabel(view.filter_id.empty() ? QStringLiteral("Outputs")
                                                 : QStringLiteral("Artifacts"),
                          nullptr, box);
  QFont font = title->font();
  font.setWeight(QFont::Medium);
  title->setFont(font);
  header->addWidget(title, 1);
  auto* folder = MakeButton(QStringLiteral("Output folder"), "outline",
                            QStringLiteral("folder-open"), box);
  QObject::connect(folder, &QPushButton::clicked, box, [] {
    try {
      App::Get().session().reveal_outputs();
    } catch (const rust::Error& error) {
      App::Get().ShowError(QString::fromUtf8(error.what()));
    }
  });
  header->addWidget(folder);
  column->addLayout(header);
  if (!view.filter_id.empty()) {
    const bridge::ReplayView replay = App::Get().session().replay(
        view.filter_id, ToRust(QStringLiteral("replay")));
    if (replay.can_replay || !replay.trace_id.empty()) {
      auto* buttons = new QHBoxLayout();
      buttons->setSpacing(8);
      AddReplayButtons(buttons, ToQ(view.filter_id), replay.can_replay,
                       !replay.trace_id.empty(), box);
      buttons->addStretch(1);
      column->addLayout(buttons);
    }
  }
  if (view.artifacts.empty()) {
    column->addWidget(MakeLabel(QStringLiteral("Nothing yet."), "muted", box));
  }
  for (const auto& artifact : view.artifacts) {
    const QString kind = ToQ(artifact.kind);
    const QString file = ToQ(artifact.file);
    const QString name = ToQ(artifact.name);
    if (kind == QStringLiteral("image")) {
      column->addWidget(new ImagePreview(file, name, kPreviewWidth, box));
    } else if (kind == QStringLiteral("video") ||
               kind == QStringLiteral("audio")) {
      column->addWidget(
          new FilePlayer(file, name, kind == QStringLiteral("video"), box));
    }
    auto* link = MakeButton(ToQ(artifact.name), "link", QString(), box);
    link->setObjectName(QStringLiteral("ArtifactLink"));
    const QString path = ToQ(artifact.path);
    QObject::connect(link, &QPushButton::clicked, box, [path] {
      try {
        App::Get().session().reveal_file(ToRust(path));
      } catch (const rust::Error& error) {
        App::Get().ShowError(QString::fromUtf8(error.what()));
      }
    });
    column->addWidget(link);
  }
  column->addStretch(1);
  return box;
}

}

RunPanel::RunPanel(QWidget* parent)
    : QWidget(parent),
      nodes_(new QScrollArea(this)),
      log_header_(new QWidget(this)),
      log_(new QPlainTextEdit(this)),
      outputs_(new QScrollArea(this)) {
  setObjectName(QStringLiteral("Panel"));
  setAttribute(Qt::WA_StyledBackground);
  auto* layout = new QHBoxLayout(this);
  layout->setContentsMargins(0, 0, 0, 0);
  auto* split = new QSplitter(Qt::Horizontal, this);
  for (QScrollArea* scroll : {nodes_, outputs_}) {
    scroll->setWidgetResizable(true);
    scroll->setFrameShape(QFrame::NoFrame);
    scroll->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
  }
  auto* logs = new QWidget(split);
  auto* log_column = new QVBoxLayout(logs);
  log_column->setContentsMargins(0, 0, 0, 0);
  log_column->setSpacing(0);
  log_header_->setObjectName(QStringLiteral("TimelineToolbar"));
  log_header_->setAttribute(Qt::WA_StyledBackground);
  auto* header = new QHBoxLayout(log_header_);
  header->setContentsMargins(12, 4, 8, 4);
  auto* scope = MakeLabel(QStringLiteral("All logs"), "muted", log_header_);
  scope->setObjectName(QStringLiteral("LogScope"));
  header->addWidget(scope, 1);
  auto* clear = MakeButton(QStringLiteral("Clear"), "ghost",
                           QStringLiteral("trash-2"), log_header_);
  connect(clear, &QPushButton::clicked, this,
          [] { App::Get().session().clear_run_log(); });
  header->addWidget(clear);
  log_column->addWidget(log_header_);
  log_->setObjectName(QStringLiteral("RunLog"));
  log_->setReadOnly(true);
  log_->setFrameShape(QFrame::NoFrame);
  log_->setAccessibleName(QStringLiteral("Run log"));
  QFont mono(QStringLiteral("JetBrains Mono"));
  mono.setPixelSize(12);
  log_->setFont(mono);
  log_column->addWidget(log_, 1);
  nodes_->setMinimumWidth(kNodesWidth);
  outputs_->setMinimumWidth(kOutputsWidth);
  split->addWidget(nodes_);
  split->addWidget(logs);
  split->addWidget(outputs_);
  split->setStretchFactor(0, 2);
  split->setStretchFactor(1, 4);
  split->setStretchFactor(2, 2);
  layout->addWidget(split);
  for (auto signal : {&EventHub::RunChanged, &EventHub::SelectionChanged,
                      &EventHub::CompiledChanged, &EventHub::DocumentChanged,
                      &EventHub::ProjectChanged}) {
    connect(EventHub::Instance(), signal, this, &RunPanel::Rebuild);
  }
  Rebuild();
}

RunPanel::~RunPanel() = default;

void RunPanel::Rebuild() {
  const bridge::RunPanelView view = App::Get().session().run_panel();
  if (last_ != nullptr && *last_ == view) return;
  last_ = std::make_unique<bridge::RunPanelView>(view);
  const int node_scroll = nodes_->verticalScrollBar()->value();
  nodes_->setWidget(NodeColumn(view, nodes_));
  nodes_->verticalScrollBar()->setValue(node_scroll);
  QString outputs_key = view.running ? QStringLiteral("running") : QString();
  for (const auto& artifact : view.artifacts) {
    const QString file = ToQ(artifact.file);
    outputs_key +=
        QLatin1Char('\n') + ToQ(artifact.kind) + QLatin1Char(' ') + file +
        QLatin1Char(' ') +
        QString::number(QFileInfo(file).lastModified().toMSecsSinceEpoch());
  }
  if (outputs_key != outputs_key_ || outputs_->widget() == nullptr) {
    outputs_key_ = outputs_key;
    outputs_->setWidget(OutputsColumn(view, outputs_));
  }
  if (auto* scope =
          log_header_->findChild<QLabel*>(QStringLiteral("LogScope"))) {
    scope->setText(view.filtered ? QStringLiteral("Logs for the selected node")
                                 : QStringLiteral("All logs"));
  }
  const QString key = ToQ(view.filter_id);
  const std::uint64_t newest = view.logs.empty() ? 0 : view.logs.back().id;
  const bool append = key == log_scope_ && newest >= last_log_ && last_log_ > 0;
  QScrollBar* bar = log_->verticalScrollBar();
  const bool at_end = bar->value() >= bar->maximum() - 4;
  if (!append) log_->clear();
  const QString muted =
      App::Get().Token(QStringLiteral("muted-foreground")).name();
  const QString error = App::Get().Token(QStringLiteral("destructive")).name();
  for (const auto& line : view.logs) {
    if (append && line.id <= last_log_) continue;
    const QString level = ToQ(line.level);
    const QString time = QDateTime::fromMSecsSinceEpoch(line.at).toString(
        QStringLiteral("HH:mm:ss"));
    const QString colour =
        level == QStringLiteral("error")
            ? error
            : (level == QStringLiteral("warn") ? muted : QString());
    const QString message = ToQ(line.message).toHtmlEscaped();
    log_->appendHtml(
        QStringLiteral("<span style=\"color:%1\">%2</span> %3")
            .arg(muted, time,
                 colour.isEmpty()
                     ? message
                     : QStringLiteral("<span style=\"color:%1\">%2</span>")
                           .arg(colour, message)));
  }
  last_log_ = newest;
  log_scope_ = key;
  if (at_end || !append) bar->setValue(bar->maximum());
}

}
