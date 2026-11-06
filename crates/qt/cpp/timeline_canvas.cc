#include "timeline_canvas.h"

#include <QApplication>
#include <QDialog>
#include <QFontMetrics>
#include <QHBoxLayout>
#include <QJsonArray>
#include <QJsonDocument>
#include <QKeyEvent>
#include <QMouseEvent>
#include <QPainter>
#include <QPainterPath>
#include <QPushButton>
#include <QScrollBar>
#include <QToolTip>
#include <QVBoxLayout>
#include <QWheelEvent>
#include <algorithm>
#include <cmath>

#include "app.h"
#include "event_hub.h"
#include "playback_clock.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr int kHeader = 84;
constexpr int kRuler = 24;
constexpr int kMarkers = 20;
constexpr int kVideoH = 46;
constexpr int kAudioH = 46;
constexpr int kInset = 8;
constexpr double kSnapPx = 8;
constexpr double kTail = 240;
constexpr double kFitPadding = 32;
constexpr double kEdgePx = 6;
constexpr double kMinBarPx = 2;
constexpr double kDragStart = 4;
constexpr double kCorner = 10;
constexpr double kCameraKey = 9;
constexpr double kFollowMargin = 24;
constexpr double kZoomStep = 1.5;
constexpr double kMinPpf = 0.01;
constexpr double kMaxPpf = 24;
constexpr int kMaxTicks = 600;
constexpr int kTagHeight = 16;
constexpr int kTagPad = 4;
constexpr int kSmallFont = 10;
constexpr int kLabelFont = 11;
constexpr double kMsPerSecond = 1000.0;
constexpr double kStripOpacity = 0.6;
constexpr int kCachedPictures = 64;

QColor WithAlpha(QColor color, double alpha) {
  color.setAlphaF(static_cast<float>(alpha));
  return color;
}

QColor Token(const char* name) { return App::Get().Token(QLatin1String(name)); }

QString Compact(const QJsonObject& object) {
  return QString::fromUtf8(
      QJsonDocument(object).toJson(QJsonDocument::Compact));
}

}

TimelineCanvas::TimelineCanvas(QWidget* parent) : QAbstractScrollArea(parent) {
  setObjectName(QStringLiteral("TimelineCanvas"));
  setFrameShape(QFrame::NoFrame);
  setFocusPolicy(Qt::StrongFocus);
  viewport()->setMouseTracking(true);
  setHorizontalScrollBarPolicy(Qt::ScrollBarAsNeeded);
  setVerticalScrollBarPolicy(Qt::ScrollBarAsNeeded);
  connect(EventHub::Instance(), &EventHub::MediaChanged, viewport(),
          [this] { viewport()->update(); });
  connect(PlaybackClock::Instance(), &PlaybackClock::Changed, this, [this] {
    FollowPlayhead();
    viewport()->update();
  });
}

void TimelineCanvas::Reload() {
  view_ =
      std::make_unique<bridge::TimelineView>(App::Get().session().timeline());
  PlaybackClock::Instance()->SetFilm(static_cast<int>(view_->fps),
                                     view_->duration);
  if (fit_) {
    Fit();
  } else {
    UpdateScroll();
  }
  viewport()->update();
  emit StateChanged();
}

void TimelineCanvas::SetTool(const QString& tool) {
  tool_ = tool;
  emit StateChanged();
}

void TimelineCanvas::SetSnapping(bool on) {
  snapping_ = on;
  emit StateChanged();
}

void TimelineCanvas::SetTimecode(bool on) {
  timecode_ = on;
  viewport()->update();
  emit StateChanged();
}

QString TimelineCanvas::TimeText(double frame) const {
  const std::uint32_t fps = view_ != nullptr ? view_->fps : 30;
  return ToQ(App::Get().session().format_time(frame, fps, timecode_));
}

double TimelineCanvas::ContentWidth() const {
  const double duration = view_ != nullptr ? view_->duration : 1;
  return std::max<double>(viewport()->width() - kHeader,
                          duration * ppf_ + kTail);
}

int TimelineCanvas::RulerBottom() const { return kRuler + kMarkers; }

int TimelineCanvas::VideoRowTop(std::uint32_t lane) const {
  const std::uint32_t lanes = view_ != nullptr ? std::max(1U, view_->lanes) : 1;
  return kRuler + kMarkers +
         static_cast<int>(lanes - 1 - std::min(lane, lanes - 1)) * kVideoH -
         verticalScrollBar()->value();
}

int TimelineCanvas::AudioRowTop(std::uint32_t lane) const {
  const std::uint32_t lanes = view_ != nullptr ? std::max(1U, view_->lanes) : 1;
  return kRuler + kMarkers + static_cast<int>(lanes) * kVideoH +
         static_cast<int>(lane) * kAudioH - verticalScrollBar()->value();
}

double TimelineCanvas::FrameX(double frame) const {
  return kHeader + frame * ppf_ - horizontalScrollBar()->value();
}

double TimelineCanvas::FrameAt(double x) const {
  return (x - kHeader + horizontalScrollBar()->value()) / ppf_;
}

void TimelineCanvas::UpdateScroll() {
  const int visible = std::max(1, viewport()->width() - kHeader);
  horizontalScrollBar()->setPageStep(visible);
  horizontalScrollBar()->setRange(
      0, std::max(0, static_cast<int>(ContentWidth()) - visible));
  const int rows =
      view_ != nullptr
          ? static_cast<int>(std::max(1U, view_->lanes) * kVideoH +
                             std::max(1U, view_->audio_lanes) * kAudioH)
          : 0;
  const int room = std::max(1, viewport()->height() - kRuler - kMarkers);
  verticalScrollBar()->setPageStep(room);
  verticalScrollBar()->setRange(0, std::max(0, rows - room));
}

void TimelineCanvas::resizeEvent(QResizeEvent* event) {
  QAbstractScrollArea::resizeEvent(event);
  if (fit_) {
    Fit();
  } else {
    UpdateScroll();
  }
}

void TimelineCanvas::scrollContentsBy(int, int) { viewport()->update(); }

void TimelineCanvas::ZoomAround(double factor, double x) {
  const double frame = FrameAt(x);
  ppf_ = std::clamp(ppf_ * factor, kMinPpf, kMaxPpf);
  fit_ = false;
  UpdateScroll();
  horizontalScrollBar()->setValue(
      static_cast<int>(std::lround(frame * ppf_ - (x - kHeader))));
  viewport()->update();
  emit StateChanged();
}

