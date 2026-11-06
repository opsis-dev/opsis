#include "graph_canvas.h"

#include <QApplication>
#include <QClipboard>
#include <QConicalGradient>
#include <QElapsedTimer>
#include <QFontMetricsF>
#include <QGraphicsSceneHoverEvent>
#include <QGraphicsSceneMouseEvent>
#include <QGuiApplication>
#include <QHBoxLayout>
#include <QKeyEvent>
#include <QLinearGradient>
#include <QMimeData>
#include <QNativeGestureEvent>
#include <QPainter>
#include <QPainterPathStroker>
#include <QScrollBar>
#include <QStyleOptionGraphicsItem>
#include <QToolButton>
#include <QVariantAnimation>
#include <QtMath>
#include <algorithm>
#include <array>
#include <cmath>
#include <iterator>
#include <vector>

#include "app.h"
#include "event_hub.h"
#include "motion.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr double kRadius = 8.0;
constexpr double kHeader = 44.0;
constexpr double kPortRadius = 5.0;
constexpr double kPortHit = 9.0;
constexpr double kRowHeight = 22.0;
constexpr double kStepOffset = 20.0;
constexpr double kCorner = 8.0;
constexpr double kGridGap = 20.0;
constexpr double kMinZoom = 0.1;
constexpr double kMaxZoom = 2.5;
constexpr int kRevealMs = 300;
constexpr double kMinRevealZoom = 0.8;
constexpr double kRevealPadding = 0.4;
constexpr double kGroupBorderMix = 0.53;
constexpr double kGroupFillMix = 0.06;
constexpr double kGroupRadius = 12.0;
constexpr double kGroupTagPadX = 12.0;
constexpr double kGroupTagHeight = 24.0;
constexpr double kGroupMinWidth = 240.0;
constexpr double kGroupMinHeight = 160.0;
constexpr double kHandle = 7.0;
constexpr double kHandleReach = 6.0;
constexpr double kNoteBorderMix = 0.4;
constexpr double kNoteFillMix = 0.1;
constexpr double kNoteInkMix = 0.3;
constexpr double kGlyphInset = 5.0;
constexpr double kGlyphSize = 14.0;
constexpr int kGlyphPx = 14;
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
constexpr int kMotionFrameMs = 16;
constexpr std::array<std::pair<double, double>, 3> kSmear = {
    {{-3.0, 0.45}, {-6.0, 0.25}, {-9.0, 0.12}}};
constexpr int kZoomButtonWidth = 56;
constexpr int kFloatSize = 36;
constexpr int kOverlayMargin = 12;
constexpr double kRingOut = 3.0;
constexpr double kBadgeHeight = 20.0;
constexpr double kBadgeIcon = 12.0;
constexpr double kBadgePad = 6.0;
constexpr int kBadgeFillAlpha = 40;
constexpr double kProgressBar = 4.0;
constexpr double kProgressHeight = 28.0;
constexpr int kSpinFrameMs = 33;
constexpr double kSpinStep = 6.0;
constexpr double kDetailZoom = 0.12;
constexpr int kToastMs = 3200;
constexpr int kMinimapWidth = 200;
constexpr int kMinimapHeight = 140;
constexpr QLatin1StringView kNodeMime("application/x-opsis-node");

QColor Mix(const QColor& color, double alpha) {
  QColor mixed = color;
  mixed.setAlphaF(static_cast<float>(alpha));
  return mixed;
}

QPainterPath RoundedPolyline(const std::vector<QPointF>& points) {
  QPainterPath path(points.front());
  for (std::size_t i = 1; i + 1 < points.size(); ++i) {
    const QPointF prev = points.at(i - 1);
    const QPointF corner = points.at(i);
    const QPointF next = points.at(i + 1);
    const double into = QLineF(prev, corner).length();
    const double out = QLineF(corner, next).length();
    const double radius = std::min({kCorner, into / 2, out / 2});
    if (radius <= 0.5) {
      path.lineTo(corner);
      continue;
    }
    const QPointF before = corner + (prev - corner) * (radius / into);
    const QPointF after = corner + (next - corner) * (radius / out);
    path.lineTo(before);
    path.quadTo(corner, after);
  }
  path.lineTo(points.back());
  return path;
}

QString Elided(const QFont& font, const QString& text, double width) {
  return QFontMetrics(font).elidedText(text, Qt::ElideRight,
                                       static_cast<int>(width));
}

}

CardItem::CardItem(GraphCanvas* canvas, const bridge::CardView& card)
    : canvas_(canvas), card_(card), id_(ToQ(card.id)) {
  setFlags(ItemIsSelectable | ItemIsMovable | ItemSendsGeometryChanges);
  setAcceptHoverEvents(true);
  if (qEnvironmentVariableIsEmpty("OPSIS_QT_NO_ITEM_CACHE")) {
    setCacheMode(DeviceCoordinateCache);
  }
  SetCard(card);
}

void CardItem::SetCard(const bridge::CardView& card) {
  prepareGeometryChange();
  card_ = card;
  setZValue(card.kind == bridge::CardKind::Group ? -1 : 1);
  setPos(card.x, card.y);
  update();
}

QRectF CardItem::boundingRect() const {
  const double margin = std::max(kRingOut, kHandleReach);
  return {-kPortHit, -margin, card_.width + 2 * kPortHit,
          card_.height + 2 * margin};
}

QPainterPath CardItem::shape() const {
  QPainterPath path;
  if (card_.kind == bridge::CardKind::Group && isSelected()) {
    path.addRect(QRectF(0, 0, card_.width, card_.height)
                     .adjusted(-kHandleReach, -kHandleReach, kHandleReach,
                               kHandleReach));
    return path;
  }
  path.addRoundedRect(QRectF(0, 0, card_.width, card_.height), kRadius,
                      kRadius);
  for (const auto& port : card_.inputs) {
    path.addEllipse(QPointF(0, port.y), kPortHit, kPortHit);
  }
  for (const auto& port : card_.outputs) {
    path.addEllipse(QPointF(card_.width, port.y), kPortHit, kPortHit);
  }
  return path;
}

std::optional<PortHit> CardItem::PortAt(const QPointF& local) const {
  auto check = [&](const rust::Vec<bridge::PortView>& ports, double x,
                   bool output) -> std::optional<PortHit> {
    for (const auto& port : ports) {
      if (QLineF(local, QPointF(x, port.y)).length() <= kPortHit) {
        return PortHit{id_, ToQ(port.id), output, mapToScene(x, port.y)};
      }
    }
    return std::nullopt;
  };
  if (auto hit = check(card_.outputs, card_.width, true)) return hit;
  return check(card_.inputs, 0, false);
}

QPointF CardItem::PortPoint(const QString& port, bool output) const {
  const auto& ports = output ? card_.outputs : card_.inputs;
  for (const auto& candidate : ports) {
    if (ToQ(candidate.id) == port) {
      return mapToScene(output ? card_.width : 0, candidate.y);
    }
  }
  return mapToScene(output ? card_.width : 0, kHeader / 2);
}

QVariant CardItem::itemChange(GraphicsItemChange change,
                              const QVariant& value) {
  if (change == ItemPositionChange && card_.kind == bridge::CardKind::Group &&
      resizing_ == 0U) {
    canvas_->MoveChildren(id_, value.toPointF() - pos());
  }
  if (change == ItemPositionHasChanged) canvas_->CardMoved(id_);
  if (change == ItemSelectedHasChanged) {
    prepareGeometryChange();
    update();
  }
  return QGraphicsItem::itemChange(change, value);
}

void CardItem::mousePressEvent(QGraphicsSceneMouseEvent* event) {
  if (event->button() == Qt::LeftButton) {
    if (const unsigned edges = EdgesAt(event->pos()); edges != 0) {
      resizing_ = edges;
      resize_from_ = QRectF(pos(), QSizeF(card_.width, card_.height));
      press_at_ = event->scenePos();
      event->accept();
      return;
    }
    if (auto hit = PortAt(event->pos())) {
      connecting_ = true;
      canvas_->BeginConnect(*hit);
      event->accept();
      return;
    }
  }
  QGraphicsItem::mousePressEvent(event);
}

