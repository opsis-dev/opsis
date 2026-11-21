#include "file_player.h"

#include <QDialog>
#include <QGuiApplication>
#include <QHBoxLayout>
#include <QKeyEvent>
#include <QLayout>
#include <QMenu>
#include <QMouseEvent>
#include <QPainter>
#include <QPainterPath>
#include <QPushButton>
#include <QScreen>
#include <QSlider>
#include <QStackedWidget>
#include <QVBoxLayout>
#include <algorithm>
#include <array>
#include <utility>

#include "app.h"
#include "opsis-qt/src/bridge.rs.h"
#include "wake_hub.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr int kTickMs = 33;
constexpr double kStepSeconds = 5;
constexpr double kNudgeSeconds = 1;
constexpr int kVolumeWidth = 64;
constexpr int kWideEnough = 320;
constexpr double kTrackHeight = 4;
constexpr double kThumb = 10;
constexpr std::array<double, 7> kSpeeds = {0.5, 0.75, 1, 1.25, 1.5, 1.75, 2};
constexpr double kScreenShare = 0.94;
constexpr double kScreenHeightShare = 0.88;

QString SpeedText(double rate) {
  return QString::number(rate, 'g', 3) + QStringLiteral("×");
}

}

PlayerSurface::PlayerSurface(QWidget* parent) : QWidget(parent) {
  setObjectName(QStringLiteral("PlayerSurface"));
  setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Preferred);
  setCursor(Qt::PointingHandCursor);
}

void PlayerSurface::SetFrame(QImage frame) {
  frame_ = std::move(frame);
  update();
}

void PlayerSurface::SetAspect(int width, int height) {
  if (width <= 0 || height <= 0) return;
  if (width == aspect_width_ && height == aspect_height_) return;
  aspect_width_ = width;
  aspect_height_ = height;
  updateGeometry();
}

int PlayerSurface::heightForWidth(int width) const {
  return width * aspect_height_ / std::max(1, aspect_width_);
}

QSize PlayerSurface::sizeHint() const {
  constexpr int kHintWidth = 240;
  return {kHintWidth, heightForWidth(kHintWidth)};
}

void PlayerSurface::paintEvent(QPaintEvent*) {
  QPainter painter(this);
  painter.fillRect(rect(), Qt::black);
  if (frame_.isNull()) return;
  const QSize fitted = frame_.size().scaled(size(), Qt::KeepAspectRatio);
  const QRect target(
      QPoint((width() - fitted.width()) / 2, (height() - fitted.height()) / 2),
      fitted);
  painter.setRenderHint(QPainter::SmoothPixmapTransform);
  painter.drawImage(target, frame_);
}

void PlayerSurface::mousePressEvent(QMouseEvent* event) {
  if (event->button() == Qt::LeftButton) emit Clicked();
}

void PlayerSurface::mouseDoubleClickEvent(QMouseEvent* event) {
  if (event->button() == Qt::LeftButton) emit DoubleClicked();
}

SeekBar::SeekBar(QWidget* parent) : QWidget(parent) {
  setObjectName(QStringLiteral("PlayerSeek"));
  setFocusPolicy(Qt::StrongFocus);
  setCursor(Qt::PointingHandCursor);
  setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Fixed);
  setAccessibleName(QStringLiteral("Seek"));
}

void SeekBar::SetRange(double duration) {
  duration_ = std::max(0.0, duration);
  update();
}

void SeekBar::SetPosition(double seconds) {
  if (dragging_) return;
  position_ = seconds;
  update();
}

double SeekBar::SecondsAt(double x) const {
  const double usable = std::max(1.0, width() - kThumb);
  return std::clamp((x - kThumb / 2) / usable, 0.0, 1.0) * duration_;
}