void TimelineCanvas::ZoomBy(double factor) {
  ZoomAround(factor, FrameX(PlaybackClock::Instance()->frame()));
}

void TimelineCanvas::Fit() {
  const double duration = view_ != nullptr ? view_->duration : 1;
  const double room =
      std::max(1.0, viewport()->width() - kHeader - kFitPadding);
  ppf_ = std::clamp(room / std::max(1.0, duration), kMinPpf, kMaxPpf);
  fit_ = true;
  UpdateScroll();
  horizontalScrollBar()->setValue(0);
  viewport()->update();
  emit StateChanged();
}

void TimelineCanvas::FollowPlayhead() {
  const PlaybackClock* clock = PlaybackClock::Instance();
  if (!clock->playing()) return;
  const double x = clock->frame() * ppf_;
  const int left = horizontalScrollBar()->value();
  const int visible = viewport()->width() - kHeader;
  if (x < left || x > left + visible - kFollowMargin) {
    horizontalScrollBar()->setValue(
        static_cast<int>(std::max(0.0, x - kFollowMargin)));
  }
}

void TimelineCanvas::paintEvent(QPaintEvent*) {
  QPainter painter(viewport());
  painter.fillRect(viewport()->rect(), Token("background"));
  if (view_ == nullptr || !view_->has_film) return;
  painter.setRenderHint(QPainter::Antialiasing);
  PaintRows(&painter);
  PaintHeader(&painter);
  PaintMarkers(&painter);
  PaintRuler(&painter);
  const double bottom = viewport()->height();
  if (guide_) {
    const double x = FrameX(*guide_);
    painter.setPen(QPen(Token("keyframe"), 1));
    painter.drawLine(QPointF(x, kRuler), QPointF(x, bottom));
  }
  if (marquee_) {
    painter.setPen(QPen(Token("primary"), 1));
    painter.setBrush(WithAlpha(Token("primary"), 0.1));
    painter.drawRect(*marquee_);
  }
  const double x = FrameX(PlaybackClock::Instance()->frame());
  if (x >= kHeader) {
    const QColor playhead = Token("playhead");
    painter.setPen(QPen(playhead, 1));
    painter.drawLine(QPointF(x, 0), QPointF(x, bottom));
    QPainterPath head;
    head.moveTo(x - 6, 0);
    head.lineTo(x + 6, 0);
    head.lineTo(x, 10);
    head.closeSubpath();
    painter.fillPath(head, playhead);
  }
}

void TimelineCanvas::PaintRuler(QPainter* painter) {
  const QRect ruler(0, 0, viewport()->width(), kRuler);
  painter->fillRect(ruler, Token("sidebar"));
  painter->setPen(Token("border"));
  painter->drawLine(0, kRuler - 1, viewport()->width(), kRuler - 1);
  painter->save();
  painter->setClipRect(
      QRect(kHeader, 0, viewport()->width() - kHeader, kRuler));
  const PlaybackClock* clock = PlaybackClock::Instance();
  if (clock->in_point() >= 0 || clock->out_point() >= 0) {
    const double from = std::max(0.0, clock->in_point());
    const double to =
        clock->out_point() >= 0 ? clock->out_point() : view_->duration;
    painter->fillRect(QRectF(FrameX(from), 0, (to - from) * ppf_, kRuler),
                      WithAlpha(Token("primary"), 0.15));
  }
  const bridge::RulerStep step =
      App::Get().session().ruler_step(ppf_, view_->fps);
  const double left = horizontalScrollBar()->value();
  const double first = std::floor(left / ppf_ / step.minor) * step.minor;
  const double last = (left + viewport()->width() - kHeader) / ppf_;
  QFont font = painter->font();
  font.setPixelSize(kSmallFont);
  painter->setFont(font);
  const QColor muted = Token("muted-foreground");
  for (int tick = 0; tick < kMaxTicks; ++tick) {
    const double f = first + tick * step.minor;
    if (f > last) break;
    const bool major =
        std::abs(f / step.step - std::round(f / step.step)) < 1e-6;
    const double x = FrameX(f);
    painter->setPen(QPen(WithAlpha(muted, major ? 0.6 : 0.3), 1));
    painter->drawLine(QPointF(x, kRuler - (major ? 10 : 6)),
                      QPointF(x, kRuler));
    if (major) {
      painter->setPen(muted);
      painter->drawText(QPointF(x + 4, kRuler - 12), TimeText(f));
    }
  }
  painter->setPen(Token("border"));
  const double end = FrameX(view_->duration);
  painter->drawLine(QPointF(end, 0), QPointF(end, kRuler));
  painter->restore();
  painter->fillRect(QRect(0, 0, kHeader, kRuler), Token("sidebar"));
  painter->setPen(Token("border"));
  painter->drawLine(kHeader - 1, 0, kHeader - 1, kRuler);
}

QRectF TimelineCanvas::MarkerRect(const QString& id) const {
  if (view_ == nullptr) return {};
  QFont font;
  font.setPixelSize(kSmallFont);
  const QFontMetrics metrics(font);
  for (const auto& marker : view_->markers) {
    if (ToQ(marker.id) != id) continue;
    const QString label =
        ToQ(marker.label).isEmpty() ? QStringLiteral(" ") : ToQ(marker.label);
    return {FrameX(marker.frame), kRuler + 2.0,
            static_cast<double>(metrics.horizontalAdvance(label) + kTagPad * 2),
            static_cast<double>(kTagHeight)};
  }
  return {};
}

