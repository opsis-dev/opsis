#include "look_glyph.h"

#include <QLinearGradient>
#include <QPainter>
#include <QPainterPath>
#include <QtMath>
#include <algorithm>
#include <array>
#include <cmath>
#include <utility>

#include "app.h"
#include "motion.h"
#include "opsis-qt/src/bridge.rs.h"

namespace opsis {

namespace {

constexpr int kFrameMs = 16;
constexpr double kGlyphSize = 14.0;
constexpr double kViewBox = 24.0;
constexpr double kStroke = 2.0;
constexpr double kRingPen = 1.5;
constexpr double kFlareHalf = 8.0;
constexpr int kFlareAlpha = 200;
constexpr double kClockStart = 90.0;
constexpr double kWipeReach = 1.35;
constexpr double kBlurAlpha = 0.45;
constexpr double kShimmerDim = 0.3;
constexpr double kShimmerFrom = 1.5;
constexpr double kShimmerTravel = 2.0;
constexpr double kShimmerHalf = 3.0;
constexpr std::array<std::pair<double, double>, 3> kSmear = {
    {{-3.0, 0.45}, {-6.0, 0.25}, {-9.0, 0.12}}};

const bridge::MotionInfo& InfoOf(const QString& motion) {
  static QHash<QString, bridge::MotionInfo> infos;
  auto found = infos.find(motion);
  if (found == infos.end()) {
    found =
        infos.insert(motion, App::Get().session().motion_info(ToRust(motion)));
  }
  return *found;
}

void DrawGlyph(QPainter* painter, const QPointF& at, const QPixmap& glyph) {
  painter->drawPixmap(QRectF(at, QSizeF(kGlyphSize, kGlyphSize)), glyph,
                      QRectF(glyph.rect()));
}

void PaintMorph(QPainter* painter, const Look& look, double position) {
  const auto paths = App::Get().session().morph_frame(
      ToRust(look.glyph), ToRust(look.morph), position);
  QPainterPath drawn;
  for (const auto& path : paths) {
    const auto& points = path.points;
    for (std::size_t i = 0; i + 1 < points.size(); i += 2) {
      const QPointF point(points.at(i), points.at(i + 1));
      if (i == 0) {
        drawn.moveTo(point);
      } else {
        drawn.lineTo(point);
      }
    }
    if (path.closed && !points.empty()) drawn.closeSubpath();
  }
  painter->save();
  painter->scale(kGlyphSize / kViewBox, kGlyphSize / kViewBox);
  painter->setBrush(Qt::NoBrush);
  painter->setPen(
      QPen(Qt::black, kStroke, Qt::SolidLine, Qt::RoundCap, Qt::RoundJoin));
  painter->drawPath(drawn);
  painter->restore();
}

void PaintRingOrFlare(QPainter* painter, const QRectF& badge,
                      const QString& part, const bridge::PoseView& pose,
                      const QColor& accent) {
  painter->setOpacity(pose.opacity);
  if (part == QStringLiteral("ring")) {
    painter->setBrush(Qt::NoBrush);
    painter->setPen(QPen(accent, kRingPen));
    const double radius = badge.width() / 2 * pose.sx;
    painter->drawEllipse(badge.center(), radius, radius);
    return;
  }
  painter->setClipRect(badge);
  const double x = badge.center().x() + pose.tx * badge.width();
  QLinearGradient light(x - kFlareHalf, 0, x + kFlareHalf, 0);
  light.setColorAt(0.0, QColor(255, 255, 255, 0));
  light.setColorAt(0.5, QColor(255, 255, 255, kFlareAlpha));
  light.setColorAt(1.0, QColor(255, 255, 255, 0));
  painter->fillRect(badge, light);
}

QPainterPath MotionClip(const bridge::MotionInfo& info,
                        const bridge::PoseView& pose) {
  const QRectF local(0, 0, kGlyphSize, kGlyphSize);
  QPainterPath clip;
  clip.addRect(QRectF(0, pose.clip_top * kGlyphSize, kGlyphSize,
                      kGlyphSize * (1 - pose.clip_top)));
  QPainterPath iris;
  iris.addEllipse(local.center(), pose.iris * kGlyphSize,
                  pose.iris * kGlyphSize);
  clip = clip.intersected(iris);
  const QString mask = ToQ(info.mask);
  if (mask == QStringLiteral("clock")) {
    QPainterPath pie;
    pie.moveTo(local.center());
    pie.arcTo(local.adjusted(-kGlyphSize, -kGlyphSize, kGlyphSize, kGlyphSize),
              kClockStart, -pose.sweep);
    pie.closeSubpath();
    clip = clip.intersected(pie);
  } else if (mask == QStringLiteral("wipe")) {
    QPainterPath shown;
    shown.addRect(QRectF(
        0, 0, kGlyphSize * std::clamp(kWipeReach - 2 * pose.mask, 0.0, 1.0),
        kGlyphSize));
    clip = clip.intersected(shown);
  }
  if (ToQ(info.part) == QStringLiteral("shapes")) {
    QPainterPath drawn;
    drawn.addRect(QRectF(0, 0, kGlyphSize * (1 - pose.dash), kGlyphSize));
    clip = clip.intersected(drawn);
  }
  return clip;
}

void PaintMotion(QPainter* painter, const QRectF& badge, const Look& look,
                 double elapsed, int glyph_px, const QColor& accent) {
  App& app = App::Get();
  const bridge::MotionInfo& info = InfoOf(look.motion);
  const bridge::PoseView pose =
      app.session().motion_pose(ToRust(look.motion), elapsed);
  const QPixmap glyph = app.Glyph(look.glyph, Qt::black, glyph_px);
  const QString part = ToQ(info.part);
  if (part == QStringLiteral("ring") || part == QStringLiteral("flare")) {
    DrawGlyph(painter, QPointF(0, 0), glyph);
    PaintRingOrFlare(painter, badge, part, pose, accent);
    return;
  }
  if (info.clip) painter->setClipRect(badge, Qt::IntersectClip);
  const QPointF origin(info.origin_x * kGlyphSize, info.origin_y * kGlyphSize);
  QTransform transform;
  transform.translate(origin.x() + (pose.tx + pose.split_x) * kGlyphSize,
                      origin.y() + (pose.ty + pose.split_y) * kGlyphSize);
  transform.rotate(pose.rotate);
  transform.scale(pose.sx * std::cos(qDegreesToRadians(pose.tilt_y)),
                  pose.sy * std::cos(qDegreesToRadians(pose.tilt_x)));
  transform.translate(-origin.x(), -origin.y());
  painter->setTransform(transform, true);
  painter->setOpacity(painter->opacity() * pose.opacity);
  painter->setClipPath(MotionClip(info, pose), Qt::IntersectClip);
  if (pose.rgb != 0) {
    DrawGlyph(painter, QPointF(-pose.rgb, 0),
              app.Glyph(look.glyph, QColor(0xff, 0x2d, 0x55), glyph_px));
    DrawGlyph(painter, QPointF(pose.rgb, 0),
              app.Glyph(look.glyph, QColor(0x00, 0xe5, 0xff), glyph_px));
  }
  const double base = painter->opacity();
  for (const auto& [shift, alpha] : kSmear) {
    if (pose.smear <= 0) break;
    painter->setOpacity(base * alpha);
    DrawGlyph(painter, QPointF(shift * pose.smear, 0), glyph);
  }
  if (pose.blur > 0) {
    painter->setOpacity(base * kBlurAlpha);
    for (const QPointF& shift :
         {QPointF(-pose.blur / 2, 0), QPointF(pose.blur / 2, 0),
          QPointF(0, -pose.blur / 2), QPointF(0, pose.blur / 2)}) {
      DrawGlyph(painter, shift, glyph);
    }
  }
  painter->setOpacity(base * (pose.blur > 0 ? kBlurAlpha : 1.0));
  if (ToQ(info.mask) == QStringLiteral("shimmer")) {
    painter->setOpacity(base * kShimmerDim);
    DrawGlyph(painter, QPointF(0, 0), glyph);
    const double centre =
        kGlyphSize * (kShimmerFrom - kShimmerTravel * pose.mask);
    painter->setClipRect(
        QRectF(centre - kShimmerHalf, 0, 2 * kShimmerHalf, kGlyphSize),
        Qt::IntersectClip);
    painter->setOpacity(base);
  }
  DrawGlyph(painter, QPointF(0, 0), glyph);
}

}

LookPlayer::LookPlayer(QObject* parent) : QObject(parent) {
  timer_.setInterval(kFrameMs);
  connect(&timer_, &QTimer::timeout, this, &LookPlayer::Tick);
}

void LookPlayer::Run() {
  if (!clock_.isValid()) clock_.start();
  if (!timer_.isActive()) {
    last_tick_ = clock_.elapsed();
    timer_.start();
  }
}

void LookPlayer::Enter(const QString& id, const Look& look) {
  const bool reduced = ReducedMotion();
  if (!look.morph.isEmpty()) {
    auto found = springs_.find(id);
    if (found == springs_.end()) found = springs_.insert(id, Spring{});
    Spring& spring = *found;
    spring.target = 1;
    spring.settled = reduced;
    if (reduced) spring.position = 1;
    if (reduced) emit Changed(id);
    if (!reduced) Run();
  }
  if (look.motion.isEmpty() || reduced || started_.contains(id)) return;
  Run();
  started_.insert(id, clock_.elapsed());
  lengths_.insert(id, static_cast<qint64>(InfoOf(look.motion).ms));
}

void LookPlayer::Leave(const QString& id) {
  auto found = springs_.find(id);
  if (found == springs_.end()) return;
  if (ReducedMotion()) {
    springs_.erase(found);
    emit Changed(id);
    return;
  }
  found->target = 0;
  found->settled = false;
  Run();
}

void LookPlayer::Tick() {
  const qint64 now = clock_.elapsed();
  const auto dt = static_cast<double>(now - last_tick_);
  last_tick_ = now;
  for (auto it = started_.begin(); it != started_.end();) {
    const QString id = it.key();
    const bool over = now - it.value() > lengths_.value(id);
    it = over ? started_.erase(it) : std::next(it);
    emit Changed(id);
  }
  for (auto it = springs_.begin(); it != springs_.end();) {
    Spring& spring = it.value();
    if (spring.settled) {
      ++it;
      continue;
    }
    const bridge::MorphStep step = App::Get().session().morph_step(
        spring.position, spring.velocity, spring.target, dt);
    spring.position = step.position;
    spring.velocity = step.velocity;
    spring.settled = step.settled;
    const QString id = it.key();
    const bool rest = step.settled && spring.target == 0;
    it = rest ? springs_.erase(it) : std::next(it);
    emit Changed(id);
  }
  const bool moving = std::any_of(springs_.cbegin(), springs_.cend(),
                                  [](const Spring& s) { return !s.settled; });
  if (started_.isEmpty() && !moving) timer_.stop();
}

LookFrame LookPlayer::FrameOf(const QString& id) const {
  LookFrame frame;
  frame.motion_ms = MotionElapsed(id);
  const auto spring = springs_.find(id);
  if (spring != springs_.end()) {
    frame.morph_at = spring->position;
    frame.morphing = !spring->settled;
  }
  return frame;
}

double LookPlayer::MotionElapsed(const QString& id) const {
  const auto found = started_.find(id);
  if (found == started_.end()) return -1;
  return static_cast<double>(clock_.elapsed() - *found);
}

double LookPlayer::MorphAt(const QString& id) const {
  const auto found = springs_.find(id);
  return found == springs_.end() ? 0 : found->position;
}

void PaintLookGlyph(QPainter* painter, const QRectF& badge, const QRectF& box,
                    const Look& look, const LookFrame& frame,
                    const QColor& accent) {
  const double scale = box.width() / kGlyphSize;
  const int glyph_px = qRound(box.width());
  painter->save();
  painter->translate(box.topLeft());
  painter->scale(scale, scale);
  const QRectF local_badge((badge.x() - box.x()) / scale,
                           (badge.y() - box.y()) / scale, badge.width() / scale,
                           badge.height() / scale);
  if (!look.morph.isEmpty() && frame.morphing) {
    PaintMorph(painter, look, frame.morph_at);
  } else if (!look.morph.isEmpty() && frame.morph_at >= 1) {
    DrawGlyph(painter, QPointF(0, 0),
              App::Get().Glyph(look.morph, Qt::black, glyph_px));
  } else if (frame.motion_ms >= 0 && !look.motion.isEmpty()) {
    PaintMotion(painter, local_badge, look, frame.motion_ms, glyph_px, accent);
  } else {
    DrawGlyph(painter, QPointF(0, 0),
              App::Get().Glyph(look.glyph, Qt::black, glyph_px));
  }
  painter->restore();
}

}
