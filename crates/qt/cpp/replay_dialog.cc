#include "replay_dialog.h"

#include <QButtonGroup>
#include <QGuiApplication>
#include <QHBoxLayout>
#include <QHeaderView>
#include <QLabel>
#include <QListWidget>
#include <QPainter>
#include <QPainterPath>
#include <QPushButton>
#include <QScreen>
#include <QScrollArea>
#include <QSplitter>
#include <QStackedWidget>
#include <QTableWidget>
#include <QVBoxLayout>
#include <algorithm>
#include <array>
#include <utility>
#include <vector>

#include "app.h"
#include "file_player.h"
#include "opsis-qt/src/bridge.rs.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr double kScreenShare = 0.94;
constexpr int kActionsWidth = 340;
constexpr int kTraceListMin = 260;
constexpr int kTraceListMax = 340;
constexpr int kTabHeight = 192;
constexpr int kIndent = 16;
constexpr int kTimeWidth = 52;
constexpr int kFollowMs = 120;
constexpr double kBoxRadius = 6;
constexpr double kBoxStroke = 1.0 / 480;
constexpr double kPointStroke = 1.0 / 960;
constexpr double kTracePoint = 1.0 / 160;
constexpr int kBoxFillAlpha = 26;
constexpr int kIcon = 16;
constexpr int kMonoSize = 12;

QFont Mono() {
  QFont mono(QStringLiteral("JetBrains Mono"));
  mono.setPixelSize(kMonoSize);
  return mono;
}

QLabel* Wrapped(const QString& text, const char* role, QWidget* parent) {
  auto* label = MakeLabel(text, role, parent);
  label->setWordWrap(true);
  return label;
}

QWidget* EmptyState(const QString& title, const QString& about,
                    QWidget* parent) {
  auto* box = new QFrame(parent);
  box->setProperty("role", "alert");
  auto* column = new QVBoxLayout(box);
  column->setContentsMargins(16, 24, 16, 24);
  column->setSpacing(4);
  auto* heading = MakeLabel(title, "title-small", box);
  heading->setAlignment(Qt::AlignCenter);
  column->addWidget(heading);
  auto* body = Wrapped(about, "muted", box);
  body->setAlignment(Qt::AlignCenter);
  column->addWidget(body);
  return box;
}

QWidget* Tabs(QWidget* parent,
              const std::vector<std::pair<QString, QWidget*>>& pages,
              int current, QStackedWidget** stack_out = nullptr) {
  auto* box = new QWidget(parent);
  auto* column = new QVBoxLayout(box);
  column->setContentsMargins(0, 0, 0, 0);
  column->setSpacing(8);
  auto* bar = new QFrame(box);
  bar->setObjectName(QStringLiteral("FormTabs"));
  auto* row = new QHBoxLayout(bar);
  row->setContentsMargins(3, 3, 3, 3);
  row->setSpacing(3);
  auto* stack = new QStackedWidget(box);
  auto* group = new QButtonGroup(box);
  int index = 0;
  for (const auto& [name, page] : pages) {
    auto* tab = MakeButton(name, "tab", QString(), bar);
    tab->setCheckable(true);
    group->addButton(tab, index);
    row->addWidget(tab);
    stack->addWidget(page);
    index += 1;
  }
  row->addStretch(1);
  QObject::connect(group, &QButtonGroup::idClicked, stack,
                   &QStackedWidget::setCurrentIndex);
  if (QAbstractButton* chosen = group->button(current))
    chosen->setChecked(true);
  stack->setCurrentIndex(current);
  bar->setSizePolicy(QSizePolicy::Maximum, QSizePolicy::Fixed);
  column->addWidget(bar, 0, Qt::AlignLeft);
  column->addWidget(stack, 1);
  if (stack_out != nullptr) *stack_out = stack;
  return box;
}

QWidget* Scrolled(QWidget* inner, QWidget* parent) {
  auto* scroll = new QScrollArea(parent);
  scroll->setWidgetResizable(true);
  scroll->setFrameShape(QFrame::NoFrame);
  scroll->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
  scroll->setWidget(inner);
  scroll->setFixedHeight(kTabHeight);
  return scroll;
}