void TimelineCanvas::PaintMarkers(QPainter* painter) {
  const QRect row(0, kRuler, viewport()->width(), kMarkers);
  painter->fillRect(row, Token("background"));
  painter->setPen(Token("border"));
  painter->drawLine(0, kRuler + kMarkers - 1, viewport()->width(),
                    kRuler + kMarkers - 1);
  painter->save();
  painter->setClipRect(
      QRect(kHeader, kRuler, viewport()->width() - kHeader, kMarkers));
  QFont font = painter->font();
  font.setPixelSize(kSmallFont);
  painter->setFont(font);
  const QColor muted = Token("muted-foreground");
  for (const auto& mark : view_->marks) {
    if (!mark.chapter) continue;
    const double x = FrameX(mark.frame);
    painter->fillRect(QRectF(x, kRuler + 4, 2, 12), WithAlpha(muted, 0.7));
    painter->setPen(muted);
    painter->drawText(QPointF(x + 6, kRuler + 14), ToQ(mark.label));
  }
  for (const auto& marker : view_->markers) {
    const QString id = ToQ(marker.id);
    const QRectF tag = MarkerRect(id);
    const bool chosen = id == selected_marker_;
    QPainterPath path;
    path.addRoundedRect(tag, 3, 3);
    painter->fillPath(path, chosen ? Token("playhead") : Token("accent"));
    painter->setPen(chosen ? Token("background") : Token("foreground"));
    painter->drawText(tag.adjusted(kTagPad, 0, 0, 0),
                      Qt::AlignLeft | Qt::AlignVCenter, ToQ(marker.label));
  }
  painter->restore();
  painter->fillRect(QRect(0, kRuler, kHeader, kMarkers), Token("sidebar"));
  painter->setPen(Token("border"));
  painter->drawLine(kHeader - 1, kRuler, kHeader - 1, kRuler + kMarkers);
}

QRectF TimelineCanvas::BarRect(const QString& key) const {
  if (view_ == nullptr) return {};
  for (const auto& bar : view_->bars) {
    if (ToQ(bar.key) != key) continue;
    return {FrameX(bar.from), VideoRowTop(bar.lane) + kInset / 2.0,
            std::max(kMinBarPx, (bar.to - bar.from) * ppf_),
            static_cast<double>(kVideoH - kInset)};
  }
  for (const auto& bar : view_->audio) {
    if (ToQ(bar.key) != key) continue;
    return {FrameX(bar.from), AudioRowTop(bar.lane) + kInset / 2.0,
            std::max(kMinBarPx, (bar.to - bar.from) * ppf_),
            static_cast<double>(kAudioH - kInset)};
  }
  return {};
}

QPointF TimelineCanvas::CameraKeyPoint(const QString& key) const {
  if (view_ == nullptr) return {};
  for (const auto& mark : view_->camera_keys) {
    if (ToQ(mark.key) != key) continue;
    const QRectF bar = BarRect(ToQ(mark.bar_key));
    return {FrameX(mark.frame), bar.bottom() - kCameraKey / 2 - 4};
  }
  return {};
}

const QPixmap& TimelineCanvas::Picture(const QString& path) {
  auto found = pictures_.find(path);
  if (found == pictures_.end()) {
    if (pictures_.size() > kCachedPictures) pictures_.clear();
    found = pictures_.insert(path, QPixmap(path));
  }
  return found.value();
}

void TimelineCanvas::PaintFilmstrip(QPainter* painter, const QString& key,
                                    const QRectF& box) {
  const double offset = std::max(0.0, kHeader - box.left());
  const double right = std::min<double>(box.right(), viewport()->width());
  const double width = right - (box.left() + offset);
  if (width < 1) return;
  const bridge::FilmTiles tiles = App::Get().session().film_tiles(
      ToRust(key), ppf_, box.height(), offset, width);
  if (tiles.image.empty()) return;
  const QPixmap& picture = Picture(ToQ(tiles.image));
  if (picture.isNull()) return;
  painter->save();
  painter->setOpacity(kStripOpacity);
  if (tiles.still) {
    const double tile =
        picture.width() * box.height() / std::max(1, picture.height());
    const double left = box.left() + horizontalScrollBar()->value() - kHeader;
    const double first = std::floor((left + offset) / tile) * tile - left;
    const int count =
        static_cast<int>(std::ceil((offset + width - first) / tile));
    for (int n = 0; n < count; ++n) {
      const double x = first + n * tile;
      painter->drawPixmap(QRectF(box.left() + x, box.top(), tile, box.height()),
                          picture, QRectF(picture.rect()));
    }
  } else {
    for (const auto& tile : tiles.tiles) {
      painter->drawPixmap(
          QRectF(box.left() + tile.x, box.top(), tile.width, box.height()),
          picture,
          QRectF(static_cast<double>(tile.index * tiles.tile_width), 0,
                 tiles.tile_width, tiles.tile_height));
    }
  }
  painter->restore();
}

void TimelineCanvas::PaintWaveform(QPainter* painter, const QString& key,
                                   const QRectF& box) {
  const double offset = std::max(0.0, kHeader - box.left());
  const double right = std::min<double>(box.right(), viewport()->width());
  const auto width = static_cast<std::uint32_t>(
      std::max(0.0, std::floor(right - (box.left() + offset))));
  if (width == 0) return;
  const rust::Vec<float> columns =
      App::Get().session().wave_columns(ToRust(key), ppf_, offset, width);
  if (columns.size() < static_cast<std::size_t>(width) * 2) return;
  const QColor colour = Token("waveform");
  const double mid = box.top() + box.height() / 2;
  const double half = box.height() / 2;
  for (std::uint32_t i = 0; i < width; ++i) {
    const float high = columns.at(static_cast<std::size_t>(i) * 2);
    const float low = columns.at(static_cast<std::size_t>(i) * 2 + 1);
    if (std::isnan(high) || std::isnan(low)) continue;
    const double top = mid - high * half;
    const double bottom = mid - low * half;
    painter->fillRect(
        QRectF(box.left() + offset + i, top, 1, std::max(1.0, bottom - top)),
        colour);
  }
}

