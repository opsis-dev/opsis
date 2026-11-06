#include "look_style.h"

#include <QAbstractButton>
#include <QAbstractItemView>
#include <QAbstractSpinBox>
#include <QComboBox>
#include <QFocusEvent>
#include <QFocusFrame>
#include <QLineEdit>
#include <QPainter>
#include <QPainterPath>
#include <QStyleOption>
#include <QWidget>

#include "app.h"

namespace opsis {

namespace {

constexpr int kRingWidth = 3;
constexpr double kRingRadius = 6.0 + kRingWidth;
constexpr int kRingAlpha = 128;

bool TakesRing(const QWidget* widget) {
  return qobject_cast<const QAbstractButton*>(widget) != nullptr ||
         qobject_cast<const QLineEdit*>(widget) != nullptr ||
         qobject_cast<const QComboBox*>(widget) != nullptr ||
         qobject_cast<const QAbstractSpinBox*>(widget) != nullptr ||
         qobject_cast<const QAbstractItemView*>(widget) != nullptr;
}

bool FromKeyboard(Qt::FocusReason reason) {
  return reason == Qt::TabFocusReason || reason == Qt::BacktabFocusReason;
}

}

LookStyle::LookStyle(QStyle* base) : QProxyStyle(base) {}

void LookStyle::drawControl(ControlElement element, const QStyleOption* option,
                            QPainter* painter, const QWidget* widget) const {
  if (element != CE_FocusFrame) {
    QProxyStyle::drawControl(element, option, painter, widget);
    return;
  }
  QColor ring = App::Get().Token("ring");
  ring.setAlpha(kRingAlpha);
  const QRectF outer = QRectF(option->rect);
  const QRectF inner =
      outer.adjusted(kRingWidth, kRingWidth, -kRingWidth, -kRingWidth);
  const double radius = std::min(kRingRadius, outer.height() / 2.0);
  QPainterPath path;
  path.addRoundedRect(outer, radius, radius);
  QPainterPath hole;
  const double inner_radius = std::max(0.0, radius - kRingWidth);
  hole.addRoundedRect(inner, inner_radius, inner_radius);
  painter->save();
  painter->setRenderHint(QPainter::Antialiasing);
  painter->fillPath(path.subtracted(hole), ring);
  painter->restore();
}

void LookStyle::drawPrimitive(PrimitiveElement element,
                              const QStyleOption* option, QPainter* painter,
                              const QWidget* widget) const {
  if (element == PE_FrameFocusRect) return;
  QProxyStyle::drawPrimitive(element, option, painter, widget);
}

int LookStyle::pixelMetric(PixelMetric metric, const QStyleOption* option,
                           const QWidget* widget) const {
  if (metric == PM_FocusFrameHMargin || metric == PM_FocusFrameVMargin) {
    return kRingWidth;
  }
  return QProxyStyle::pixelMetric(metric, option, widget);
}

FocusRing::FocusRing(QObject* parent) : QObject(parent) {}

bool FocusRing::eventFilter(QObject* watched, QEvent* event) {
  if (event->type() != QEvent::FocusIn || !watched->isWidgetType()) {
    return false;
  }
  auto* widget = static_cast<QWidget*>(watched);
  const auto* focus = static_cast<QFocusEvent*>(event);
  const bool show = FromKeyboard(focus->reason()) && TakesRing(widget);
  if (!show) {
    if (!frame_.isNull()) frame_->setWidget(nullptr);
    return false;
  }
  if (frame_.isNull() || frame_->window() != widget->window()) {
    if (!frame_.isNull()) frame_->deleteLater();
    frame_ = new QFocusFrame(widget->window());
    frame_->setAttribute(Qt::WA_TransparentForMouseEvents);
  }
  frame_->setWidget(widget);
  return false;
}

}