void SelectNode(const QString& node) {
  if (node.isEmpty()) return;
  rust::Vec<rust::String> ids;
  ids.push_back(ToRust(node));
  App::Get().session().select(std::move(ids));
}

QSize ScreenSize(const QWidget* widget) {
  const QScreen* screen = widget != nullptr && widget->screen() != nullptr
                              ? widget->screen()
                              : QGuiApplication::primaryScreen();
  const QRect area = screen->availableGeometry();
  return {static_cast<int>(area.width() * kScreenShare),
          static_cast<int>(area.height() * kScreenShare)};
}

}

TraceShot::TraceShot(QWidget* parent) : QWidget(parent) {
  setObjectName(QStringLiteral("TraceShot"));
  setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Expanding);
  setMinimumHeight(120);
}

void TraceShot::Show(const QString& file, QSize viewport, const QRectF& box,
                     const QPointF& point, bool has_box, bool has_point) {
  picture_ = QPixmap(file);
  viewport_ = viewport.isEmpty() ? picture_.size() : viewport;
  box_ = box;
  point_ = point;
  has_box_ = has_box;
  has_point_ = has_point;
  update();
}

void TraceShot::Clear() {
  picture_ = QPixmap();
  update();
}

void TraceShot::paintEvent(QPaintEvent*) {
  if (picture_.isNull() || viewport_.isEmpty()) return;
  QPainter painter(this);
  painter.setRenderHint(QPainter::Antialiasing);
  painter.setRenderHint(QPainter::SmoothPixmapTransform);
  const QSize fitted = viewport_.scaled(size(), Qt::KeepAspectRatio);
  const QRectF target(QPointF((width() - fitted.width()) / 2.0,
                              (height() - fitted.height()) / 2.0),
                      QSizeF(fitted));
  QPainterPath clip;
  clip.addRoundedRect(target, kBoxRadius, kBoxRadius);
  painter.setClipPath(clip);
  painter.fillRect(target, Qt::black);
  const QSize shown = picture_.size().scaled(fitted, Qt::KeepAspectRatio);
  painter.drawPixmap(
      QRectF(QPointF(target.x() + (target.width() - shown.width()) / 2.0,
                     target.y() + (target.height() - shown.height()) / 2.0),
             QSizeF(shown)),
      picture_, QRectF(picture_.rect()));
  const double scale = target.width() / viewport_.width();
  const double wide = viewport_.width();
  const QColor primary = App::Get().Token(QStringLiteral("primary"));
  if (has_box_) {
    QColor fill = primary;
    fill.setAlpha(kBoxFillAlpha);
    painter.setBrush(fill);
    painter.setPen(QPen(primary, std::max(1.0, wide * kBoxStroke * scale)));
    const QRectF area(target.x() + box_.x() * scale,
                      target.y() + box_.y() * scale, box_.width() * scale,
                      box_.height() * scale);
    painter.drawRoundedRect(area, kBoxRadius * scale, kBoxRadius * scale);
  }
  if (has_point_) {
    painter.setBrush(primary);
    painter.setPen(QPen(App::Get().Token(QStringLiteral("background")),
                        std::max(1.0, wide * kPointStroke * scale)));
    const double radius = wide * kTracePoint * scale;
    painter.drawEllipse(QPointF(target.x() + point_.x() * scale,
                                target.y() + point_.y() * scale),
                        radius, radius);
  }
}

TracePanel::TracePanel(const QString& trace_id, QWidget* parent)
    : QWidget(parent),
      view_(std::make_unique<bridge::TraceView>(
          App::Get().session().trace(ToRust(trace_id)))) {
  setObjectName(QStringLiteral("TracePanel"));
  Build();
}

TracePanel::~TracePanel() = default;