void CardItem::mouseMoveEvent(QGraphicsSceneMouseEvent* event) {
  if (resizing_ != 0U) {
    const QPointF delta = event->scenePos() - press_at_;
    QRectF rect = resize_from_;
    if ((resizing_ & kLeft) != 0U) {
      rect.setLeft(
          std::min(rect.left() + delta.x(), rect.right() - kGroupMinWidth));
    }
    if ((resizing_ & kRight) != 0U) {
      rect.setRight(
          std::max(rect.right() + delta.x(), rect.left() + kGroupMinWidth));
    }
    if ((resizing_ & kTop) != 0U) {
      rect.setTop(
          std::min(rect.top() + delta.y(), rect.bottom() - kGroupMinHeight));
    }
    if ((resizing_ & kBottom) != 0U) {
      rect.setBottom(
          std::max(rect.bottom() + delta.y(), rect.top() + kGroupMinHeight));
    }
    prepareGeometryChange();
    card_.width = rect.width();
    card_.height = rect.height();
    setPos(rect.topLeft());
    update();
    return;
  }
  if (connecting_) {
    canvas_->UpdateConnect(event->scenePos());
    return;
  }
  QGraphicsItem::mouseMoveEvent(event);
}

void CardItem::mouseReleaseEvent(QGraphicsSceneMouseEvent* event) {
  if (resizing_ != 0U) {
    resizing_ = 0;
    canvas_->ResizeGroup(id_, QRectF(pos(), QSizeF(card_.width, card_.height)));
    return;
  }
  if (connecting_) {
    connecting_ = false;
    canvas_->FinishConnect(event->scenePos());
    return;
  }
  QGraphicsItem::mouseReleaseEvent(event);
  canvas_->CommitMoves();
}

void CardItem::hoverEnterEvent(QGraphicsSceneHoverEvent* event) {
  if (card_.kind == bridge::CardKind::Card) canvas_->PlayMotion(id_);
  QGraphicsItem::hoverEnterEvent(event);
}

void CardItem::PaintGlyph(QPainter* painter, const QRectF& badge,
                          const QColor& accent) {
  App& app = App::Get();
  const QString name = ToQ(card_.glyph);
  const QRectF box(badge.x() + kGlyphInset, badge.y() + kGlyphInset, kGlyphSize,
                   kGlyphSize);
  const double elapsed = canvas_->MotionElapsed(id_);
  if (elapsed < 0) {
    painter->drawPixmap(box.topLeft(), app.Glyph(name, Qt::black, kGlyphPx));
    return;
  }
  const QString motion = ToQ(card_.motion);
  const bridge::MotionInfo& info = canvas_->MotionInfoOf(motion);
  const bridge::PoseView pose =
      app.session().motion_pose(card_.motion, elapsed);
  const QString part = ToQ(info.part);
  painter->save();
  if (part == QStringLiteral("ring") || part == QStringLiteral("flare")) {
    painter->drawPixmap(box.topLeft(), app.Glyph(name, Qt::black, kGlyphPx));
    painter->setOpacity(pose.opacity);
    if (part == QStringLiteral("ring")) {
      painter->setBrush(Qt::NoBrush);
      painter->setPen(QPen(accent, kRingPen));
      const double radius = badge.width() / 2 * pose.sx;
      painter->drawEllipse(badge.center(), radius, radius);
    } else {
      painter->setClipRect(badge);
      const double x = badge.center().x() + pose.tx * badge.width();
      QLinearGradient light(x - kFlareHalf, 0, x + kFlareHalf, 0);
      light.setColorAt(0.0, QColor(255, 255, 255, 0));
      light.setColorAt(0.5, QColor(255, 255, 255, kFlareAlpha));
      light.setColorAt(1.0, QColor(255, 255, 255, 0));
      painter->fillRect(badge, light);
    }
    painter->restore();
    return;
  }
  if (info.clip) painter->setClipRect(badge, Qt::IntersectClip);
  const QPointF origin(info.origin_x * kGlyphSize, info.origin_y * kGlyphSize);
  QTransform transform;
  transform.translate(
      box.x() + origin.x() + (pose.tx + pose.split_x) * kGlyphSize,
      box.y() + origin.y() + (pose.ty + pose.split_y) * kGlyphSize);
  transform.rotate(pose.rotate);
  transform.scale(pose.sx * std::cos(qDegreesToRadians(pose.tilt_y)),
                  pose.sy * std::cos(qDegreesToRadians(pose.tilt_x)));
  transform.translate(-origin.x(), -origin.y());
  painter->setTransform(transform, true);
  painter->setOpacity(painter->opacity() * pose.opacity);
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
  if (part == QStringLiteral("shapes")) {
    QPainterPath drawn;
    drawn.addRect(QRectF(0, 0, kGlyphSize * (1 - pose.dash), kGlyphSize));
    clip = clip.intersected(drawn);
  }
  painter->setClipPath(clip, Qt::IntersectClip);
  const QPixmap glyph = app.Glyph(name, Qt::black, kGlyphPx);
  if (pose.rgb != 0) {
    painter->drawPixmap(QPointF(-pose.rgb, 0),
                        app.Glyph(name, QColor(0xff, 0x2d, 0x55), kGlyphPx));
    painter->drawPixmap(QPointF(pose.rgb, 0),
                        app.Glyph(name, QColor(0x00, 0xe5, 0xff), kGlyphPx));
  }
  const double base = painter->opacity();
  for (const auto& [shift, alpha] : kSmear) {
    if (pose.smear <= 0) break;
    painter->setOpacity(base * alpha);
    painter->drawPixmap(QPointF(shift * pose.smear, 0), glyph);
  }
  if (pose.blur > 0) {
    painter->setOpacity(base * kBlurAlpha);
    for (const QPointF& shift :
         {QPointF(-pose.blur / 2, 0), QPointF(pose.blur / 2, 0),
          QPointF(0, -pose.blur / 2), QPointF(0, pose.blur / 2)}) {
      painter->drawPixmap(shift, glyph);
    }
  }
  painter->setOpacity(base * (pose.blur > 0 ? kBlurAlpha : 1.0));
  if (mask == QStringLiteral("shimmer")) {
    painter->setOpacity(base * kShimmerDim);
    painter->drawPixmap(QPointF(0, 0), glyph);
    const double centre =
        kGlyphSize * (kShimmerFrom - kShimmerTravel * pose.mask);
    painter->setClipRect(
        QRectF(centre - kShimmerHalf, 0, 2 * kShimmerHalf, kGlyphSize),
        Qt::IntersectClip);
    painter->setOpacity(base);
  }
  painter->drawPixmap(QPointF(0, 0), glyph);
  painter->restore();
}

void CardItem::hoverMoveEvent(QGraphicsSceneHoverEvent* event) {
  const unsigned edges = EdgesAt(event->pos());
  const bool diagonal_down =
      edges == (kLeft | kTop) || edges == (kRight | kBottom);
  const bool diagonal_up =
      edges == (kRight | kTop) || edges == (kLeft | kBottom);
  if (diagonal_down) {
    setCursor(Qt::SizeFDiagCursor);
  } else if (diagonal_up) {
    setCursor(Qt::SizeBDiagCursor);
  } else if ((edges & (kLeft | kRight)) != 0U) {
    setCursor(Qt::SizeHorCursor);
  } else if ((edges & (kTop | kBottom)) != 0U) {
    setCursor(Qt::SizeVerCursor);
  } else {
    unsetCursor();
  }
  QGraphicsItem::hoverMoveEvent(event);
}

void CardItem::paint(QPainter* painter, const QStyleOptionGraphicsItem* option,
                     QWidget*) {
  painter->setRenderHint(QPainter::Antialiasing);
  switch (card_.kind) {
    case bridge::CardKind::Group:
      PaintGroup(painter);
      return;
    case bridge::CardKind::Note:
      PaintNote(painter);
      return;
    default:
      break;
  }
  const double lod =
      option->levelOfDetailFromTransform(painter->worldTransform());
  PaintCard(painter, lod >= kDetailZoom);
}

