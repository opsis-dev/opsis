#include "live_view.h"

#include <QCloseEvent>
#include <QDateTime>
#include <QEvent>
#include <QHBoxLayout>
#include <QLabel>
#include <QMouseEvent>
#include <QPainter>
#include <QPainterPath>
#include <QPushButton>
#include <QToolButton>
#include <QVBoxLayout>
#include <algorithm>
#include <utility>

#include "app.h"
#include "opsis-qt/src/bridge.rs.h"
#include "wake_hub.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr int kPipWidth = 384;
constexpr int kPipMargin = 16;
constexpr int kRippleFrameMs = 16;
constexpr double kRippleMs = 600;
constexpr double kDesignWidth = 1920;
constexpr double kRippleStart = 12;
constexpr double kRippleGrowth = 36;
constexpr double kBoxRadius = 6;
constexpr double kBoxStroke = 1.0 / 480;
constexpr double kPointStroke = 1.0 / 960;
constexpr double kRippleStroke = 1.0 / 640;
constexpr double kLivePoint = 1.0 / 220;
constexpr int kBoxFillAlpha = 26;
constexpr int kBadgeHeight = 20;
constexpr int kBadgePad = 6;
constexpr int kBadgeInset = 8;
constexpr int kSpinnerSize = 20;

QPointer<LiveWindow>& OpenWindow() {
  static QPointer<LiveWindow> window;
  return window;
}

const bridge::LiveCaptureView* Focused(const bridge::LiveView& view) {
  if (view.focus < 0 || view.focus >= static_cast<int>(view.captures.size())) {
    return nullptr;
  }
  return &view.captures.at(static_cast<std::size_t>(view.focus));
}

QRectF Badge(QPainter& painter, QPointF at, const QString& text,
             const QColor& fill, const QColor& ink, bool from_bottom) {
  const QFontMetricsF metrics(painter.font());
  const double width = metrics.horizontalAdvance(text) + 2.0 * kBadgePad;
  QRectF box(at, QSizeF(width, kBadgeHeight));
  if (from_bottom) box.moveBottom(at.y());
  painter.setPen(Qt::NoPen);
  painter.setBrush(fill);
  painter.drawRoundedRect(box, 4, 4);
  painter.setPen(ink);
  painter.drawText(box, Qt::AlignCenter, text);
  return box;
}

}

LiveSurface::LiveSurface(bool overlays, QWidget* parent)
    : QWidget(parent), overlays_(overlays) {
  setObjectName(QStringLiteral("LiveSurface"));
  setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Preferred);
  setCursor(Qt::PointingHandCursor);
  ripples_.setInterval(kRippleFrameMs);
  connect(&ripples_, &QTimer::timeout, this, [this] { update(); });
}

LiveSurface::~LiveSurface() = default;

int LiveSurface::heightForWidth(int width) const {
  if (shown_ == nullptr || shown_->width == 0 || shown_->height == 0) {
    return width * 9 / 16;
  }
  return static_cast<int>(static_cast<double>(width) * shown_->height /
                          shown_->width);
}

void LiveSurface::Show(const bridge::LiveCaptureView* capture) {
  if (capture == nullptr) {
    shown_.reset();
    frame_ = QImage();
    serial_ = 0;
    capture_.clear();
    ripples_.start();
    update();
    return;
  }
  const QString id = ToQ(capture->capture_id);
  if (id != capture_) {
    capture_ = id;
    serial_ = 0;
    frame_ = QImage();
  }
  const bool resized = shown_ == nullptr || shown_->width != capture->width ||
                       shown_->height != capture->height;
  shown_ = std::make_unique<bridge::LiveCaptureView>(*capture);
  if (capture->frame != serial_) {
    const rust::Box<bridge::FrameRef> frame =
        App::Get().session().live_frame(capture->capture_id);
    serial_ = frame->serial();
    const auto bytes = frame->bytes();
    QImage picture = QImage::fromData(
        QByteArrayView(bytes.data(), static_cast<qsizetype>(bytes.size())),
        "JPEG");
    if (!picture.isNull()) frame_ = std::move(picture);
  }
  if (resized) updateGeometry();
  const bool moving =
      frame_.isNull() || (overlays_ && !capture->clicks.empty());
  if (moving && !ripples_.isActive()) ripples_.start();
  if (!moving) ripples_.stop();
  update();
}