void TimelineCanvas::PaintBar(QPainter* painter,
                              const bridge::TimelineBarView& bar) {
  const QRectF box = BarRect(ToQ(bar.key));
  if (box.right() < kHeader || box.left() > viewport()->width()) return;
  const double frame = PlaybackClock::Instance()->frame();
  const bool under = frame >= bar.from && frame < bar.to;
  QColor fill = App::Get().Token(ToQ(bar.tone));
  if (under) fill = fill.lighter(125);
  QPainterPath path;
  path.addRoundedRect(box, 6, 6);
  painter->fillPath(path, fill);
  painter->save();
  painter->setClipPath(path);
  PaintFilmstrip(painter, ToQ(bar.key), box);
  QFont font = painter->font();
  font.setPixelSize(kLabelFont);
  painter->setFont(font);
  const QString title = ToQ(bar.title);
  const QString wrappers = ToQ(bar.wrappers);
  const QFontMetrics metrics(font);
  const QRectF text = box.adjusted(6, 2, -6, 0);
  const QString shown =
      metrics.elidedText(title, Qt::ElideRight, static_cast<int>(text.width()));
  painter->setPen(Token("foreground"));
  painter->drawText(text, Qt::AlignLeft | Qt::AlignTop, shown);
  if (!wrappers.isEmpty()) {
    const double used = metrics.horizontalAdvance(shown);
    painter->setPen(WithAlpha(Token("foreground"), 0.6));
    painter->drawText(
        text.adjusted(used, 0, 0, 0), Qt::AlignLeft | Qt::AlignTop,
        metrics.elidedText(QStringLiteral(" · ") + wrappers, Qt::ElideRight,
                           static_cast<int>(text.width() - used)));
  }
  for (const auto& mark : view_->marks) {
    if (mark.chapter || mark.bar_key != bar.key) continue;
    const double x = FrameX(mark.frame);
    painter->fillRect(QRectF(x, box.bottom() - 8, 2, 8),
                      WithAlpha(Token("foreground"), 0.8));
  }
  QPainterPath corner;
  corner.moveTo(box.right() - kCorner, box.top());
  corner.lineTo(box.right(), box.top());
  corner.lineTo(box.right(), box.top() + kCorner);
  corner.closeSubpath();
  painter->fillPath(corner, WithAlpha(Token("foreground"), 0.3));
  painter->restore();
  painter->setBrush(Qt::NoBrush);
  painter->setPen(QPen(under ? WithAlpha(Token("primary"), 0.8)
                             : WithAlpha(Token("foreground"), 0.1),
                       1));
  painter->drawPath(path);
  if (bar.selected || bar.related) {
    painter->setPen(QPen(
        bar.selected ? Token("primary") : WithAlpha(Token("foreground"), 0.4),
        bar.selected ? 2 : 1));
    painter->drawRoundedRect(box.adjusted(-1, -1, 1, 1), 7, 7);
  }
  for (const auto& mark : view_->camera_keys) {
    if (mark.bar_key != bar.key) continue;
    const QPointF at = CameraKeyPoint(ToQ(mark.key));
    QPainterPath diamond;
    diamond.moveTo(at.x(), at.y() - kCameraKey / 2);
    diamond.lineTo(at.x() + kCameraKey / 2, at.y());
    diamond.lineTo(at.x(), at.y() + kCameraKey / 2);
    diamond.lineTo(at.x() - kCameraKey / 2, at.y());
    diamond.closeSubpath();
    painter->fillPath(diamond, Token("keyframe"));
    painter->setPen(QPen(Token("background"), 1));
    painter->drawPath(diamond);
  }
}

void TimelineCanvas::PaintAudio(QPainter* painter,
                                const bridge::TimelineAudioView& bar) {
  const QRectF box = BarRect(ToQ(bar.key));
  if (box.right() < kHeader || box.left() > viewport()->width()) return;
  const double frame = PlaybackClock::Instance()->frame();
  const bool under = frame >= bar.from && frame < bar.to;
  QColor fill = App::Get().Token(ToQ(bar.tone));
  if (under) fill = fill.lighter(125);
  QPainterPath path;
  path.addRoundedRect(box, 6, 6);
  painter->fillPath(path, fill);
  painter->save();
  painter->setClipPath(path);
  PaintWaveform(painter, ToQ(bar.key), box);
  QFont font = painter->font();
  font.setPixelSize(kLabelFont);
  painter->setFont(font);
  painter->setPen(Token("foreground"));
  const QFontMetrics metrics(font);
  painter->drawText(box.adjusted(6, 2, -6, 0), Qt::AlignLeft | Qt::AlignTop,
                    metrics.elidedText(ToQ(bar.title), Qt::ElideRight,
                                       static_cast<int>(box.width() - 12)));
  painter->restore();
  painter->setBrush(Qt::NoBrush);
  painter->setPen(QPen(under ? WithAlpha(Token("primary"), 0.8)
                             : WithAlpha(Token("foreground"), 0.1),
                       1));
  painter->drawPath(path);
  if (bar.selected) {
    painter->setPen(QPen(Token("primary"), 2));
    painter->drawRoundedRect(box.adjusted(-1, -1, 1, 1), 7, 7);
  }
}

void TimelineCanvas::PaintRows(QPainter* painter) {
  painter->save();
  painter->setClipRect(QRect(0, kRuler + kMarkers, viewport()->width(),
                             viewport()->height() - kRuler - kMarkers));
  const QColor line = WithAlpha(Token("foreground"), 0.05);
  for (std::uint32_t lane = 0; lane < std::max(1U, view_->lanes); ++lane) {
    painter->setPen(line);
    const int bottom = VideoRowTop(lane) + kVideoH - 1;
    painter->drawLine(0, bottom, viewport()->width(), bottom);
  }
  for (std::uint32_t lane = 0; lane < std::max(1U, view_->audio_lanes);
       ++lane) {
    painter->setPen(line);
    const int bottom = AudioRowTop(lane) + kAudioH - 1;
    painter->drawLine(0, bottom, viewport()->width(), bottom);
  }
  painter->setClipRect(QRect(kHeader, kRuler + kMarkers,
                             viewport()->width() - kHeader,
                             viewport()->height() - kRuler - kMarkers));
  for (const auto& bar : view_->bars) PaintBar(painter, bar);
  for (const auto& mark : view_->transitions) {
    for (const std::uint32_t lane : mark.lanes) {
      const QRectF band(FrameX(mark.from), VideoRowTop(lane) + 2.0,
                        std::max(kMinBarPx, (mark.to - mark.from) * ppf_), 8);
      QPainterPath path;
      path.addRoundedRect(band, 3, 3);
      painter->fillPath(path, WithAlpha(Token("foreground"), 0.35));
    }
  }
  for (const auto& bar : view_->audio) PaintAudio(painter, bar);
  painter->restore();
}