void TracePanel::Build() {
  auto* layout = new QHBoxLayout(this);
  layout->setContentsMargins(0, 0, 0, 0);
  layout->setSpacing(16);
  if (!view_->error.empty()) {
    layout->addWidget(EmptyState(QStringLiteral("Couldn't open the trace"),
                                 ToQ(view_->error), this));
    return;
  }
  list_ = new QListWidget(this);
  list_->setObjectName(QStringLiteral("TraceActions"));
  list_->setMinimumWidth(kTraceListMin);
  list_->setMaximumWidth(kTraceListMax);
  list_->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
  list_->setAccessibleName(QStringLiteral("Trace actions"));
  const QColor muted = App::Get().Token(QStringLiteral("muted-foreground"));
  const QColor danger = App::Get().Token(QStringLiteral("destructive"));
  for (const auto& action : view_->actions) {
    auto* item = new QListWidgetItem(list_);
    auto* row = new QWidget(list_);
    auto* line = new QHBoxLayout(row);
    line->setContentsMargins(8 + static_cast<int>(action.depth) * kIndent, 4, 8,
                             4);
    line->setSpacing(6);
    auto* icon = new QLabel(row);
    icon->setFixedSize(kIcon, kIcon);
    if (action.failed) {
      icon->setPixmap(App::Get()
                          .Icon(QStringLiteral("circle-x"), danger, kIcon)
                          .pixmap(kIcon, kIcon));
    } else if (action.group) {
      icon->setPixmap(App::Get()
                          .Icon(QStringLiteral("folder"), muted, kIcon)
                          .pixmap(kIcon, kIcon));
    } else if (action.has_point) {
      icon->setPixmap(
          App::Get()
              .Icon(QStringLiteral("mouse-pointer-click"), muted, kIcon)
              .pixmap(kIcon, kIcon));
    }
    line->addWidget(icon, 0, Qt::AlignTop);
    auto* words = new QVBoxLayout();
    words->setSpacing(0);
    auto* title = new ClampedLabel(ToQ(action.title), 1, row);
    if (action.group) {
      QFont bold = title->font();
      bold.setWeight(QFont::Medium);
      title->setFont(bold);
    }
    words->addWidget(title);
    if (!action.target.empty()) {
      auto* target = new ClampedLabel(ToQ(action.target), 1, row);
      target->setProperty("role", "muted");
      UseFont(target, Mono());
      words->addWidget(target);
    }
    line->addLayout(words, 1);
    line->addWidget(MakeLabel(ToQ(action.length), "small", row), 0,
                    Qt::AlignTop);
    item->setSizeHint(row->sizeHint());
    list_->setItemWidget(item, row);
  }
  connect(list_, &QListWidget::currentRowChanged, this, [this](int row) {
    if (row < 0 || row == selected_) return;
    selected_ = row;
    ShowSelected();
    SelectNode(ToQ(view_->actions.at(static_cast<std::size_t>(row)).node_id));
  });
  layout->addWidget(list_);
  auto* right = new QWidget(this);
  auto* column = new QVBoxLayout(right);
  column->setContentsMargins(0, 0, 0, 0);
  column->setSpacing(12);
  auto* header = new QHBoxLayout();
  header->setSpacing(8);
  title_ = MakeLabel(QString(), "title-small", right);
  title_->setSizePolicy(QSizePolicy::Ignored, QSizePolicy::Preferred);
  header->addWidget(title_, 1);
  badge_ = MakeLabel(QString(), "badge", right);
  header->addWidget(badge_);
  moments_ = new Segmented(right);
  moments_->SetChoices({{QStringLiteral("before"), QStringLiteral("Before")},
                        {QStringLiteral("after"), QStringLiteral("After")}});
  moments_->SetValue(moment_);
  connect(moments_, &Segmented::Changed, this, [this](const QString& value) {
    moment_ = value;
    ShowSelected();
  });
  header->addWidget(moments_);
  column->addLayout(header);
  shot_ = new TraceShot(right);
  column->addWidget(shot_, 1);
  auto* empty =
      EmptyState(QStringLiteral("No screenshots"),
                 QStringLiteral("This take has no screenshots."), right);
  empty->setObjectName(QStringLiteral("NoScreenshots"));
  column->addWidget(empty, 1);
  empty->setVisible(false);
  call_ = Wrapped(QString(), nullptr, right);
  call_->setObjectName(QStringLiteral("TraceCall"));
  UseFont(call_, Mono());
  call_->setTextFormat(Qt::RichText);
  call_->setAlignment(Qt::AlignTop | Qt::AlignLeft);
  call_->setContentsMargins(8, 8, 8, 8);
  call_->setTextInteractionFlags(Qt::TextSelectableByMouse);
  console_ = new QWidget(right);
  network_ = new QWidget(right);
  auto* errors = new QWidget(right);
  auto* error_rows = new QVBoxLayout(errors);
  error_rows->setContentsMargins(4, 4, 4, 4);
  error_rows->setSpacing(2);
  if (view_->errors.empty()) {
    error_rows->addWidget(
        MakeLabel(QStringLiteral("No errors."), "muted", errors));
  }
  for (const auto& error : view_->errors) {
    auto* button =
        MakeButton(ToQ(error.at) + QStringLiteral("  ") + ToQ(error.text),
                   "ghost", QStringLiteral("circle-alert"), errors);
    button->setObjectName(QStringLiteral("TraceError"));
    UseFont(button, Mono());
    button->setStyleSheet(QStringLiteral("text-align: left;"));
    const int target = error.action;
    connect(button, &QPushButton::clicked, this,
            [this, target] { Select(target); });
    error_rows->addWidget(button);
  }
  error_rows->addStretch(1);
  const auto counted = [](const QString& name, std::size_t count) {
    return count == 0 ? name
                      : name + QStringLiteral("  ") + QString::number(count);
  };
  column->addWidget(Tabs(
      right,
      {{QStringLiteral("Call"), Scrolled(call_, right)},
       {counted(QStringLiteral("Console"), view_->warnings),
        Scrolled(console_, right)},
       {QStringLiteral("Network  ") + QString::number(view_->network.size()),
        Scrolled(network_, right)},
       {counted(QStringLiteral("Errors"), view_->errors.size()),
        Scrolled(errors, right)}},
      0));
  layout->addWidget(right, 1);
  Select(view_->picked);
}