void SeekBar::paintEvent(QPaintEvent*) {
  QPainter painter(this);
  painter.setRenderHint(QPainter::Antialiasing);
  const double usable = std::max(1.0, width() - kThumb);
  const double share =
      duration_ > 0 ? std::clamp(position_ / duration_, 0.0, 1.0) : 0;
  const double middle = height() / 2.0;
  const QRectF track(kThumb / 2, middle - kTrackHeight / 2, usable,
                     kTrackHeight);
  painter.setPen(Qt::NoPen);
  painter.setBrush(App::Get().Token(QStringLiteral("muted")));
  painter.drawRoundedRect(track, kTrackHeight / 2, kTrackHeight / 2);
  QRectF played = track;
  played.setWidth(usable * share);
  painter.setBrush(App::Get().Token(QStringLiteral("primary")));
  painter.drawRoundedRect(played, kTrackHeight / 2, kTrackHeight / 2);
  if (hasFocus() || underMouse() || dragging_) {
    painter.setBrush(App::Get().Token(QStringLiteral("foreground")));
    painter.drawEllipse(QPointF(kThumb / 2 + usable * share, middle),
                        kThumb / 2, kThumb / 2);
  }
}

void SeekBar::mousePressEvent(QMouseEvent* event) {
  if (event->button() != Qt::LeftButton || duration_ <= 0) return;
  dragging_ = true;
  position_ = SecondsAt(event->position().x());
  update();
  emit Seeked(position_);
}

void SeekBar::mouseMoveEvent(QMouseEvent* event) {
  if (!dragging_) return;
  position_ = SecondsAt(event->position().x());
  update();
  emit Seeked(position_);
}

void SeekBar::mouseReleaseEvent(QMouseEvent* event) {
  if (event->button() != Qt::LeftButton || !dragging_) return;
  dragging_ = false;
  update();
}

void SeekBar::keyPressEvent(QKeyEvent* event) {
  double to = position_;
  switch (event->key()) {
    case Qt::Key_Left:
      to -= kStepSeconds;
      break;
    case Qt::Key_Right:
      to += kStepSeconds;
      break;
    case Qt::Key_Home:
      to = 0;
      break;
    case Qt::Key_End:
      to = duration_;
      break;
    default:
      QWidget::keyPressEvent(event);
      return;
  }
  position_ = std::clamp(to, 0.0, duration_);
  update();
  emit Seeked(position_);
}