void CardItem::PaintCard(QPainter* painter, bool detailed) {
  App& app = App::Get();
  const QRectF body(0, 0, card_.width, card_.height);
  const QColor accent = QColor::fromString(ToQ(card_.color));
  if (card_.disabled) painter->setOpacity(0.5);
  painter->setPen(QPen(app.Token("border"), 1));
  painter->setBrush(app.Token("card"));
  painter->drawRoundedRect(body.adjusted(0.5, 0.5, -0.5, -0.5), kRadius,
                           kRadius);
  PaintRing(painter, body);
  if (!detailed) {
    painter->setPen(Qt::NoPen);
    painter->setBrush(accent);
    painter->drawRoundedRect(QRectF(0, 0, card_.width, kHeader), kRadius,
                             kRadius);
    if (isSelected()) {
      painter->setBrush(Qt::NoBrush);
      painter->setPen(QPen(app.Token("selection"), 4));
      painter->drawRoundedRect(body, kRadius, kRadius);
    }
    return;
  }
  painter->setPen(QPen(app.Token("border"), 1));
  painter->drawLine(QPointF(1, kHeader), QPointF(card_.width - 1, kHeader));
  const QRectF badge(12, 10, 24, 24);
  painter->setPen(Qt::NoPen);
  painter->setBrush(accent);
  painter->drawRoundedRect(badge, 6, 6);
  PaintGlyph(painter, badge, accent);
  QFont title = QApplication::font();
  title.setPixelSize(14);
  title.setWeight(QFont::DemiBold);
  const double badge_width = PaintBadge(painter);
  const double title_width = card_.width - 56 - badge_width;
  painter->setFont(title);
  painter->setPen(app.Token("foreground"));
  painter->drawText(QRectF(44, 0, title_width, kHeader),
                    Qt::AlignVCenter | Qt::AlignLeft,
                    Elided(title, ToQ(card_.title), title_width));
  QFont small = QApplication::font();
  small.setPixelSize(12);
  painter->setFont(small);
  const double half = card_.width / 2 - 18;
  for (const auto& port : card_.inputs) {
    painter->setPen(QPen(app.Token("card"), 2));
    painter->setBrush(QColor::fromString(ToQ(port.color)));
    painter->drawEllipse(QPointF(0, port.y), kPortRadius, kPortRadius);
    painter->setPen(port.optional ? app.Token("muted-foreground")
                                  : app.Token("foreground"));
    painter->drawText(QRectF(14, port.y - 10, half, 20),
                      Qt::AlignVCenter | Qt::AlignLeft,
                      Elided(small, ToQ(port.label), half));
  }
  for (const auto& port : card_.outputs) {
    painter->setPen(QPen(app.Token("card"), 2));
    painter->setBrush(QColor::fromString(ToQ(port.color)));
    painter->drawEllipse(QPointF(card_.width, port.y), kPortRadius,
                         kPortRadius);
    painter->setPen(app.Token("foreground"));
    painter->drawText(QRectF(card_.width - 14 - half, port.y - 10, half, 20),
                      Qt::AlignVCenter | Qt::AlignRight,
                      Elided(small, ToQ(port.label), half));
  }
  const std::size_t port_rows =
      std::max(card_.inputs.size(), card_.outputs.size());
  double y = kHeader + 8 + static_cast<double>(port_rows) * 24 +
             (port_rows > 0 ? 4 : 0);
  for (const auto& row : card_.rows) {
    const QColor ink =
        row.muted ? app.Token("muted-foreground") : app.Token("foreground");
    painter->drawPixmap(
        QPointF(12, y + 4),
        app.Glyph(ToQ(row.icon), app.Token("muted-foreground"), 14));
    painter->setPen(ink);
    painter->drawText(QRectF(32, y, card_.width - 44, kRowHeight),
                      Qt::AlignVCenter | Qt::AlignLeft,
                      Elided(small, ToQ(row.text), card_.width - 44));
    y += kRowHeight;
  }
  if (card_.progress >= 0) {
    const QRectF track(12, y + 6, card_.width - 24, kProgressBar);
    painter->setPen(Qt::NoPen);
    painter->setBrush(app.Token("muted"));
    painter->drawRoundedRect(track, kProgressBar / 2, kProgressBar / 2);
    painter->setBrush(app.Token("primary"));
    painter->drawRoundedRect(
        QRectF(track.topLeft(),
               QSizeF(track.width() * card_.progress, track.height())),
        kProgressBar / 2, kProgressBar / 2);
    if (!card_.progress_message.empty()) {
      QFont tiny = small;
      tiny.setPixelSize(11);
      painter->setFont(tiny);
      painter->setPen(app.Token("muted-foreground"));
      painter->drawText(
          QRectF(12, y + 10, card_.width - 24, 16),
          Qt::AlignVCenter | Qt::AlignLeft,
          Elided(tiny, ToQ(card_.progress_message), card_.width - 24));
      painter->setFont(small);
    }
    y += kProgressHeight;
  }
  const bool error = !card_.problem.empty();
  if (error || !card_.warning.empty()) {
    painter->setPen(error ? app.Token("destructive")
                          : app.Token("muted-foreground"));
    painter->drawText(QRectF(12, y, card_.width - 24, kRowHeight * 2),
                      Qt::AlignTop | Qt::AlignLeft | Qt::TextWordWrap,
                      ToQ(error ? card_.problem : card_.warning));
  }
  if (isSelected()) {
    painter->setOpacity(1.0);
    painter->setBrush(Qt::NoBrush);
    painter->setPen(QPen(app.Token("selection"), 2));
    painter->drawRoundedRect(body.adjusted(-2, -2, 2, 2), kRadius + 2,
                             kRadius + 2);
  }
}

void CardItem::PaintRing(QPainter* painter, const QRectF& body) {
  if (card_.ring.empty()) return;
  App& app = App::Get();
  const QRectF ring = body.adjusted(-kRingOut, -kRingOut, kRingOut, kRingOut);
  QPen pen(Qt::NoPen);
  if (card_.ring == "loading") {
    QConicalGradient sweep(ring.center(), -canvas_->spin_angle());
    const QColor bright = app.Token("primary");
    QColor faint = bright;
    faint.setAlpha(0);
    sweep.setColorAt(0.0, bright);
    sweep.setColorAt(0.35, faint);
    sweep.setColorAt(0.65, faint);
    sweep.setColorAt(1.0, bright);
    pen = QPen(QBrush(sweep), 2);
  } else {
    pen = QPen(app.Token(card_.ring == "error" ? "destructive" : "success"), 2);
  }
  painter->save();
  painter->setBrush(Qt::NoBrush);
  painter->setPen(pen);
  painter->drawRoundedRect(ring, kRadius + kRingOut, kRadius + kRingOut);
  painter->restore();
}

double CardItem::PaintBadge(QPainter* painter) {
  if (card_.badge.empty()) return 0;
  App& app = App::Get();
  QFont font = QApplication::font();
  font.setPixelSize(11);
  font.setWeight(QFont::Medium);
  const QString label = ToQ(card_.badge);
  const double text_width = QFontMetricsF(font).horizontalAdvance(label);
  const double width = text_width + kBadgeIcon + 3 * kBadgePad;
  const QRectF pill(card_.width - 12 - width, (kHeader - kBadgeHeight) / 2,
                    width, kBadgeHeight);
  const QString tone = ToQ(card_.badge_tone);
  QColor ink = app.Token("foreground");
  painter->setBrush(Qt::NoBrush);
  painter->setPen(QPen(app.Token("border"), 1));
  if (tone == QStringLiteral("secondary")) {
    painter->setPen(Qt::NoPen);
    painter->setBrush(app.Token("secondary"));
  } else if (tone == QStringLiteral("destructive")) {
    ink = app.Token("destructive");
    QColor fill = ink;
    fill.setAlpha(kBadgeFillAlpha);
    painter->setPen(Qt::NoPen);
    painter->setBrush(fill);
  }
  painter->drawRoundedRect(pill, kBadgeHeight / 2, kBadgeHeight / 2);
  painter->drawPixmap(
      QPointF(pill.x() + kBadgePad, pill.center().y() - kBadgeIcon / 2),
      app.Glyph(ToQ(card_.badge_icon), ink, static_cast<int>(kBadgeIcon)));
  painter->setFont(font);
  painter->setPen(ink);
  painter->drawText(QRectF(pill.x() + 2 * kBadgePad + kBadgeIcon, pill.y(),
                           text_width + kBadgePad, pill.height()),
                    Qt::AlignVCenter | Qt::AlignLeft, label);
  return width + 8;
}