void TracePanel::Select(int index) {
  if (list_ == nullptr || index < 0 ||
      index >= static_cast<int>(view_->actions.size())) {
    return;
  }
  selected_ = index;
  list_->blockSignals(true);
  list_->setCurrentRow(index);
  list_->blockSignals(false);
  ShowSelected();
}

void TracePanel::ShowSelected() {
  if (selected_ < 0) return;
  const auto& action = view_->actions.at(static_cast<std::size_t>(selected_));
  title_->setText(ToQ(action.title));
  badge_->setText(ToQ(action.at) + QStringLiteral(" · ") + ToQ(action.length));
  const int frame =
      moment_ == QStringLiteral("before") ? action.before : action.after;
  auto* empty = findChild<QWidget*>(QStringLiteral("NoScreenshots"));
  if (frame < 0 || frame >= static_cast<int>(view_->frames.size())) {
    shot_->Clear();
    shot_->setVisible(false);
    if (empty != nullptr) empty->setVisible(true);
  } else {
    const auto& picture = view_->frames.at(static_cast<std::size_t>(frame));
    shot_->Show(
        ToQ(picture.file),
        QSize(static_cast<int>(view_->width), static_cast<int>(view_->height)),
        QRectF(action.box_x, action.box_y, action.box_width, action.box_height),
        QPointF(action.point_x, action.point_y), action.has_box,
        action.has_point);
    shot_->setVisible(true);
    if (empty != nullptr) empty->setVisible(false);
  }
  FillTabs();
}