void TimelineCanvas::PaintHeader(QPainter* painter) {
  painter->save();
  painter->setClipRect(QRect(0, kRuler + kMarkers, kHeader,
                             viewport()->height() - kRuler - kMarkers));
  painter->fillRect(QRect(0, kRuler + kMarkers, kHeader, viewport()->height()),
                    Token("sidebar"));
  QFont font = painter->font();
  font.setPixelSize(kLabelFont);
  painter->setFont(font);
  painter->setPen(Token("muted-foreground"));
  for (std::uint32_t lane = 0; lane < std::max(1U, view_->lanes); ++lane) {
    painter->drawText(QRect(8, VideoRowTop(lane), kHeader - 8, kVideoH),
                      Qt::AlignLeft | Qt::AlignVCenter,
                      QStringLiteral("Video %1").arg(lane + 1));
  }
  for (std::uint32_t lane = 0; lane < std::max(1U, view_->audio_lanes);
       ++lane) {
    painter->drawText(QRect(8, AudioRowTop(lane), kHeader - 8, kAudioH),
                      Qt::AlignLeft | Qt::AlignVCenter,
                      QStringLiteral("Sound %1").arg(lane + 1));
  }
  painter->setPen(Token("border"));
  painter->drawLine(kHeader - 1, kRuler + kMarkers, kHeader - 1,
                    viewport()->height());
  painter->restore();
}

std::vector<double> TimelineCanvas::SnapTargets(const QString& exclude,
                                                bool markers) const {
  std::vector<double> targets;
  const PlaybackClock* clock = PlaybackClock::Instance();
  targets.push_back(clock->frame());
  for (const auto& bar : view_->bars) {
    if (ToQ(bar.key) == exclude) continue;
    targets.push_back(bar.from);
    targets.push_back(bar.to);
  }
  for (const auto& bar : view_->audio) {
    if (ToQ(bar.key) == exclude) continue;
    targets.push_back(bar.from);
    targets.push_back(bar.to);
  }
  if (markers) {
    for (const auto& marker : view_->markers) targets.push_back(marker.frame);
  }
  for (const auto& mark : view_->marks) targets.push_back(mark.frame);
  if (clock->in_point() >= 0) targets.push_back(clock->in_point());
  if (clock->out_point() >= 0) targets.push_back(clock->out_point());
  targets.push_back(0);
  targets.push_back(view_->duration);
  return targets;
}

double TimelineCanvas::SnapThreshold(bool alt) const {
  return snapping_ && !alt ? kSnapPx / ppf_ : 0;
}

const bridge::TimelineBarView* TimelineCanvas::VideoAt(
    const QPointF& point) const {
  for (std::size_t i = view_->bars.size(); i > 0; --i) {
    const auto& item = view_->bars.at(i - 1);
    if (BarRect(ToQ(item.key)).contains(point)) return &item;
  }
  return nullptr;
}

const bridge::TimelineAudioView* TimelineCanvas::AudioAt(
    const QPointF& point) const {
  for (std::size_t i = view_->audio.size(); i > 0; --i) {
    const auto& item = view_->audio.at(i - 1);
    if (BarRect(ToQ(item.key)).contains(point)) return &item;
  }
  return nullptr;
}

const bridge::TimelineMarkerView* TimelineCanvas::MarkerAt(
    const QPointF& point) const {
  for (std::size_t i = view_->markers.size(); i > 0; --i) {
    const auto& item = view_->markers.at(i - 1);
    if (MarkerRect(ToQ(item.id)).contains(point)) return &item;
  }
  return nullptr;
}

const bridge::TimelineCameraKeyView* TimelineCanvas::CameraKeyAt(
    const QPointF& point) const {
  for (const auto& mark : view_->camera_keys) {
    const QPointF at = CameraKeyPoint(ToQ(mark.key));
    if (std::abs(point.x() - at.x()) <= kCameraKey / 2 + 1 &&
        std::abs(point.y() - at.y()) <= kCameraKey / 2 + 1) {
      return &mark;
    }
  }
  return nullptr;
}

QString TimelineCanvas::EditKind(const bridge::TimelineBarView& bar,
                                 const QPointF& point, bool* ripple) const {
  const QRectF box = BarRect(ToQ(bar.key));
  const double edge = std::min(kEdgePx, box.width() / 3);
  const bool corner = point.x() >= box.right() - kCorner &&
                      point.y() <= box.top() + kCorner &&
                      (box.right() - point.x()) >= (point.y() - box.top());
  QString side;
  if (point.x() <= box.left() + edge) side = QStringLiteral("start");
  if (point.x() >= box.right() - edge) side = QStringLiteral("end");
  *ripple = false;
  if (corner || (!side.isEmpty() && tool_ == QStringLiteral("rate"))) {
    return QStringLiteral("speed");
  }
  if (tool_ == QStringLiteral("slip")) return QStringLiteral("slip");
  if (!side.isEmpty() && tool_ == QStringLiteral("roll")) {
    return side == QStringLiteral("start") ? QStringLiteral("rollStart")
                                           : QStringLiteral("rollEnd");
  }
  *ripple = tool_ == QStringLiteral("ripple");
  if (!side.isEmpty()) {
    return side == QStringLiteral("start") ? QStringLiteral("trimStart")
                                           : QStringLiteral("trimEnd");
  }
  return QStringLiteral("move");
}

void TimelineCanvas::Select(const QStringList& ids, bool additive) {
  QStringList next = ids;
  if (additive) {
    const QString current = ToQ(App::Get().session().selected());
    QStringList chosen;
    for (const auto& bar : view_->bars) {
      if (bar.selected && !chosen.contains(ToQ(bar.node_id))) {
        chosen.append(ToQ(bar.node_id));
      }
    }
    for (const auto& bar : view_->audio) {
      if (bar.selected && !chosen.contains(ToQ(bar.node_id))) {
        chosen.append(ToQ(bar.node_id));
      }
    }
    if (!current.isEmpty() && !chosen.contains(current)) chosen.append(current);
    for (const QString& id : ids) {
      if (chosen.contains(id)) {
        chosen.removeAll(id);
      } else {
        chosen.append(id);
      }
    }
    next = chosen;
  }
  rust::Vec<rust::String> list;
  for (const QString& id : next) list.push_back(ToRust(id));
  App::Get().session().select_pinned(std::move(list), view_->source_id);
}

void TimelineCanvas::SendEdit(const QJsonObject& edit, QString* error) {
  const QString reply =
      ToQ(App::Get().session().edit_timeline(ToRust(Compact(edit))));
  if (error != nullptr) {
    *error = reply;
  } else {
    ShowError(reply);
  }
}

void TimelineCanvas::ShowError(const QString& error) {
  if (!error.isEmpty()) App::Get().ShowError(error);
}