void LiveSurface::paintEvent(QPaintEvent*) {
  QPainter painter(this);
  painter.setRenderHint(QPainter::Antialiasing);
  painter.setRenderHint(QPainter::SmoothPixmapTransform);
  QPainterPath clip;
  clip.addRoundedRect(QRectF(rect()), kBoxRadius, kBoxRadius);
  painter.setClipPath(clip);
  painter.fillRect(rect(), Qt::black);
  const QColor muted = App::Get().Token(QStringLiteral("muted-foreground"));
  if (frame_.isNull() || shown_ == nullptr) {
    const QRectF spin(width() / 2.0 - kSpinnerSize / 2.0,
                      height() / 2.0 - kSpinnerSize / 2.0, kSpinnerSize,
                      kSpinnerSize);
    painter.setPen(QPen(muted, 2, Qt::SolidLine, Qt::RoundCap));
    const int turn = static_cast<int>(QDateTime::currentMSecsSinceEpoch() %
                                      1000 * 360 / 1000);
    painter.drawArc(spin, -turn * 16, 270 * 16);
    return;
  }
  const QSizeF viewport(std::max(1U, shown_->width),
                        std::max(1U, shown_->height));
  const QSizeF fitted = viewport.scaled(QSizeF(size()), Qt::KeepAspectRatio);
  const QRectF target(QPointF((width() - fitted.width()) / 2.0,
                              (height() - fitted.height()) / 2.0),
                      fitted);
  painter.drawImage(target, frame_);
  const double scale = target.width() / viewport.width();
  const double wide = viewport.width();
  const QColor primary = App::Get().Token(QStringLiteral("primary"));
  const QColor background = App::Get().Token(QStringLiteral("background"));
  const auto at = [&](double x, double y) {
    return QPointF(target.x() + x * scale, target.y() + y * scale);
  };
  if (overlays_) {
    if (shown_->has_target) {
      QColor fill = primary;
      fill.setAlpha(kBoxFillAlpha);
      painter.setBrush(fill);
      painter.setPen(QPen(primary, std::max(1.0, wide * kBoxStroke * scale)));
      painter.drawRoundedRect(QRectF(at(shown_->target_x, shown_->target_y),
                                     QSizeF(shown_->target_width * scale,
                                            shown_->target_height * scale)),
                              kBoxRadius * scale, kBoxRadius * scale);
    }
    const double design = wide / kDesignWidth;
    painter.setBrush(Qt::NoBrush);
    for (const auto& click : shown_->clicks) {
      const double age = std::clamp(click.age, 0.0, 1.0);
      QColor ring = primary;
      ring.setAlphaF(static_cast<float>(1.0 - age));
      painter.setPen(QPen(ring, std::max(1.0, wide * kRippleStroke * scale)));
      const double radius =
          (kRippleStart + age * kRippleGrowth) * design * scale;
      painter.drawEllipse(at(click.x, click.y), radius, radius);
    }
    if (shown_->has_cursor) {
      painter.setBrush(primary);
      painter.setPen(
          QPen(background, std::max(1.0, wide * kPointStroke * scale)));
      const double radius = wide * kLivePoint * scale;
      painter.drawEllipse(at(shown_->cursor_x, shown_->cursor_y), radius,
                          radius);
    }
  }
  QFont small = painter.font();
  small.setPixelSize(11);
  small.setWeight(QFont::DemiBold);
  painter.setFont(small);
  const QString status = ToQ(shown_->status);
  if (status == QStringLiteral("live")) {
    Badge(painter, QPointF(kBadgeInset, kBadgeInset), QStringLiteral("● LIVE"),
          App::Get().Token(QStringLiteral("live")),
          App::Get().Token(QStringLiteral("live-foreground")), false);
  }
  if (!overlays_) return;
  QColor plate = background;
  plate.setAlphaF(0.8F);
  const QColor ink = App::Get().Token(QStringLiteral("foreground"));
  double x = kBadgeInset;
  const double bottom = height() - kBadgeInset;
  if (status != QStringLiteral("live")) {
    const bool failed = status == QStringLiteral("error");
    x = Badge(painter, QPointF(x, bottom),
              failed ? QStringLiteral("Failed") : QStringLiteral("Done"),
              App::Get().Token(failed ? QStringLiteral("destructive")
                                      : QStringLiteral("secondary")),
              ink, true)
            .right() +
        kBadgePad;
  }
  const QString step = ToQ(shown_->step);
  if (!step.isEmpty()) {
    x = Badge(painter, QPointF(x, bottom), step, plate, ink, true).right() +
        kBadgePad;
  }
  const QString label = ToQ(shown_->target_label);
  if (!label.isEmpty() && label != step) {
    Badge(painter, QPointF(x, bottom), label, plate, ink, true);
  }
}

void LiveSurface::mousePressEvent(QMouseEvent* event) {
  if (event->button() == Qt::LeftButton) emit Clicked();
}

void LiveWindow::Open() {
  if (!OpenWindow().isNull()) {
    OpenWindow()->raise();
    OpenWindow()->activateWindow();
    return;
  }
  auto* window = new LiveWindow();
  OpenWindow() = window;
  window->show();
  App::Get().session().live_pip(false);
}

bool LiveWindow::IsOpen() { return !OpenWindow().isNull(); }

LiveWindow::LiveWindow()
    : QWidget(nullptr), surface_(new LiveSurface(false, this)) {
  setObjectName(QStringLiteral("LiveWindow"));
  setAttribute(Qt::WA_DeleteOnClose);
  setWindowTitle(QStringLiteral("Live recording"));
  setStyleSheet(QStringLiteral("QWidget#LiveWindow { background: black; }"));
  resize(1280, 760);
  auto* layout = new QVBoxLayout(this);
  layout->setContentsMargins(0, 0, 0, 0);
  surface_->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Expanding);
  layout->addWidget(surface_);
  WakeHub::Get().Listen(App::Get().session().live_target(), this,
                        [this] { Sync(); });
  Sync();
}