FilePlayer::FilePlayer(const QString& file, const QString& label, bool video,
                       QWidget* parent)
    : QFrame(parent),
      id_(App::Get().session().player_open(ToRust(file))),
      video_(video),
      label_(label),
      seek_(new SeekBar(this)),
      play_(
          MakeIconButton(QStringLiteral("play"), QStringLiteral("Play"), this)),
      mute_(MakeIconButton(QStringLiteral("volume-2"), QStringLiteral("Mute"),
                           this)),
      volume_(new QSlider(Qt::Horizontal, this)),
      time_(MakeLabel(QString(), "muted", this)),
      speed_(new QToolButton(this)) {
  setObjectName(video ? QStringLiteral("VideoPlayer")
                      : QStringLiteral("AudioPlayer"));
  setAttribute(Qt::WA_StyledBackground);
  setAccessibleName(label);
  setFocusPolicy(Qt::StrongFocus);
  auto* layout = new QVBoxLayout(this);
  layout->setContentsMargins(0, 0, 0, 0);
  layout->setSpacing(0);
  stage_ = new QStackedWidget(this);
  if (video) {
    surface_ = new PlayerSurface(stage_);
    stage_->addWidget(surface_);
    connect(surface_, &PlayerSurface::Clicked, this, &FilePlayer::TogglePlay);
    connect(surface_, &PlayerSurface::DoubleClicked, this,
            [this] { SetFullscreen(placeholder_.isNull()); });
  }
  failure_ = new QWidget(stage_);
  failure_->setObjectName(QStringLiteral("PlayerError"));
  auto* failure = new QVBoxLayout(failure_);
  failure->setContentsMargins(12, 12, 12, 12);
  failure->setSpacing(4);
  auto* title =
      MakeLabel(QStringLiteral("Couldn't play this file"), nullptr, failure_);
  title->setWordWrap(true);
  failure->addWidget(title);
  auto* about = MakeLabel(
      QStringLiteral(
          "It may have been moved or deleted. Run the node again to make it."),
      "muted", failure_);
  about->setWordWrap(true);
  failure->addWidget(about);
  auto* retry = MakeButton(QStringLiteral("Try again"), "outline",
                           QStringLiteral("rotate-ccw"), failure_);
  connect(retry, &QPushButton::clicked, this, [this] {
    serial_ = 0;
    failed_ = false;
    App::Get().session().player_retry(id_);
    Tick();
  });
  failure->addWidget(retry, 0, Qt::AlignLeft);
  stage_->addWidget(failure_);
  if (!video) stage_->setVisible(false);
  layout->addWidget(stage_);
  loading_ = MakeLabel(QStringLiteral("Loading…"), "muted", this);
  loading_->setAlignment(Qt::AlignCenter);
  layout->addWidget(loading_);
  auto* controls = new QWidget(this);
  controls->setObjectName(QStringLiteral("PlayerControls"));
  auto* column = new QVBoxLayout(controls);
  column->setContentsMargins(6, 4, 6, 4);
  column->setSpacing(2);
  column->addWidget(seek_);
  auto* row = new QHBoxLayout();
  row->setSpacing(2);
  row->addWidget(play_);
  row->addWidget(mute_);
  volume_->setRange(0, 100);
  volume_->setValue(100);
  volume_->setFixedWidth(kVolumeWidth);
  volume_->setAccessibleName(QStringLiteral("Volume"));
  row->addWidget(volume_);
  time_->setObjectName(QStringLiteral("PlayerTime"));
  row->addWidget(time_);
  row->addStretch(1);
  speed_->setObjectName(QStringLiteral("PlayerSpeed"));
  speed_->setText(SpeedText(rate_));
  speed_->setToolTip(QStringLiteral("Playback speed"));
  speed_->setAccessibleName(QStringLiteral("Playback speed"));
  row->addWidget(speed_);
  if (video) {
    fullscreen_ = MakeIconButton(QStringLiteral("maximize"),
                                 QStringLiteral("Full screen"), this);
    row->addWidget(fullscreen_);
    connect(fullscreen_, &QToolButton::clicked, this,
            [this] { SetFullscreen(placeholder_.isNull()); });
  }
  column->addLayout(row);
  layout->addWidget(controls);
  connect(play_, &QToolButton::clicked, this, &FilePlayer::TogglePlay);
  connect(mute_, &QToolButton::clicked, this, [this] {
    muted_ = !muted_;
    ApplyVolume();
  });
  connect(volume_, &QSlider::valueChanged, this, [this] {
    if (muted_ && volume_->value() > 0) muted_ = false;
    ApplyVolume();
  });
  connect(speed_, &QToolButton::clicked, this, &FilePlayer::ShowSpeedMenu);
  connect(seek_, &SeekBar::Seeked, this, [this](double seconds) {
    App::Get().session().player_seek(id_, seconds);
    Tick();
  });
  connect(&timer_, &QTimer::timeout, this, &FilePlayer::Tick);
  timer_.setInterval(kTickMs);
  WakeHub::Get().Listen(id_, this, [this] { Tick(); });
  Tick();
}

FilePlayer::~FilePlayer() {
  WakeHub::Get().Forget(id_, this);
  if (placeholder_) delete placeholder_;
  App::Get().session().player_close(id_);
}

void FilePlayer::showEvent(QShowEvent* event) {
  QFrame::showEvent(event);
  Tick();
}

void FilePlayer::hideEvent(QHideEvent* event) {
  QFrame::hideEvent(event);
  if (placeholder_.isNull()) timer_.stop();
}

void FilePlayer::keyPressEvent(QKeyEvent* event) {
  switch (event->key()) {
    case Qt::Key_Space:
    case Qt::Key_K:
      TogglePlay();
      return;
    case Qt::Key_Escape:
      if (!placeholder_.isNull()) {
        SetFullscreen(false);
        return;
      }
      break;
    case Qt::Key_F:
      if (video_) {
        SetFullscreen(placeholder_.isNull());
        return;
      }
      break;
    case Qt::Key_M:
      muted_ = !muted_;
      ApplyVolume();
      return;
    case Qt::Key_Left:
    case Qt::Key_Right: {
      const bridge::PlayerView view = App::Get().session().player_view(id_);
      const double by =
          event->key() == Qt::Key_Left ? -kNudgeSeconds : kNudgeSeconds;
      App::Get().session().player_seek(id_, view.position + by);
      Tick();
      return;
    }
    default:
      break;
  }
  QFrame::keyPressEvent(event);
}