void TimelineCanvas::PressBar(QMouseEvent* event) {
  const QPointF point = event->position();
  const bool shift = (event->modifiers() & Qt::ShiftModifier) != 0;
  const bool alt = (event->modifiers() & Qt::AltModifier) != 0;
  if (const auto* bar = VideoAt(point)) {
    const QString node = ToQ(bar->node_id);
    if (tool_ == QStringLiteral("blade")) {
      const double at =
          App::Get()
              .session()
              .snap_frame(
                  FrameAt(point.x()),
                  [this] {
                    rust::Vec<double> list;
                    for (double f : SnapTargets(QString(), true)) {
                      list.push_back(f);
                    }
                    return list;
                  }(),
                  SnapThreshold(alt))
              .value;
      SendEdit(QJsonObject{{QStringLiteral("op"), QStringLiteral("splitBar")},
                           {QStringLiteral("bar"), ToQ(bar->key)},
                           {QStringLiteral("at"), std::round(at)}},
               nullptr);
      return;
    }
    if (!bar->selected || shift) {
      Select({node}, shift && tool_ == QStringLiteral("select"));
    }
    drag_ = Drag{};
    drag_.press = Press::kEdit;
    drag_.start = point;
    drag_.key = ToQ(bar->key);
    drag_.edit = EditKind(*bar, point, &drag_.ripple);
    drag_.from = bar->from;
    drag_.to = bar->to;
    drag_.node = node;
    return;
  }
  if (const auto* bar = AudioAt(point)) {
    const QString node = ToQ(bar->node_id);
    if (!node.isEmpty() && (!bar->selected || shift)) {
      Select({node}, shift && tool_ == QStringLiteral("select"));
    }
    if (bar->hits) return;
    const QRectF box = BarRect(ToQ(bar->key));
    const double edge = std::min(kEdgePx, box.width() / 3);
    drag_ = Drag{};
    drag_.press = Press::kEdit;
    drag_.start = point;
    drag_.key = ToQ(bar->key);
    drag_.audio = true;
    drag_.ripple = tool_ == QStringLiteral("ripple");
    drag_.edit =
        point.x() <= box.left() + edge
            ? QStringLiteral("audioTrimStart")
            : (point.x() >= box.right() - edge ? QStringLiteral("audioTrimEnd")
                                               : QStringLiteral("audioMove"));
    drag_.from = bar->from;
    drag_.to = bar->to;
    drag_.node = node;
  }
}

void TimelineCanvas::mousePressEvent(QMouseEvent* event) {
  setFocus();
  if (view_ == nullptr || !view_->has_film ||
      event->button() != Qt::LeftButton) {
    return;
  }
  if (rename_ != nullptr) rename_->deleteLater();
  const QPointF point = event->position();
  if (point.x() < kHeader) return;
  if (point.y() < kRuler) {
    drag_ = Drag{};
    drag_.press = Press::kScrub;
    PlaybackClock::Instance()->Seek(FrameAt(point.x()));
    return;
  }
  if (point.y() < kRuler + kMarkers) {
    if (const auto* marker = MarkerAt(point)) {
      drag_ = Drag{};
      drag_.press = Press::kMarker;
      drag_.start = point;
      drag_.key = ToQ(marker->id);
      drag_.origin = marker->frame;
      selected_marker_ = drag_.key;
      viewport()->update();
    }
    return;
  }
  if (const auto* mark = CameraKeyAt(point)) {
    drag_ = Drag{};
    drag_.press = Press::kCameraKey;
    drag_.start = point;
    drag_.key = ToQ(mark->key);
    drag_.node = ToQ(mark->node_id);
    return;
  }
  if (VideoAt(point) != nullptr || AudioAt(point) != nullptr) {
    PressBar(event);
    return;
  }
  drag_ = Drag{};
  drag_.press = Press::kMarquee;
  drag_.start = point;
  drag_.shift = (event->modifiers() & Qt::ShiftModifier) != 0;
}

void TimelineCanvas::MoveEdit(QMouseEvent* event) {
  const double raw = (event->position().x() - drag_.start.x()) / ppf_;
  const double threshold =
      SnapThreshold((event->modifiers() & Qt::AltModifier) != 0);
  const bool shift = (event->modifiers() & Qt::ShiftModifier) != 0;
  rust::Vec<double> targets;
  for (double f : SnapTargets(drag_.key, true)) targets.push_back(f);
  auto& session = App::Get().session();
  double delta = raw;
  guide_.reset();
  const bool moving = drag_.edit == QStringLiteral("move") ||
                      drag_.edit == QStringLiteral("audioMove");
  if (moving && threshold > 0) {
    const bridge::SnapView snapped = session.snap_span(
        drag_.from, drag_.to, raw, std::move(targets), threshold);
    delta = snapped.value;
    if (snapped.snapped) {
      const double from = drag_.from + delta;
      const double to = drag_.to + delta;
      guide_ =
          std::abs(from - (drag_.from + raw)) <= std::abs(to - (drag_.to + raw))
              ? from
              : to;
    }
  } else if (!moving && drag_.edit != QStringLiteral("slip") && threshold > 0) {
    const bool at_start = drag_.edit == QStringLiteral("trimStart") ||
                          drag_.edit == QStringLiteral("rollStart") ||
                          drag_.edit == QStringLiteral("audioTrimStart");
    const double edge = at_start ? drag_.from : drag_.to;
    const bridge::SnapView snapped =
        session.snap_frame(edge + raw, std::move(targets), threshold);
    delta = snapped.value - edge;
    if (snapped.snapped) guide_ = snapped.value;
  }
  QJsonObject edit{{QStringLiteral("op"), drag_.edit},
                   {QStringLiteral("bar"), drag_.key}};
  if (drag_.edit == QStringLiteral("slip")) {
    edit.insert(QStringLiteral("delta"), std::round(-raw));
  } else {
    edit.insert(QStringLiteral("delta"), std::round(delta));
  }
  if (moving) {
    edit.insert(QStringLiteral("ripple"), drag_.ripple || shift);
  } else if (drag_.edit == QStringLiteral("trimEnd")) {
    edit.insert(QStringLiteral("ripple"), drag_.ripple);
  }
  SendEdit(edit, &drag_.error);
  viewport()->update();
}