void CardItem::PaintGroup(QPainter* painter) {
  App& app = App::Get();
  QColor color = QColor::fromString(ToQ(card_.text));
  if (!color.isValid()) color = app.Token("muted-foreground");
  const QRectF body(1, 1, card_.width - 2, card_.height - 2);
  QPen border(Mix(color, kGroupBorderMix), 2, Qt::DashLine);
  border.setDashPattern({3, 3});
  painter->setPen(border);
  painter->setBrush(Mix(color, kGroupFillMix));
  painter->drawRoundedRect(body, kGroupRadius, kGroupRadius);
  QFont font = QApplication::font();
  font.setPixelSize(12);
  font.setWeight(QFont::DemiBold);
  const QString title = ToQ(card_.title);
  const double width =
      QFontMetricsF(font).horizontalAdvance(title) + 2 * kGroupTagPadX;
  const QRectF tag(0, 0, std::min(width, card_.width), kGroupTagHeight);
  QPainterPath shape;
  shape.moveTo(tag.topLeft() + QPointF(kGroupRadius, 0));
  shape.lineTo(tag.topRight());
  shape.lineTo(tag.right(), tag.bottom() - kGroupRadius);
  shape.quadTo(tag.bottomRight(), tag.bottomRight() - QPointF(kGroupRadius, 0));
  shape.lineTo(tag.bottomLeft());
  shape.lineTo(tag.left(), tag.top() + kGroupRadius);
  shape.quadTo(tag.topLeft(), tag.topLeft() + QPointF(kGroupRadius, 0));
  painter->setPen(Qt::NoPen);
  painter->setBrush(color);
  painter->drawPath(shape);
  painter->setFont(font);
  painter->setPen(Qt::black);
  painter->drawText(tag, Qt::AlignCenter, title);
  if (isSelected()) PaintResizeHandles(painter);
}

void CardItem::PaintResizeHandles(QPainter* painter) {
  App& app = App::Get();
  painter->setPen(QPen(app.Token("ring"), 1));
  painter->setBrush(Qt::NoBrush);
  painter->drawRect(QRectF(0, 0, card_.width, card_.height));
  painter->setBrush(app.Token("foreground"));
  const double half = kHandle / 2;
  for (const QPointF& corner :
       {QPointF(0, 0), QPointF(card_.width, 0), QPointF(0, card_.height),
        QPointF(card_.width, card_.height)}) {
    painter->drawRect(
        QRectF(corner - QPointF(half, half), QSizeF(kHandle, kHandle)));
  }
}

unsigned CardItem::EdgesAt(const QPointF& local) const {
  if (card_.kind != bridge::CardKind::Group || !isSelected()) return 0;
  unsigned edges = 0;
  if (std::abs(local.x()) <= kHandleReach) edges |= kLeft;
  if (std::abs(local.x() - card_.width) <= kHandleReach) edges |= kRight;
  if (std::abs(local.y()) <= kHandleReach) edges |= kTop;
  if (std::abs(local.y() - card_.height) <= kHandleReach) edges |= kBottom;
  return edges;
}

void CardItem::PaintNote(QPainter* painter) {
  App& app = App::Get();
  QColor color = QColor::fromString(ToQ(card_.color));
  if (!color.isValid()) color = app.Token("muted-foreground");
  const QRectF body(0.5, 0.5, card_.width - 1, card_.height - 1);
  painter->setPen(
      QPen(isSelected() ? app.Token("selection") : Mix(color, kNoteBorderMix),
           isSelected() ? 2 : 1));
  painter->setBrush(Mix(color, kNoteFillMix));
  painter->drawRoundedRect(body, kRadius, kRadius);
  QFont font = QApplication::font();
  font.setPixelSize(12);
  painter->setFont(font);
  painter->setPen(QColor::fromRgbF(
      static_cast<float>(color.redF() * kNoteInkMix + (1 - kNoteInkMix)),
      static_cast<float>(color.greenF() * kNoteInkMix + (1 - kNoteInkMix)),
      static_cast<float>(color.blueF() * kNoteInkMix + (1 - kNoteInkMix))));
  painter->drawText(body.adjusted(12, 12, -12, -12),
                    Qt::AlignTop | Qt::AlignLeft | Qt::TextWordWrap,
                    ToQ(card_.text));
}

EdgeItem::EdgeItem(const bridge::EdgeView& edge) {
  setFlags(ItemIsSelectable);
  setZValue(0);
  SetEdge(edge);
}

void EdgeItem::SetEdge(const bridge::EdgeView& edge) {
  edge_ = edge;
  id_ = ToQ(edge.id);
  source_ = ToQ(edge.source);
  target_ = ToQ(edge.target);
  update();
}

void EdgeItem::Route(const QPointF& from, const QPointF& to) {
  std::vector<QPointF> points;
  const double left = from.x() + kStepOffset;
  const double right = to.x() - kStepOffset;
  points.push_back(from);
  if (right > left) {
    const double middle = left + (right - left) * edge_.step;
    points.emplace_back(middle, from.y());
    points.emplace_back(middle, to.y());
  } else {
    const double between = (from.y() + to.y()) / 2;
    points.emplace_back(left, from.y());
    points.emplace_back(left, between);
    points.emplace_back(right, between);
    points.emplace_back(right, to.y());
  }
  points.push_back(to);
  setPath(RoundedPolyline(points));
}

QPainterPath EdgeItem::shape() const {
  QPainterPathStroker stroker;
  stroker.setWidth(10);
  return stroker.createStroke(path());
}

void EdgeItem::paint(QPainter* painter, const QStyleOptionGraphicsItem*,
                     QWidget*) {
  painter->setRenderHint(QPainter::Antialiasing);
  QColor color = QColor::fromString(ToQ(edge_.color));
  if (!isSelected()) color.setAlphaF(0.75);
  QPen pen(color, isSelected() ? 3 : 2);
  pen.setCosmetic(true);
  painter->setPen(pen);
  painter->setBrush(Qt::NoBrush);
  painter->drawPath(path());
}