void LiveWindow::Sync() {
  const bridge::LiveView view = App::Get().session().live_view();
  surface_->Show(Focused(view));
}

void LiveWindow::closeEvent(QCloseEvent* event) {
  WakeHub::Get().Forget(App::Get().session().live_target(), this);
  OpenWindow().clear();
  App::Get().session().live_pip(true);
  QWidget::closeEvent(event);
}

LivePip::LivePip(QWidget* host, QPushButton* toggle)
    : QFrame(host),
      host_(host),
      toggle_(toggle),
      title_(MakeLabel(QString(), nullptr, this)),
      count_(MakeLabel(QString(), "muted", this)),
      surface_(new LiveSurface(true, this)),
      switcher_(new QWidget(this)),
      switcher_row_(new QHBoxLayout(switcher_)) {
  setObjectName(QStringLiteral("LivePip"));
  setProperty("role", "card");
  setAttribute(Qt::WA_StyledBackground);
  setFixedWidth(kPipWidth);
  auto* layout = new QVBoxLayout(this);
  layout->setContentsMargins(8, 8, 8, 8);
  layout->setSpacing(8);
  auto* header = new QHBoxLayout();
  header->setSpacing(4);
  QFont strong = title_->font();
  strong.setWeight(QFont::Medium);
  title_->setFont(strong);
  title_->setSizePolicy(QSizePolicy::Ignored, QSizePolicy::Preferred);
  header->addWidget(title_, 1);
  header->addWidget(count_);
  auto* enlarge =
      MakeIconButton(QStringLiteral("maximize-2"),
                     QStringLiteral("Open in its own window"), this);
  connect(enlarge, &QToolButton::clicked, this, [] { LiveWindow::Open(); });
  header->addWidget(enlarge);
  auto* close = MakeIconButton(QStringLiteral("x"),
                               QStringLiteral("Close live view"), this);
  connect(close, &QToolButton::clicked, this,
          [] { App::Get().session().live_pip(false); });
  header->addWidget(close);
  layout->addLayout(header);
  connect(surface_, &LiveSurface::Clicked, this, [] { LiveWindow::Open(); });
  layout->addWidget(surface_);
  switcher_row_->setContentsMargins(0, 0, 0, 0);
  switcher_row_->setSpacing(4);
  layout->addWidget(switcher_);
  if (toggle_ != nullptr) {
    connect(toggle_, &QPushButton::clicked, this,
            [] { App::Get().session().live_pip(true); });
  }
  host_->installEventFilter(this);
  WakeHub::Get().Listen(App::Get().session().live_target(), this,
                        [this] { Sync(); });
  hide();
  Sync();
}

void LivePip::Sync() {
  const bridge::LiveView view = App::Get().session().live_view();
  const bridge::LiveCaptureView* capture = Focused(view);
  const bool any = !view.captures.empty();
  const bool window = LiveWindow::IsOpen();
  if (toggle_ != nullptr) toggle_->setVisible(any && !view.pip_open && !window);
  const bool shown = capture != nullptr && view.pip_open && !window;
  if (!shown) {
    hide();
    return;
  }
  title_->setText(ToQ(capture->title));
  const int count = static_cast<int>(view.captures.size());
  count_->setVisible(count > 1);
  count_->setText(QStringLiteral("%1/%2").arg(view.focus + 1).arg(count));
  QString key;
  for (const auto& each : view.captures) {
    key += ToQ(each.capture_id) + ToQ(each.title);
  }
  key += QString::number(view.focus);
  if (key != switcher_key_) {
    switcher_key_ = key;
    while (QLayoutItem* item = switcher_row_->takeAt(0)) {
      delete item->widget();
      delete item;
    }
    for (int i = 0; i < count; ++i) {
      const auto& each = view.captures.at(static_cast<std::size_t>(i));
      auto* button =
          MakeButton(ToQ(each.title), "toggle-outline", QString(), switcher_);
      button->setCheckable(true);
      button->setChecked(i == view.focus);
      const QString id = ToQ(each.capture_id);
      connect(button, &QPushButton::clicked, this,
              [id] { App::Get().session().live_focus(ToRust(id)); });
      switcher_row_->addWidget(button);
    }
    switcher_row_->addStretch(1);
  }
  switcher_->setVisible(count > 1);
  surface_->Show(capture);
  show();
  FitHeightToWidth(this);
  Place();
  raise();
}

void LivePip::Place() {
  const QRect area = host_->rect();
  move(area.right() - width() - kPipMargin,
       area.bottom() - height() - kPipMargin);
}

bool LivePip::eventFilter(QObject* watched, QEvent* event) {
  if (watched == host_ && event->type() == QEvent::Resize && isVisible()) {
    Place();
  }
  return QFrame::eventFilter(watched, event);
}

}