void FilePlayer::Tick() {
  auto& session = App::Get().session();
  const bridge::PlayerView view = session.player_view(id_);
  failed_ = view.failed;
  loading_->setVisible(!view.ready && !view.failed);
  if (view.failed) {
    stage_->setVisible(true);
    stage_->setCurrentWidget(failure_);
  } else if (surface_ != nullptr) {
    stage_->setVisible(true);
    stage_->setCurrentWidget(surface_);
    surface_->SetAspect(static_cast<int>(view.width),
                        static_cast<int>(view.height));
  } else {
    stage_->setVisible(false);
  }
  const bool usable = view.ready && !view.failed;
  for (QWidget* control : std::initializer_list<QWidget*>{
           seek_, play_, mute_, volume_, speed_, fullscreen_}) {
    if (control != nullptr) control->setEnabled(usable);
  }
  seek_->SetRange(view.duration);
  seek_->SetPosition(view.position);
  time_->setText(ToQ(session.player_time(view.position, view.duration)) +
                 QStringLiteral(" / ") +
                 ToQ(session.player_time(view.duration, view.duration)));
  if (view.playing != playing_) {
    playing_ = view.playing;
    play_->setIcon(App::Get().Icon(
        playing_ ? QStringLiteral("pause") : QStringLiteral("play"),
        App::Get().Token(QStringLiteral("foreground"))));
    play_->setToolTip(playing_ ? QStringLiteral("Pause")
                               : QStringLiteral("Play"));
    play_->setAccessibleName(play_->toolTip());
  }
  if (surface_ != nullptr && view.serial != serial_) ShowFrame();
  const bool tick = view.playing && (isVisible() || !placeholder_.isNull());
  if (tick && !timer_.isActive()) timer_.start();
  if (!tick && timer_.isActive()) timer_.stop();
}

void FilePlayer::resizeEvent(QResizeEvent* event) {
  QFrame::resizeEvent(event);
  volume_->setVisible(width() >= kWideEnough);
}

void FilePlayer::ShowFrame() {
  rust::Box<bridge::FrameRef> frame = App::Get().session().player_frame(id_);
  serial_ = frame->serial();
  const int width = static_cast<int>(frame->width());
  const int height = static_cast<int>(frame->height());
  if (width <= 0 || height <= 0) return;
  bridge::FrameRef* shared = frame.into_raw();
  const QImage image(
      shared->bytes().data(), width, height,
      static_cast<qsizetype>(shared->stride()), QImage::Format_RGB888,
      [](void* info) {
        rust::Box<bridge::FrameRef>::from_raw(
            static_cast<bridge::FrameRef*>(info));
      },
      shared);
  surface_->SetFrame(image);
}

void FilePlayer::TogglePlay() {
  if (failed_) return;
  auto& session = App::Get().session();
  if (session.player_view(id_).playing) {
    session.player_pause(id_);
  } else {
    session.player_play(id_);
  }
  Tick();
}

void FilePlayer::ApplyVolume() {
  App::Get().session().player_volume(id_, volume_->value() / 100.0, muted_);
  const bool silent = muted_ || volume_->value() == 0;
  mute_->setIcon(App::Get().Icon(
      silent ? QStringLiteral("volume-x") : QStringLiteral("volume-2"),
      App::Get().Token(QStringLiteral("foreground"))));
  mute_->setToolTip(muted_ ? QStringLiteral("Unmute") : QStringLiteral("Mute"));
  mute_->setAccessibleName(mute_->toolTip());
}

void FilePlayer::ShowSpeedMenu() {
  QMenu menu(speed_);
  for (double speed : kSpeeds) {
    QAction* action = menu.addAction(SpeedText(speed));
    action->setIcon(CheckedIcon(speed == rate_));
    connect(action, &QAction::triggered, this, [this, speed] {
      rate_ = speed;
      speed_->setText(SpeedText(rate_));
      App::Get().session().player_rate(id_, rate_);
    });
  }
  menu.exec(speed_->mapToGlobal(QPoint(0, speed_->height())));
}