GraphCanvas::GraphCanvas(QWidget* parent) : QGraphicsView(parent) {
  setObjectName(QStringLiteral("Canvas"));
  setScene(&scene_);
  scene_.setItemIndexMethod(QGraphicsScene::BspTreeIndex);
  setRenderHints(QPainter::Antialiasing | QPainter::TextAntialiasing |
                 QPainter::SmoothPixmapTransform);
  setViewportUpdateMode(QGraphicsView::SmartViewportUpdate);
  setTransformationAnchor(QGraphicsView::NoAnchor);
  setDragMode(QGraphicsView::RubberBandDrag);
  setRubberBandSelectionMode(Qt::IntersectsItemShape);
  setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
  setVerticalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
  setFrameShape(QFrame::NoFrame);
  setAcceptDrops(true);
  viewport()->grabGesture(Qt::PinchGesture);
  scene_.setSceneRect(-50000, -50000, 100000, 100000);
  connect(&scene_, &QGraphicsScene::selectionChanged, this,
          &GraphCanvas::SyncSelection);

  auto* out =
      MakeIconButton(QStringLiteral("minus"), QStringLiteral("Zoom out"), this);
  auto* in =
      MakeIconButton(QStringLiteral("plus"), QStringLiteral("Zoom in"), this);
  zoom_button_ = MakeButton(QStringLiteral("100%"), "ghost", QString(), this);
  zoom_button_->setFixedWidth(kZoomButtonWidth);
  zoom_button_->setToolTip(QStringLiteral("Zoom to 100%"));
  auto* fit = MakeIconButton(QStringLiteral("maximize"),
                             QStringLiteral("Fit to view"), this);
  const std::array<QWidget*, 4> zoom_controls = {out, in, zoom_button_, fit};
  for (QWidget* widget : zoom_controls) widget->setFocusPolicy(Qt::NoFocus);
  connect(out, &QToolButton::clicked, this,
          [this] { ZoomBy(1 / 1.2, viewport()->rect().center()); });
  connect(in, &QToolButton::clicked, this,
          [this] { ZoomBy(1.2, viewport()->rect().center()); });
  connect(zoom_button_, &QPushButton::clicked, this,
          [this] { ZoomBy(1.0 / zoom(), viewport()->rect().center()); });
  connect(fit, &QToolButton::clicked, this, &GraphCanvas::FitAll);

  modes_ = new QWidget(this);
  modes_->setObjectName(QStringLiteral("CanvasControls"));
  auto* controls = new QHBoxLayout(modes_);
  controls->setContentsMargins(0, 0, 0, 0);
  controls->setSpacing(8);
  const auto card = [this](QHBoxLayout** row) {
    auto* box = new QWidget(modes_);
    box->setObjectName(QStringLiteral("CanvasToolbar"));
    box->setAttribute(Qt::WA_StyledBackground);
    *row = new QHBoxLayout(box);
    (*row)->setContentsMargins(4, 4, 4, 4);
    (*row)->setSpacing(4);
    return box;
  };
  QHBoxLayout* zoom_row = nullptr;
  QWidget* zoom_card = card(&zoom_row);
  QHBoxLayout* mode_row = nullptr;
  QWidget* mode_card = card(&mode_row);
  mouse_ = MakeIconButton(
      QStringLiteral("mouse"),
      QStringLiteral(
          "Mouse: the wheel zooms, drag the background to move around"),
      modes_);
  touchpad_ = MakeIconButton(
      QStringLiteral("touchpad"),
      QStringLiteral("Touchpad: two fingers move around, pinch to zoom"),
      modes_);
  auto* map = MakeIconButton(QStringLiteral("map"),
                             QStringLiteral("Hide the minimap"), modes_);
  for (QToolButton* button : {mouse_, touchpad_, map}) {
    button->setCheckable(true);
    button->setFocusPolicy(Qt::NoFocus);
  }
  map->setChecked(true);
  for (QWidget* widget : zoom_controls) {
    widget->setParent(zoom_card);
    zoom_row->addWidget(widget);
  }
  auto* grid = MakeIconButton(QStringLiteral("grid-3x3"),
                              QStringLiteral("Hide the grid"), modes_);
  grid->setCheckable(true);
  grid->setChecked(true);
  grid->setFocusPolicy(Qt::NoFocus);
  connect(grid, &QToolButton::toggled, this, [this, grid](bool shown) {
    grid_shown_ = shown;
    grid->setToolTip(shown ? QStringLiteral("Hide the grid")
                           : QStringLiteral("Show the grid"));
    resetCachedContent();
    viewport()->update();
  });
  mouse_->setParent(mode_card);
  touchpad_->setParent(mode_card);
  mode_row->addWidget(mouse_);
  mode_row->addWidget(touchpad_);
  for (const auto& [floating, glyph] :
       {std::pair{grid, QStringLiteral("grid-3x3")},
        std::pair{map, QStringLiteral("map")}}) {
    floating->setObjectName(QStringLiteral("CanvasFloat"));
    floating->setFixedSize(kFloatSize, kFloatSize);
    connect(floating, &QToolButton::toggled, floating,
            [floating, glyph](bool on) {
              App& app = App::Get();
              floating->setIcon(app.Icon(
                  glyph, app.Token(on ? "foreground" : "muted-foreground")));
            });
  }
  controls->addWidget(zoom_card);
  controls->addWidget(grid);
  controls->addWidget(mode_card);
  controls->addWidget(map);
  const auto choose = [](const QString& mode) {
    try {
      App::Get().session().patch_settings(ToRust(
          QStringLiteral(R"({"editor":{"canvasMode":"%1"}})").arg(mode)));
    } catch (const rust::Error& error) {
      App::Get().ShowError(QString::fromUtf8(error.what()));
    }
  };
  connect(mouse_, &QToolButton::clicked, this,
          [choose] { choose(QStringLiteral("mouse")); });
  connect(touchpad_, &QToolButton::clicked, this,
          [choose] { choose(QStringLiteral("touchpad")); });
  connect(map, &QToolButton::toggled, this, [this, map](bool shown) {
    SetMinimapVisible(shown);
    map->setToolTip(shown ? QStringLiteral("Hide the minimap")
                          : QStringLiteral("Show the minimap"));
  });
  connect(this, &GraphCanvas::ZoomChanged, this, [this](double value) {
    zoom_button_->setText(QStringLiteral("%1%").arg(std::lround(value * 100)));
  });

  minimap_ = new Minimap(this, &scene_);
  minimap_->setFixedSize(kMinimapWidth, kMinimapHeight);

  toast_ = new QLabel(this);
  toast_->setObjectName(QStringLiteral("Toast"));
  toast_->hide();
  toast_timer_.setSingleShot(true);
  connect(&toast_timer_, &QTimer::timeout, toast_, [this] { FadeOut(toast_); });

  sync_timer_.setSingleShot(true);
  sync_timer_.setInterval(0);
  connect(&sync_timer_, &QTimer::timeout, this, &GraphCanvas::Sync);
  connect(EventHub::Instance(), &EventHub::DocumentChanged, &sync_timer_,
          qOverload<>(&QTimer::start));
  connect(EventHub::Instance(), &EventHub::CompiledChanged, &sync_timer_,
          qOverload<>(&QTimer::start));
  connect(EventHub::Instance(), &EventHub::RunChanged, &sync_timer_,
          qOverload<>(&QTimer::start));
  motion_timer_.setInterval(kMotionFrameMs);
  connect(&motion_timer_, &QTimer::timeout, this, [this] {
    const qint64 now = motion_clock_.elapsed();
    for (auto it = motion_started_.begin(); it != motion_started_.end();) {
      CardItem* card = cards_.value(it.key());
      const bool over =
          card == nullptr ||
          now - it.value() >
              static_cast<qint64>(MotionInfoOf(ToQ(card->card().motion)).ms);
      if (card != nullptr) card->update();
      it = over ? motion_started_.erase(it) : std::next(it);
    }
    if (motion_started_.isEmpty()) motion_timer_.stop();
  });
  spin_timer_.setInterval(kSpinFrameMs);
  connect(&spin_timer_, &QTimer::timeout, this, [this] {
    spin_angle_ = std::fmod(spin_angle_ + kSpinStep, 360.0);
    for (CardItem* card : cards_) {
      if (card->card().ring == "loading") card->update();
    }
  });
  connect(EventHub::Instance(), &EventHub::Notice, this,
          [this](const QString& title, const QString& body) {
            Toast(body.isEmpty() ? title : title + QStringLiteral(": ") + body,
                  false);
          });
  connect(EventHub::Instance(), &EventHub::SettingsChanged, this,
          &GraphCanvas::SyncMode);
  SyncMode();
}

void GraphCanvas::SyncMode() {
  canvas_mode_ = ToQ(App::Get().session().settings().canvas_mode);
  mouse_->setChecked(!TouchpadMode());
  touchpad_->setChecked(TouchpadMode());
}

bool GraphCanvas::TouchpadMode() const {
  return canvas_mode_ == QStringLiteral("touchpad");
}