void TimelineCanvas::mouseMoveEvent(QMouseEvent* event) {
  const QPointF point = event->position();
  if (drag_.press == Press::kNone) {
    if (view_ == nullptr || !view_->has_film) return;
    if (const auto* bar = VideoAt(point)) {
      bool ripple = false;
      const QString kind = EditKind(*bar, point, &ripple);
      const bool edge = kind != QStringLiteral("move");
      viewport()->setCursor(
          tool_ == QStringLiteral("blade")
              ? Qt::CrossCursor
              : (edge ? Qt::SizeHorCursor : Qt::OpenHandCursor));
      viewport()->setToolTip(ToQ(bar->wrappers).isEmpty()
                                 ? ToQ(bar->title)
                                 : ToQ(bar->title) + QStringLiteral(" · ") +
                                       ToQ(bar->wrappers));
    } else if (CameraKeyAt(point) != nullptr) {
      viewport()->setCursor(Qt::SizeHorCursor);
      viewport()->setToolTip(QStringLiteral(
          "Camera key. Drag to move it; select the node to set it."));
    } else {
      viewport()->unsetCursor();
      viewport()->setToolTip(QString());
    }
    return;
  }
  const double moved = std::abs(point.x() - drag_.start.x()) +
                       std::abs(point.y() - drag_.start.y());
  switch (drag_.press) {
    case Press::kScrub:
      PlaybackClock::Instance()->Seek(FrameAt(point.x()));
      return;
    case Press::kMarquee:
      marquee_ = QRectF(drag_.start, point).normalized();
      viewport()->update();
      return;
    default:
      break;
  }
  if (!drag_.started) {
    if (moved < kDragStart) return;
    drag_.started = true;
    App::Get().session().begin_gesture();
  }
  if (drag_.press == Press::kEdit) {
    MoveEdit(event);
  } else if (drag_.press == Press::kCameraKey) {
    SendEdit(QJsonObject{{QStringLiteral("op"), QStringLiteral("cameraKey")},
                         {QStringLiteral("key"), drag_.key},
                         {QStringLiteral("delta"),
                          std::round((point.x() - drag_.start.x()) / ppf_)}},
             &drag_.error);
  } else if (drag_.press == Press::kMarker) {
    double frame = drag_.origin + (point.x() - drag_.start.x()) / ppf_;
    const double threshold =
        SnapThreshold((event->modifiers() & Qt::AltModifier) != 0);
    if (threshold > 0) {
      rust::Vec<double> targets;
      for (double f : SnapTargets(QString(), false)) targets.push_back(f);
      frame = App::Get()
                  .session()
                  .snap_frame(frame, std::move(targets), threshold)
                  .value;
    }
    const double ms =
        std::max(0.0, std::round(frame * kMsPerSecond / view_->fps));
    try {
      App::Get().session().edit_markers(ToRust(
          Compact(QJsonObject{{QStringLiteral("op"), QStringLiteral("update")},
                              {QStringLiteral("id"), drag_.key},
                              {QStringLiteral("atMs"), ms}})));
    } catch (const rust::Error& error) {
      drag_.error = QString::fromUtf8(error.what());
    }
  }
}

void TimelineCanvas::FinishMarquee(QMouseEvent* event) {
  const QRectF box = QRectF(drag_.start, event->position()).normalized();
  marquee_.reset();
  if (box.width() < kDragStart && box.height() < kDragStart) {
    if (!drag_.shift) Select({}, false);
    PlaybackClock::Instance()->Seek(FrameAt(drag_.start.x()));
    selected_marker_.clear();
    viewport()->update();
    return;
  }
  QStringList ids;
  for (const auto& bar : view_->bars) {
    if (BarRect(ToQ(bar.key)).intersects(box) &&
        !ids.contains(ToQ(bar.node_id))) {
      ids.append(ToQ(bar.node_id));
    }
  }
  for (const auto& bar : view_->audio) {
    if (!ToQ(bar.node_id).isEmpty() && BarRect(ToQ(bar.key)).intersects(box) &&
        !ids.contains(ToQ(bar.node_id))) {
      ids.append(ToQ(bar.node_id));
    }
  }
  Select(ids, drag_.shift);
  viewport()->update();
}

void TimelineCanvas::mouseReleaseEvent(QMouseEvent* event) {
  const Drag drag = drag_;
  drag_ = Drag{};
  guide_.reset();
  if (drag.press == Press::kMarquee) {
    FinishMarquee(event);
    return;
  }
  if (drag.started) {
    App::Get().session().end_gesture();
    ShowError(drag.error);
  } else if (drag.press == Press::kCameraKey) {
    Select({drag.node}, false);
  }
  viewport()->update();
}

void TimelineCanvas::StartRename(const QString& id) {
  const QRectF tag = MarkerRect(id);
  QString label;
  for (const auto& marker : view_->markers) {
    if (ToQ(marker.id) == id) label = ToQ(marker.label);
  }
  auto* line = new QLineEdit(label, viewport());
  line->setObjectName(QStringLiteral("MarkerName"));
  line->setAccessibleName(QStringLiteral("Marker name"));
  line->setGeometry(QRect(static_cast<int>(tag.left()),
                          static_cast<int>(tag.top()), 112, kTagHeight));
  line->selectAll();
  line->show();
  line->setFocus();
  rename_ = line;
  connect(line, &QLineEdit::editingFinished, this, [this, line, id] {
    if (line->isModified()) {
      try {
        App::Get().session().edit_markers(ToRust(Compact(
            QJsonObject{{QStringLiteral("op"), QStringLiteral("update")},
                        {QStringLiteral("id"), id},
                        {QStringLiteral("label"), line->text()}})));
      } catch (const rust::Error& error) {
        ShowError(QString::fromUtf8(error.what()));
      }
    }
    line->deleteLater();
    setFocus();
  });
}

void TimelineCanvas::mouseDoubleClickEvent(QMouseEvent* event) {
  if (view_ == nullptr) return;
  if (const auto* marker = MarkerAt(event->position())) {
    drag_ = Drag{};
    StartRename(ToQ(marker->id));
  }
}

void TimelineCanvas::wheelEvent(QWheelEvent* event) {
  const QPoint delta = event->angleDelta();
  if ((event->modifiers() & (Qt::ControlModifier | Qt::MetaModifier)) != 0) {
    ZoomAround(delta.y() > 0 ? kZoomStep : 1 / kZoomStep,
               event->position().x());
    return;
  }
  if ((event->modifiers() & Qt::ShiftModifier) != 0 || delta.x() != 0) {
    const int step = delta.x() != 0 ? delta.x() : delta.y();
    horizontalScrollBar()->setValue(horizontalScrollBar()->value() - step);
    return;
  }
  QAbstractScrollArea::wheelEvent(event);
}