void TracePanel::FillTabs() {
  const auto& action = view_->actions.at(static_cast<std::size_t>(selected_));
  const QString danger = App::Get().Token(QStringLiteral("destructive")).name();
  const QString muted =
      App::Get().Token(QStringLiteral("muted-foreground")).name();
  QStringList call;
  if (!action.error.empty()) {
    call << QStringLiteral(
                "<div style=\"color:%1;white-space:pre-wrap\">%2</div>")
                .arg(danger, ToQ(action.error).toHtmlEscaped());
  }
  if (!action.target.empty()) {
    call << QStringLiteral("<div>%1</div>")
                .arg(ToQ(action.target).toHtmlEscaped());
  }
  for (const auto& log : action.logs) {
    call << QStringLiteral(
                "<div style=\"color:%1;white-space:pre-wrap\">%2</div>")
                .arg(muted, ToQ(log).toHtmlEscaped());
  }
  call_->setText(call.join(QString()));
  const auto within = [&action](double time) {
    return time >= action.start && time <= action.end;
  };
  const QColor highlight = App::Get().Token(QStringLiteral("muted"));
  delete console_->layout();
  for (QWidget* old : console_->findChildren<QWidget*>(
           QString(), Qt::FindDirectChildrenOnly)) {
    delete old;
  }
  auto* lines = new QVBoxLayout(console_);
  lines->setContentsMargins(0, 0, 0, 0);
  lines->setSpacing(0);
  for (const auto& entry : view_->console) {
    auto* row = new QFrame(console_);
    row->setObjectName(QStringLiteral("TraceConsoleRow"));
    if (within(entry.time)) {
      row->setAutoFillBackground(true);
      QPalette palette = row->palette();
      palette.setColor(QPalette::Window, highlight);
      row->setPalette(palette);
    }
    auto* line = new QHBoxLayout(row);
    line->setContentsMargins(8, 4, 8, 4);
    line->setSpacing(8);
    auto* at = MakeLabel(ToQ(entry.at), "muted", row);
    UseFont(at, Mono());
    at->setFixedWidth(kTimeWidth + 8);
    line->addWidget(at, 0, Qt::AlignTop);
    const QString level = ToQ(entry.level);
    line->addWidget(
        MakeLabel(level,
                  level == QStringLiteral("error")     ? "error"
                  : level == QStringLiteral("warning") ? "badge-secondary"
                                                       : "badge",
                  row),
        0, Qt::AlignTop);
    auto* text = Wrapped(ToQ(entry.text), nullptr, row);
    UseFont(text, Mono());
    text->setTextInteractionFlags(Qt::TextSelectableByMouse);
    line->addWidget(text, 1);
    lines->addWidget(row);
  }
  lines->addStretch(1);
  delete network_->layout();
  for (QWidget* old : network_->findChildren<QWidget*>(
           QString(), Qt::FindDirectChildrenOnly)) {
    delete old;
  }
  auto* holder = new QVBoxLayout(network_);
  holder->setContentsMargins(0, 0, 0, 0);
  auto* table =
      new QTableWidget(static_cast<int>(view_->network.size()), 7, network_);
  table->setObjectName(QStringLiteral("TraceNetwork"));
  table->setHorizontalHeaderLabels(
      {QStringLiteral("Time"), QStringLiteral("Method"),
       QStringLiteral("Status"), QStringLiteral("URL"), QStringLiteral("Type"),
       QStringLiteral("Size"), QStringLiteral("Duration")});
  table->verticalHeader()->setVisible(false);
  table->setEditTriggers(QAbstractItemView::NoEditTriggers);
  table->setSelectionMode(QAbstractItemView::NoSelection);
  table->setShowGrid(false);
  table->horizontalHeader()->setSectionResizeMode(3, QHeaderView::Stretch);
  int row = 0;
  for (const auto& request : view_->network) {
    const std::array<QString, 7> cells = {
        ToQ(request.at),    ToQ(request.method), ToQ(request.status),
        ToQ(request.url),   ToQ(request.kind),   ToQ(request.size),
        ToQ(request.length)};
    for (int column = 0; column < 7; ++column) {
      auto* cell =
          new QTableWidgetItem(cells.at(static_cast<std::size_t>(column)));
      if (column == 3) cell->setToolTip(cells.at(3));
      if (column >= 5)
        cell->setTextAlignment(Qt::AlignRight | Qt::AlignVCenter);
      if (column == 2 && request.failed) {
        cell->setForeground(App::Get().Token(QStringLiteral("destructive")));
      }
      if (within(request.start)) cell->setBackground(highlight);
      table->setItem(row, column, cell);
    }
    row += 1;
  }
  table->resizeColumnsToContents();
  table->horizontalHeader()->setSectionResizeMode(3, QHeaderView::Stretch);
  holder->addWidget(table);
}