void GraphCanvas::Sync() {
  sync_timer_.stop();
  QElapsedTimer timer;
  timer.start();
  syncing_ = true;
  const bridge::GraphView graph = App::Get().session().graph();
  const qint64 read_ms = timer.elapsed();
  QSet<QString> seen;
  for (const auto& card : graph.nodes) {
    const QString id = ToQ(card.id);
    seen.insert(id);
    auto found = cards_.find(id);
    if (found == cards_.end()) {
      auto* item = new CardItem(this, card);
      scene_.addItem(item);
      cards_.insert(id, item);
    } else {
      (*found)->SetCard(card);
    }
  }
  bool spinning = false;
  for (auto it = cards_.begin(); it != cards_.end();) {
    if (!seen.contains(it.key())) {
      delete it.value();
      it = cards_.erase(it);
    } else {
      spinning = spinning || it.value()->card().ring == "loading";
      ++it;
    }
  }
  if (spinning && !ReducedMotion()) {
    if (!spin_timer_.isActive()) spin_timer_.start();
  } else {
    spin_timer_.stop();
  }
  seen.clear();
  for (const auto& edge : graph.edges) {
    const QString id = ToQ(edge.id);
    seen.insert(id);
    auto found = edges_.find(id);
    EdgeItem* item = nullptr;
    if (found == edges_.end()) {
      item = new EdgeItem(edge);
      scene_.addItem(item);
      edges_.insert(id, item);
    } else {
      item = *found;
      item->SetEdge(edge);
    }
    RouteEdge(item);
  }
  for (auto it = edges_.begin(); it != edges_.end();) {
    if (!seen.contains(it.key())) {
      delete it.value();
      it = edges_.erase(it);
    } else {
      ++it;
    }
  }
  const QString selected = ToQ(App::Get().session().selected());
  for (CardItem* card : std::as_const(cards_)) {
    if (card->id() == selected && !card->isSelected()) card->setSelected(true);
  }
  syncing_ = false;
  minimap_->Refit();
  App::Get().Log(
      QStringLiteral("sync %1 cards %2 edges: read %3 ms, total %4 ms")
          .arg(graph.nodes.size())
          .arg(graph.edges.size())
          .arg(read_ms)
          .arg(timer.elapsed()));
}

void GraphCanvas::RouteEdge(EdgeItem* edge) {
  CardItem* from = cards_.value(edge->source());
  CardItem* to = cards_.value(edge->target());
  if (from == nullptr || to == nullptr) return;
  edge->Route(from->PortPoint(ToQ(edge->edge().source_handle), true),
              to->PortPoint(ToQ(edge->edge().target_handle), false));
}

void GraphCanvas::CardMoved(const QString& id) {
  for (EdgeItem* edge : std::as_const(edges_)) {
    if (edge->source() == id || edge->target() == id) RouteEdge(edge);
  }
}

void GraphCanvas::MoveChildren(const QString& group, const QPointF& delta) {
  for (CardItem* card : std::as_const(cards_)) {
    if (ToQ(card->card().parent) == group && !card->isSelected()) {
      card->setPos(card->pos() + delta);
    }
  }
}

void GraphCanvas::ResizeGroup(const QString& id, const QRectF& rect) {
  try {
    App::Get().session().resize_node(ToRust(id), rect.x(), rect.y(),
                                     rect.width(), rect.height());
  } catch (const rust::Error& error) {
    Toast(QString::fromUtf8(error.what()), true);
  }
}

void GraphCanvas::CommitMoves() {
  QSet<QString> moved;
  for (CardItem* card : std::as_const(cards_)) {
    const QPointF at = card->pos();
    if (std::abs(at.x() - card->card().x) > 0.01 ||
        std::abs(at.y() - card->card().y) > 0.01) {
      moved.insert(card->id());
    }
  }
  rust::Vec<bridge::NodeMove> moves;
  for (const QString& id : std::as_const(moved)) {
    CardItem* card = cards_.value(id);
    if (moved.contains(ToQ(card->card().parent))) continue;
    moves.push_back(
        bridge::NodeMove{ToRust(id), card->pos().x(), card->pos().y()});
  }
  if (moves.empty()) return;
  bridge::Session& session = App::Get().session();
  session.begin_gesture();
  try {
    session.move_nodes(std::move(moves));
  } catch (const rust::Error& error) {
    Toast(QString::fromUtf8(error.what()), true);
  }
  session.end_gesture();
}

void GraphCanvas::SyncSelection() {
  if (syncing_) return;
  rust::Vec<rust::String> ids;
  for (QGraphicsItem* item : scene_.selectedItems()) {
    if (auto* card = dynamic_cast<CardItem*>(item))
      ids.push_back(ToRust(card->id()));
  }
  App::Get().session().select(std::move(ids));
}

void GraphCanvas::SelectAll() {
  for (CardItem* card : std::as_const(cards_)) card->setSelected(true);
}

void GraphCanvas::BeginConnect(const PortHit& from) {
  from_ = from;
  pending_ = new QGraphicsPathItem();
  QPen pen(App::Get().Token("ring"), 2, Qt::DashLine);
  pending_->setPen(pen);
  pending_->setZValue(5);
  scene_.addItem(pending_);
  UpdateConnect(from.at);
}

void GraphCanvas::UpdateConnect(const QPointF& scene_point) {
  if (pending_ == nullptr || !from_) return;
  QPointF a = from_->at;
  QPointF b = scene_point;
  if (!from_->output) std::swap(a, b);
  const double bend = std::max(40.0, std::abs(b.x() - a.x()) / 2);
  QPainterPath path(a);
  path.cubicTo(a + QPointF(bend, 0), b - QPointF(bend, 0), b);
  pending_->setPath(path);
}

void GraphCanvas::FinishConnect(const QPointF& scene_point) {
  delete pending_;
  pending_ = nullptr;
  if (!from_) return;
  const PortHit from = *from_;
  from_.reset();
  const std::optional<PortHit> to = PortAtScene(scene_point);
  if (!to || to->output == from.output || to->node == from.node) return;
  const PortHit& source = from.output ? from : *to;
  const PortHit& target = from.output ? *to : from;
  try {
    App::Get().session().connect(ToRust(source.node), ToRust(source.port),
                                 ToRust(target.node), ToRust(target.port));
  } catch (const rust::Error& error) {
    Toast(QString::fromUtf8(error.what()), true);
  }
}

std::optional<PortHit> GraphCanvas::PortAtScene(const QPointF& point) const {
  const QList<QGraphicsItem*> hits = scene_.items(QRectF(
      point - QPointF(kPortHit, kPortHit), QSizeF(2 * kPortHit, 2 * kPortHit)));
  for (QGraphicsItem* item : hits) {
    if (auto* card = dynamic_cast<CardItem*>(item)) {
      if (auto hit = card->PortAt(card->mapFromScene(point))) return hit;
    }
  }
  return std::nullopt;
}

void GraphCanvas::RemoveSelected() {
  rust::Vec<rust::String> nodes;
  rust::Vec<rust::String> edges;
  for (QGraphicsItem* item : scene_.selectedItems()) {
    if (auto* card = dynamic_cast<CardItem*>(item)) {
      nodes.push_back(ToRust(card->id()));
    } else if (auto* edge = dynamic_cast<EdgeItem*>(item)) {
      edges.push_back(ToRust(edge->id()));
    }
  }
  try {
    App::Get().session().remove(std::move(nodes), std::move(edges));
  } catch (const rust::Error& error) {
    Toast(QString::fromUtf8(error.what()), true);
  }
}

void GraphCanvas::Toast(const QString& message, bool error) {
  toast_->setText(message);
  toast_->setProperty("tone", error ? "error" : "info");
  toast_->style()->unpolish(toast_);
  toast_->style()->polish(toast_);
  toast_->adjustSize();
  PlaceOverlays();
  toast_->raise();
  if (!toast_->isVisible()) FadeIn(toast_);
  toast_timer_.start(kToastMs);
}

double GraphCanvas::zoom() const { return transform().m11(); }

void GraphCanvas::ZoomBy(double factor, const QPointF& anchor) {
  const double next = std::clamp(zoom() * factor, kMinZoom, kMaxZoom);
  const double applied = next / zoom();
  const QPointF before = mapToScene(anchor.toPoint());
  scale(applied, applied);
  const QPointF after = mapToScene(anchor.toPoint());
  const QPointF delta = after - before;
  translate(delta.x(), delta.y());
  emit ZoomChanged(zoom());
  minimap_->viewport()->update();
}