void FilePlayer::SetFullscreen(bool on) {
  if (!video_) return;
  if (on && placeholder_.isNull()) {
    QWidget* parent = parentWidget();
    if (parent == nullptr || parent->layout() == nullptr) return;
    auto* placeholder = new QWidget(parent);
    placeholder->setFixedSize(size());
    parent->layout()->replaceWidget(this, placeholder);
    placeholder_ = placeholder;
    connect(placeholder, &QObject::destroyed, this, [this] {
      if (parentWidget() == nullptr) deleteLater();
    });
    setParent(nullptr, Qt::Window);
    setWindowTitle(label_);
    showFullScreen();
    fullscreen_->setIcon(
        App::Get().Icon(QStringLiteral("minimize"),
                        App::Get().Token(QStringLiteral("foreground"))));
    fullscreen_->setToolTip(QStringLiteral("Exit full screen"));
    fullscreen_->setAccessibleName(fullscreen_->toolTip());
    setFocus();
    return;
  }
  if (on || placeholder_.isNull()) return;
  QWidget* placeholder = placeholder_;
  QWidget* parent = placeholder->parentWidget();
  placeholder_.clear();
  showNormal();
  setParent(parent, Qt::Widget);
  parent->layout()->replaceWidget(placeholder, this);
  delete placeholder;
  show();
  fullscreen_->setIcon(
      App::Get().Icon(QStringLiteral("maximize"),
                      App::Get().Token(QStringLiteral("foreground"))));
  fullscreen_->setToolTip(QStringLiteral("Full screen"));
  fullscreen_->setAccessibleName(fullscreen_->toolTip());
}

ImagePreview::ImagePreview(const QString& file, const QString& label, int width,
                           QWidget* parent)
    : QLabel(parent), file_(file), label_(label) {
  setObjectName(QStringLiteral("ArtifactPicture"));
  setCursor(Qt::PointingHandCursor);
  setFocusPolicy(Qt::StrongFocus);
  setAccessibleName(QStringLiteral("Enlarge ") + label);
  setToolTip(QStringLiteral("Enlarge"));
  const QPixmap pixmap(file);
  if (!pixmap.isNull()) {
    setPixmap(pixmap.scaledToWidth(width, Qt::SmoothTransformation));
  }
}

void ImagePreview::mousePressEvent(QMouseEvent* event) {
  if (event->button() == Qt::LeftButton) Enlarge();
}

void ImagePreview::keyPressEvent(QKeyEvent* event) {
  if (event->key() == Qt::Key_Return || event->key() == Qt::Key_Enter ||
      event->key() == Qt::Key_Space) {
    Enlarge();
    return;
  }
  QLabel::keyPressEvent(event);
}

void ImagePreview::Enlarge() {
  const QPixmap pixmap(file_);
  if (pixmap.isNull()) return;
  auto* dialog = new QDialog(window());
  dialog->setObjectName(QStringLiteral("ImagePreviewDialog"));
  dialog->setWindowTitle(label_);
  dialog->setAttribute(Qt::WA_DeleteOnClose);
  auto* layout = new QVBoxLayout(dialog);
  layout->setContentsMargins(8, 8, 8, 8);
  auto* picture = new QLabel(dialog);
  picture->setAccessibleName(label_);
  const QScreen* screen = window()->screen() != nullptr
                              ? window()->screen()
                              : QGuiApplication::primaryScreen();
  const QRect area = screen->availableGeometry();
  const QSize largest(static_cast<int>(area.width() * kScreenShare),
                      static_cast<int>(area.height() * kScreenHeightShare));
  picture->setPixmap(pixmap.size().width() > largest.width() ||
                             pixmap.size().height() > largest.height()
                         ? pixmap.scaled(largest, Qt::KeepAspectRatio,
                                         Qt::SmoothTransformation)
                         : pixmap);
  layout->addWidget(picture);
  dialog->open();
}

}