ReplayDialog::ReplayDialog(QWidget* parent, const bridge::ReplayView& view)
    : QDialog(parent) {
  setObjectName(QStringLiteral("ReplayDialog"));
  setWindowTitle(ToQ(view.title));
  setAttribute(Qt::WA_DeleteOnClose);
  resize(ScreenSize(parent));
  auto* layout = new QVBoxLayout(this);
  layout->setContentsMargins(24, 20, 24, 20);
  layout->setSpacing(16);
  auto* header = new QVBoxLayout();
  header->setSpacing(4);
  header->addWidget(MakeLabel(ToQ(view.title), "title", this));
  header->addWidget(MakeLabel(ToQ(view.description), "muted", this));
  layout->addLayout(header);
  QWidget* footage = Footage(view);
  const QString trace = ToQ(view.trace_id);
  if (trace.isEmpty()) {
    layout->addWidget(footage, 1);
  } else {
    auto* trace_page = new TracePanel(trace, this);
    layout->addWidget(Tabs(this,
                           {{QStringLiteral("Replay"), footage},
                            {QStringLiteral("Trace"), trace_page}},
                           ToQ(view.tab) == QStringLiteral("trace") ? 1 : 0),
                      1);
  }
  follow_.setInterval(kFollowMs);
  connect(&follow_, &QTimer::timeout, this, &ReplayDialog::Follow);
  if (!player_.isNull()) follow_.start();
}