void GraphCanvas::FitAll() {
  QRectF bounds;
  for (CardItem* card : std::as_const(cards_)) {
    bounds |= card->sceneBoundingRect();
  }
  if (bounds.isEmpty()) {
    resetTransform();
    centerOn(0, 0);
  } else {
    fitInView(bounds.adjusted(-60, -60, 60, 60), Qt::KeepAspectRatio);
    if (zoom() > 1.0) {
      const QPointF center = bounds.center();
      resetTransform();
      centerOn(center);
    }
  }
  emit ZoomChanged(zoom());
  minimap_->viewport()->update();
}

void GraphCanvas::PlayMotion(const QString& id) {
  CardItem* card = cards_.value(id);
  if (card == nullptr || card->card().motion.empty() || ReducedMotion()) return;
  if (motion_started_.contains(id)) return;
  if (!motion_clock_.isValid()) motion_clock_.start();
  motion_started_.insert(id, motion_clock_.elapsed());
  if (!motion_timer_.isActive()) motion_timer_.start();
}

double GraphCanvas::MotionElapsed(const QString& id) const {
  const auto found = motion_started_.find(id);
  if (found == motion_started_.end()) return -1;
  return static_cast<double>(motion_clock_.elapsed() - *found);
}

const bridge::MotionInfo& GraphCanvas::MotionInfoOf(const QString& motion) {
  auto found = motion_infos_.find(motion);
  if (found == motion_infos_.end()) {
    found = motion_infos_.insert(
        motion, App::Get().session().motion_info(ToRust(motion)));
  }
  return *found;
}

QString GraphCanvas::ToolbarProblems() const {
  QStringList problems;
  if (mouse_->isChecked() == TouchpadMode())
    problems << QStringLiteral("mouse button out of step");
  if (touchpad_->isChecked() != TouchpadMode())
    problems << QStringLiteral("touchpad button out of step");
  if (modes_->x() < width() / 2 || modes_->y() > kOverlayMargin * 2) {
    problems << QStringLiteral("mode pill isn't top right");
  }
  if (!modes_->isAncestorOf(zoom_button_)) {
    problems << QStringLiteral("the zoom controls aren't in the top-right row");
  }
  return problems.join(QStringLiteral("; "));
}

QStringList GraphCanvas::GroupIds() const {
  QStringList ids;
  for (const CardItem* card : cards_) {
    if (card->card().kind == bridge::CardKind::Group) ids.append(card->id());
  }
  return ids;
}

int GraphCanvas::BackwardEdges() const {
  int backward = 0;
  for (const EdgeItem* edge : edges_) {
    const CardItem* from = cards_.value(edge->source());
    const CardItem* to = cards_.value(edge->target());
    if (from == nullptr || to == nullptr) continue;
    if (to->card().kind != bridge::CardKind::Card) continue;
    if (to->pos().x() < from->pos().x() + from->card().width) ++backward;
  }
  return backward;
}

void GraphCanvas::Reveal(const QString& id) {
  CardItem* card = cards_.value(id);
  if (card == nullptr) return;
  const QRectF target = card->sceneBoundingRect();
  const QRectF visible = mapToScene(viewport()->rect()).boundingRect();
  if (visible.contains(target)) return;
  const QSizeF view = viewport()->size();
  const double fit =
      std::min(view.width() / target.width(), view.height() / target.height()) /
      (1.0 + kRevealPadding);
  const double end_zoom = std::clamp(
      std::min(fit, std::max(zoom(), kMinRevealZoom)), kMinZoom, kMaxZoom);
  const double start_zoom = zoom();
  const QPointF start = visible.center();
  const QPointF end = target.center();
  const auto apply = [this, start_zoom, end_zoom, start, end](double t) {
    const double scale = start_zoom + (end_zoom - start_zoom) * t;
    setTransform(QTransform::fromScale(scale, scale));
    centerOn(start + (end - start) * t);
    emit ZoomChanged(scale);
    minimap_->viewport()->update();
  };
  const int ms = MotionMs(kRevealMs);
  if (ms == 0) {
    apply(1.0);
    return;
  }
  auto* animation = new QVariantAnimation(this);
  animation->setDuration(ms);
  animation->setStartValue(0.0);
  animation->setEndValue(1.0);
  animation->setEasingCurve(EaseOut());
  connect(animation, &QVariantAnimation::valueChanged, this,
          [apply](const QVariant& value) { apply(value.toDouble()); });
  animation->start(QAbstractAnimation::DeleteWhenStopped);
}

void GraphCanvas::SetMinimapVisible(bool visible) {
  minimap_->setVisible(visible);
}

bool GraphCanvas::minimap_visible() const { return minimap_->isVisible(); }

void GraphCanvas::drawBackground(QPainter* painter, const QRectF& rect) {
  App& app = App::Get();
  painter->fillRect(rect, app.Token("background"));
  if (!grid_shown_) return;
  double gap = kGridGap;
  while (gap * zoom() < 10) gap *= 2;
  const QColor dot = app.Token("grid-dot");
  const double size = 1.5 / zoom();
  const double left = std::floor(rect.left() / gap) * gap;
  const double top = std::floor(rect.top() / gap) * gap;
  painter->setPen(Qt::NoPen);
  painter->setBrush(dot);
  std::vector<QRectF> dots;
  const auto columns = static_cast<int>(std::ceil((rect.right() - left) / gap));
  const auto rows = static_cast<int>(std::ceil((rect.bottom() - top) / gap));
  for (int column = 0; column <= columns; ++column) {
    for (int row = 0; row <= rows; ++row) {
      const double x = left + column * gap;
      const double y = top + row * gap;
      dots.emplace_back(x - size / 2, y - size / 2, size, size);
    }
  }
  painter->setRenderHint(QPainter::Antialiasing, false);
  painter->drawRects(dots.data(), static_cast<int>(dots.size()));
}

void GraphCanvas::wheelEvent(QWheelEvent* event) {
  const bool zooming =
      !TouchpadMode() || event->modifiers().testFlag(Qt::ControlModifier);
  if (zooming) {
    const double steps = event->angleDelta().y() / 120.0;
    ZoomBy(std::pow(1.15, steps), event->position());
  } else {
    const QPointF delta = event->pixelDelta().isNull()
                              ? QPointF(event->angleDelta()) / 3.0
                              : QPointF(event->pixelDelta());
    horizontalScrollBar()->setValue(horizontalScrollBar()->value() -
                                    static_cast<int>(delta.x()));
    verticalScrollBar()->setValue(verticalScrollBar()->value() -
                                  static_cast<int>(delta.y()));
  }
  event->accept();
}

bool GraphCanvas::viewportEvent(QEvent* event) {
  if (event->type() == QEvent::NativeGesture) {
    auto* gesture = static_cast<QNativeGestureEvent*>(event);
    if (gesture->gestureType() == Qt::ZoomNativeGesture) {
      ZoomBy(1.0 + gesture->value(), gesture->position());
      return true;
    }
  }
  return QGraphicsView::viewportEvent(event);
}

void GraphCanvas::scrollContentsBy(int dx, int dy) {
  QGraphicsView::scrollContentsBy(dx, dy);
  if (minimap_ != nullptr) minimap_->viewport()->update();
}

void GraphCanvas::mousePressEvent(QMouseEvent* event) {
  if (event->button() == Qt::MiddleButton ||
      event->button() == Qt::RightButton) {
    panning_ = true;
    pan_from_ = event->position().toPoint();
    viewport()->setCursor(Qt::ClosedHandCursor);
    event->accept();
    return;
  }
  QGraphicsView::mousePressEvent(event);
}

void GraphCanvas::mouseMoveEvent(QMouseEvent* event) {
  if (panning_) {
    const QPoint delta = event->position().toPoint() - pan_from_;
    pan_from_ = event->position().toPoint();
    horizontalScrollBar()->setValue(horizontalScrollBar()->value() - delta.x());
    verticalScrollBar()->setValue(verticalScrollBar()->value() - delta.y());
    event->accept();
    return;
  }
  QGraphicsView::mouseMoveEvent(event);
}