void TimelineCanvas::SplitAtPlayhead() {
  QJsonArray selected;
  for (const auto& bar : view_->bars) {
    if (bar.selected && !selected.contains(ToQ(bar.node_id))) {
      selected.append(ToQ(bar.node_id));
    }
  }
  SendEdit(
      QJsonObject{{QStringLiteral("op"), QStringLiteral("split")},
                  {QStringLiteral("at"), PlaybackClock::Instance()->frame()},
                  {QStringLiteral("selected"), selected}},
      nullptr);
}

void TimelineCanvas::AddMarker() {
  if (view_ == nullptr || !view_->has_film) return;
  const double ms = std::round(PlaybackClock::Instance()->frame() *
                               kMsPerSecond / view_->fps);
  try {
    App::Get().session().edit_markers(ToRust(
        Compact(QJsonObject{{QStringLiteral("op"), QStringLiteral("add")},
                            {QStringLiteral("atMs"), ms}})));
  } catch (const rust::Error& error) {
    ShowError(QString::fromUtf8(error.what()));
  }
}

bool TimelineCanvas::ConfirmRemoval(std::size_t count) {
  QDialog dialog(QWidget::window());
  dialog.setObjectName(QStringLiteral("RemoveDialog"));
  dialog.setWindowTitle(QStringLiteral("Opsis"));
  dialog.setMinimumWidth(420);
  auto* layout = new QVBoxLayout(&dialog);
  layout->setContentsMargins(24, 22, 24, 20);
  layout->setSpacing(10);
  layout->addWidget(MakeLabel(QStringLiteral("Remove %1 nodes?").arg(count),
                              "title", &dialog));
  auto* about = MakeLabel(
      QStringLiteral(
          "These nodes only feed the selected clips. Undo brings them back."),
      "muted", &dialog);
  about->setWordWrap(true);
  layout->addWidget(about);
  auto* buttons = new QHBoxLayout();
  buttons->addStretch(1);
  auto* cancel =
      MakeButton(QStringLiteral("Cancel"), nullptr, QString(), &dialog);
  auto* remove =
      MakeButton(QStringLiteral("Remove"), "destructive", QString(), &dialog);
  buttons->addWidget(cancel);
  buttons->addWidget(remove);
  layout->addSpacing(8);
  layout->addLayout(buttons);
  connect(cancel, &QPushButton::clicked, &dialog, &QDialog::reject);
  connect(remove, &QPushButton::clicked, &dialog, &QDialog::accept);
  remove->setDefault(true);
  return dialog.exec() == QDialog::Accepted;
}

void TimelineCanvas::RemoveSelection() {
  if (!selected_marker_.isEmpty()) {
    try {
      App::Get().session().edit_markers(ToRust(
          Compact(QJsonObject{{QStringLiteral("op"), QStringLiteral("remove")},
                              {QStringLiteral("id"), selected_marker_}})));
    } catch (const rust::Error& error) {
      ShowError(QString::fromUtf8(error.what()));
    }
    selected_marker_.clear();
    return;
  }
  const bridge::RemovalView removal = App::Get().session().timeline_removal();
  if (removal.node_ids.empty()) return;
  if (removal.node_ids.size() > 1 && !ConfirmRemoval(removal.node_ids.size())) {
    return;
  }
  rust::Vec<rust::String> nodes;
  for (const auto& id : removal.node_ids) nodes.push_back(id);
  rust::Vec<rust::String> edges;
  for (const auto& id : removal.edge_ids) edges.push_back(id);
  try {
    App::Get().session().remove(std::move(nodes), std::move(edges));
  } catch (const rust::Error& error) {
    ShowError(QString::fromUtf8(error.what()));
  }
  Select({}, false);
}

void TimelineCanvas::keyPressEvent(QKeyEvent* event) {
  PlaybackClock* clock = PlaybackClock::Instance();
  const bool shift = (event->modifiers() & Qt::ShiftModifier) != 0;
  const bool primary =
      (event->modifiers() & (Qt::ControlModifier | Qt::MetaModifier)) != 0;
  const bool alt = (event->modifiers() & Qt::AltModifier) != 0;
  const int fps = clock->fps();
  switch (event->key()) {
    case Qt::Key_Equal:
    case Qt::Key_Plus:
      ZoomBy(kZoomStep);
      break;
    case Qt::Key_Minus:
      ZoomBy(1 / kZoomStep);
      break;
    case Qt::Key_X:
      if (!alt) return QAbstractScrollArea::keyPressEvent(event);
      clock->ClearRange();
      break;
    default:
      if (primary || alt) return QAbstractScrollArea::keyPressEvent(event);
      switch (event->key()) {
        case Qt::Key_Space:
          clock->Toggle();
          break;
        case Qt::Key_J:
          clock->Shuttle(-1);
          break;
        case Qt::Key_K:
          clock->Shuttle(0);
          break;
        case Qt::Key_L:
          clock->Shuttle(1);
          break;
        case Qt::Key_Left:
          clock->Step(shift ? -fps : -1);
          break;
        case Qt::Key_Right:
          clock->Step(shift ? fps : 1);
          break;
        case Qt::Key_Home:
          clock->Seek(0);
          break;
        case Qt::Key_End:
          clock->Seek(clock->duration() - 1);
          break;
        case Qt::Key_I:
          clock->SetIn(clock->frame());
          break;
        case Qt::Key_O:
          clock->SetOut(clock->frame());
          break;
        case Qt::Key_Z:
          Fit();
          break;
        case Qt::Key_S:
          SplitAtPlayhead();
          break;
        case Qt::Key_M:
          AddMarker();
          break;
        case Qt::Key_Delete:
        case Qt::Key_Backspace:
          RemoveSelection();
          break;
        case Qt::Key_V:
          SetTool(QStringLiteral("select"));
          break;
        case Qt::Key_B:
          SetTool(QStringLiteral("ripple"));
          break;
        case Qt::Key_N:
          SetTool(QStringLiteral("roll"));
          break;
        case Qt::Key_Y:
          SetTool(QStringLiteral("slip"));
          break;
        case Qt::Key_C:
          SetTool(QStringLiteral("blade"));
          break;
        case Qt::Key_R:
          SetTool(QStringLiteral("rate"));
          break;
        default:
          return QAbstractScrollArea::keyPressEvent(event);
      }
  }
  event->accept();
}

}