QWidget* ReplayDialog::Footage(const bridge::ReplayView& view) {
  auto* page = new QWidget(this);
  auto* columns = new QHBoxLayout(page);
  columns->setContentsMargins(0, 0, 0, 0);
  columns->setSpacing(16);
  auto* left = new QWidget(page);
  auto* column = new QVBoxLayout(left);
  column->setContentsMargins(0, 0, 0, 0);
  column->setSpacing(12);
  if (!view.alert.empty()) {
    auto* alert = new QFrame(left);
    alert->setObjectName(QStringLiteral("ReplayAlert"));
    alert->setProperty("role", "alert");
    alert->setProperty("tone", "destructive");
    auto* words = new QVBoxLayout(alert);
    words->setContentsMargins(12, 10, 12, 10);
    words->setSpacing(2);
    words->addWidget(MakeLabel(QStringLiteral("The last take failed"),
                               "error-title", alert));
    words->addWidget(Wrapped(ToQ(view.alert), nullptr, alert));
    column->addWidget(alert);
  }
  const QString video = ToQ(view.video_file);
  const QString image = ToQ(view.image_file);
  if (!video.isEmpty()) {
    player_ = new FilePlayer(video, QStringLiteral("Recording"), true, left);
    column->addWidget(player_);
  } else if (!image.isEmpty()) {
    column->addWidget(
        new ImagePreview(image, QStringLiteral("Captured still"),
                         ScreenSize(this).width() - kActionsWidth - 96, left));
  } else {
    column->addWidget(EmptyState(QStringLiteral("No footage yet"),
                                 QStringLiteral("Run this node to record it."),
                                 left));
  }
  const QString reveal = ToQ(view.reveal);
  if (!reveal.isEmpty()) {
    auto* button = MakeButton(QStringLiteral("Reveal file"), "outline",
                              QStringLiteral("folder-open"), left);
    connect(button, &QPushButton::clicked, this, [reveal] {
      try {
        App::Get().session().reveal_file(ToRust(reveal));
      } catch (const rust::Error& error) {
        App::Get().ShowError(QString::fromUtf8(error.what()));
      }
    });
    column->addWidget(button, 0, Qt::AlignLeft);
  }
  column->addStretch(1);
  columns->addWidget(left, 1);
  actions_ = new QListWidget(page);
  actions_->setObjectName(QStringLiteral("ReplayActions"));
  actions_->setAccessibleName(QStringLiteral("Recorded actions"));
  actions_->setFixedWidth(kActionsWidth);
  actions_->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
  std::vector<QString> nodes;
  for (const auto& action : view.actions) {
    times_.push_back(action.at);
    nodes.push_back(ToQ(action.node_id));
    auto* item = new QListWidgetItem(actions_);
    auto* row = new QWidget(actions_);
    auto* line = new QHBoxLayout(row);
    line->setContentsMargins(6, 4, 6, 4);
    line->setSpacing(8);
    auto* time = MakeLabel(ToQ(action.time), "muted", row);
    time->setFixedWidth(kTimeWidth);
    time->setAlignment(Qt::AlignRight | Qt::AlignVCenter);
    line->addWidget(time);
    line->addWidget(MakeLabel(ToQ(action.kind), "badge", row));
    auto* words = new QVBoxLayout();
    words->setSpacing(0);
    words->addWidget(new ClampedLabel(ToQ(action.label), 1, row));
    if (!action.selector.empty()) {
      auto* selector = new ClampedLabel(ToQ(action.selector), 1, row);
      selector->setProperty("role", "muted");
      UseFont(selector, Mono());
      words->addWidget(selector);
    }
    line->addLayout(words, 1);
    item->setSizeHint(row->sizeHint());
    actions_->setItemWidget(item, row);
  }
  connect(actions_, &QListWidget::itemClicked, this,
          [this, nodes](QListWidgetItem* item) {
            const int row = actions_->row(item);
            if (row < 0) return;
            if (!player_.isNull()) {
              App::Get().session().player_seek(
                  player_->id(),
                  times_.at(static_cast<std::size_t>(row)) / 1000.0);
            }
            SelectNode(nodes.at(static_cast<std::size_t>(row)));
          });
  columns->addWidget(actions_);
  return page;
}

void ReplayDialog::Follow() {
  if (player_.isNull() || actions_ == nullptr) return;
  const double ms =
      App::Get().session().player_view(player_->id()).position * 1000.0;
  const auto after = std::upper_bound(times_.begin(), times_.end(), ms);
  const int row = static_cast<int>(after - times_.begin()) - 1;
  if (row == actions_->currentRow()) return;
  actions_->blockSignals(true);
  actions_->setCurrentRow(row);
  actions_->blockSignals(false);
}

void AddReplayButtons(QLayout* row, const QString& node, bool can_replay,
                      bool has_trace, QWidget* parent) {
  const auto ask = [node](const char* tab) {
    try {
      App::Get().session().show_replay(ToRust(node),
                                       ToRust(QString::fromLatin1(tab)));
    } catch (const rust::Error& error) {
      App::Get().ShowError(QString::fromUtf8(error.what()));
    }
  };
  if (can_replay) {
    auto* replay = MakeButton(QStringLiteral("Replay"), "outline",
                              QStringLiteral("film"), parent);
    QObject::connect(replay, &QPushButton::clicked, parent,
                     [ask] { ask("replay"); });
    row->addWidget(replay);
  }
  if (has_trace) {
    auto* trace = MakeButton(QStringLiteral("Trace"), "outline",
                             QStringLiteral("file-search"), parent);
    QObject::connect(trace, &QPushButton::clicked, parent,
                     [ask] { ask("trace"); });
    row->addWidget(trace);
  }
}

void ReplayDialog::Open(QWidget* parent, const bridge::ReplayView& view) {
  if (!view.shown) return;
  for (auto* open : parent->findChildren<ReplayDialog*>(
           QString(), Qt::FindDirectChildrenOnly)) {
    open->close();
  }
  auto* dialog = new ReplayDialog(parent, view);
  dialog->open();
}

}