void GraphCanvas::mouseReleaseEvent(QMouseEvent* event) {
  if (panning_) {
    panning_ = false;
    viewport()->unsetCursor();
    event->accept();
    return;
  }
  QGraphicsView::mouseReleaseEvent(event);
}

void GraphCanvas::keyPressEvent(QKeyEvent* event) {
  if (event->key() == Qt::Key_Delete || event->key() == Qt::Key_Backspace) {
    RemoveSelected();
    event->accept();
    return;
  }
  if (event->matches(QKeySequence::SelectAll)) {
    SelectAll();
    event->accept();
    return;
  }
  if (event->matches(QKeySequence::Copy)) {
    copy();
    event->accept();
    return;
  }
  if (event->matches(QKeySequence::Paste)) {
    paste();
    event->accept();
    return;
  }
  if (event->key() == Qt::Key_D && event->modifiers() == Qt::ControlModifier) {
    Duplicate();
    event->accept();
    return;
  }
  QGraphicsView::keyPressEvent(event);
}

void GraphCanvas::copy() {
  const QString text = ToQ(App::Get().session().copy_selection());
  if (!text.isEmpty()) QGuiApplication::clipboard()->setText(text);
}

void GraphCanvas::paste() {
  try {
    App::Get().session().paste_text(
        ToRust(QGuiApplication::clipboard()->text()));
  } catch (const rust::Error& error) {
    Toast(QString::fromUtf8(error.what()), true);
  }
}

void GraphCanvas::Duplicate() {
  try {
    App::Get().session().duplicate_selection();
  } catch (const rust::Error& error) {
    Toast(QString::fromUtf8(error.what()), true);
  }
}

void GraphCanvas::resizeEvent(QResizeEvent* event) {
  QGraphicsView::resizeEvent(event);
  PlaceOverlays();
}

void GraphCanvas::PlaceOverlays() {
  modes_->adjustSize();
  modes_->move(width() - modes_->width() - 12, 12);
  minimap_->move(width() - minimap_->width() - 12,
                 height() - minimap_->height() - 12);
  toast_->move((width() - toast_->width()) / 2,
               height() - toast_->height() - 64);
}

void GraphCanvas::dragEnterEvent(QDragEnterEvent* event) {
  if (Droppable(event->mimeData())) event->acceptProposedAction();
}

void GraphCanvas::dragMoveEvent(QDragMoveEvent* event) {
  if (Droppable(event->mimeData())) event->acceptProposedAction();
}

bool GraphCanvas::Droppable(const QMimeData* data) {
  if (data->hasFormat(kNodeMime)) return true;
  const QList<QUrl> urls = data->urls();
  return std::any_of(urls.begin(), urls.end(),
                     [](const QUrl& url) { return url.isLocalFile(); });
}

void GraphCanvas::DropFiles(const QList<QUrl>& urls, const QPointF& at) {
  rust::Vec<rust::String> paths;
  for (const QUrl& url : urls) {
    if (url.isLocalFile()) paths.push_back(ToRust(url.toLocalFile()));
  }
  if (paths.empty()) return;
  try {
    const std::uint32_t added =
        App::Get().session().drop_files(std::move(paths), at.x(), at.y());
    if (added > 0) {
      Toast(added == 1
                ? QStringLiteral("Added 1 file to the project")
                : QStringLiteral("Added %1 files to the project").arg(added),
            false);
    }
  } catch (const rust::Error& error) {
    Toast(QString::fromUtf8(error.what()), true);
  }
}

void GraphCanvas::dropEvent(QDropEvent* event) {
  const QPointF at = mapToScene(event->position().toPoint());
  const QByteArray type = event->mimeData()->data(kNodeMime);
  if (type.isEmpty()) {
    DropFiles(event->mimeData()->urls(), at);
    event->acceptProposedAction();
    return;
  }
  try {
    App::Get().session().add_node(
        rust::Str(type.constData(), static_cast<std::size_t>(type.size())),
        at.x(), at.y());
  } catch (const rust::Error& error) {
    Toast(QString::fromUtf8(error.what()), true);
  }
  event->acceptProposedAction();
}

QPoint GraphCanvas::ViewPointOf(const QString& node) const {
  CardItem* card = cards_.value(node);
  if (card == nullptr) return {};
  return mapFromScene(card->mapToScene(card->card().width / 2, 22));
}

QPoint GraphCanvas::ViewPointOfEdge(const QString& edge) const {
  EdgeItem* item = edges_.value(edge);
  if (item == nullptr) return {};
  return mapFromScene(item->mapToScene(item->path().pointAtPercent(0.5)));
}

QPoint GraphCanvas::ViewPointOfPort(const QString& node, const QString& port,
                                    bool output) const {
  CardItem* card = cards_.value(node);
  if (card == nullptr) return {};
  return mapFromScene(card->PortPoint(port, output));
}

QString GraphCanvas::Measure(int frames) {
  QElapsedTimer total;
  std::vector<double> times;
  times.reserve(static_cast<std::size_t>(frames));
  const QTransform start = transform();
  total.start();
  for (int i = 0; i < frames; ++i) {
    if (i % 2 == 0) {
      ZoomBy(i % 4 == 0 ? 1.03 : 1 / 1.03, viewport()->rect().center());
    } else {
      horizontalScrollBar()->setValue(horizontalScrollBar()->value() + 7);
    }
    QElapsedTimer frame;
    frame.start();
    viewport()->repaint();
    times.push_back(static_cast<double>(frame.nsecsElapsed()) / 1e6);
  }
  setTransform(start);
  std::sort(times.begin(), times.end());
  double sum = 0;
  for (double value : times) sum += value;
  const double mean = sum / static_cast<double>(times.size());
  const double p95 = times.at(times.size() * 95 / 100);
  return QStringLiteral(
             "%1 frames over %2 cards: mean %3 ms, p95 %4 ms, max %5 ms, wall "
             "%6 ms")
      .arg(frames)
      .arg(cards_.size())
      .arg(mean, 0, 'f', 2)
      .arg(p95, 0, 'f', 2)
      .arg(times.back(), 0, 'f', 2)
      .arg(total.elapsed());
}

Minimap::Minimap(GraphCanvas* canvas, QGraphicsScene* scene)
    : QGraphicsView(scene, canvas), canvas_(canvas) {
  setObjectName(QStringLiteral("Minimap"));
  setInteractive(false);
  setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
  setVerticalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
  setRenderHints(QPainter::Antialiasing);
  setFrameShape(QFrame::NoFrame);
  setBackgroundBrush(App::Get().Token("popover"));
  setStyleSheet(QStringLiteral("border: 1px solid %1; border-radius: 8px;")
                    .arg(App::Get().Token("input").name(QColor::HexArgb)));
  setCursor(Qt::PointingHandCursor);
}

void Minimap::Refit() {
  QRectF bounds;
  for (QGraphicsItem* item : scene()->items()) {
    if (dynamic_cast<CardItem*>(item) != nullptr) {
      bounds |= item->sceneBoundingRect();
    }
  }
  if (bounds.isEmpty()) bounds = QRectF(-400, -300, 800, 600);
  fitInView(bounds.adjusted(-80, -80, 80, 80), Qt::KeepAspectRatio);
  viewport()->update();
}

void Minimap::drawForeground(QPainter* painter, const QRectF&) {
  const QRectF visible =
      canvas_->mapToScene(canvas_->viewport()->rect()).boundingRect();
  painter->setPen(QPen(App::Get().Token("ring"), 0));
  painter->setBrush(App::Get().Token("rubber-band"));
  painter->drawRect(visible);
}

void Minimap::mousePressEvent(QMouseEvent* event) {
  canvas_->centerOn(mapToScene(event->position().toPoint()));
  viewport()->update();
}

void Minimap::mouseMoveEvent(QMouseEvent* event) {
  if (event->buttons() & Qt::LeftButton) {
    canvas_->centerOn(mapToScene(event->position().toPoint()));
    viewport()->update();
  }
}

}
