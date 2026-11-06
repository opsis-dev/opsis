#include "motion.h"

#include <QAccessibilityHints>
#include <QGraphicsOpacityEffect>
#include <QGuiApplication>
#include <QPointer>
#include <QPropertyAnimation>
#include <QStyleHints>
#include <QVariantAnimation>
#include <QWidget>

namespace opsis {

namespace {

constexpr const char* kFadingProperty = "opsisFading";

QGraphicsOpacityEffect* OpacityOf(QWidget* widget) {
  if (auto* effect =
          qobject_cast<QGraphicsOpacityEffect*>(widget->graphicsEffect())) {
    return effect;
  }
  auto* effect = new QGraphicsOpacityEffect(widget);
  widget->setGraphicsEffect(effect);
  return effect;
}

void StopFading(QWidget* widget) {
  if (auto* running = widget->property(kFadingProperty).value<QObject*>()) {
    running->deleteLater();
    widget->setProperty(kFadingProperty, QVariant());
  }
}

QPropertyAnimation* Fade(QWidget* widget, double from, double to, int ms) {
  StopFading(widget);
  QGraphicsOpacityEffect* effect = OpacityOf(widget);
  effect->setOpacity(from);
  auto* animation = new QPropertyAnimation(effect, "opacity", widget);
  animation->setDuration(ms);
  animation->setStartValue(from);
  animation->setEndValue(to);
  animation->setEasingCurve(EaseOut());
  widget->setProperty(kFadingProperty,
                      QVariant::fromValue<QObject*>(animation));
  return animation;
}

}

bool ReducedMotion() {
  return QGuiApplication::styleHints()->accessibility()->motionPreference() ==
         Qt::MotionPreference::ReducedMotion;
}

QEasingCurve EaseOut() {
  QEasingCurve curve(QEasingCurve::BezierSpline);
  curve.addCubicBezierSegment(QPointF(0.22, 1.0), QPointF(0.36, 1.0),
                              QPointF(1.0, 1.0));
  return curve;
}

int MotionMs(int ms) { return ReducedMotion() ? 0 : ms; }

void FadeIn(QWidget* widget, int ms, int rise) {
  if (ReducedMotion()) {
    StopFading(widget);
    widget->setGraphicsEffect(nullptr);
    widget->show();
    return;
  }
  if (widget->isWindow()) {
    const QPoint end = widget->pos();
    widget->setWindowOpacity(0.0);
    widget->move(end + QPoint(0, rise));
    widget->show();
    auto* animation = new QVariantAnimation(widget);
    animation->setDuration(ms);
    animation->setStartValue(0.0);
    animation->setEndValue(1.0);
    animation->setEasingCurve(EaseOut());
    QPointer<QWidget> target(widget);
    QObject::connect(
        animation, &QVariantAnimation::valueChanged, widget,
        [target, end, rise](const QVariant& value) {
          if (target.isNull()) return;
          const double progress = value.toDouble();
          target->setWindowOpacity(progress);
          target->move(end + QPoint(0, qRound(rise * (1.0 - progress))));
        });
    animation->start(QAbstractAnimation::DeleteWhenStopped);
    return;
  }
  QPropertyAnimation* animation = Fade(widget, 0.0, 1.0, ms);
  QPointer<QWidget> target(widget);
  QObject::connect(animation, &QPropertyAnimation::finished, widget, [target] {
    if (target.isNull()) return;
    target->setProperty(kFadingProperty, QVariant());
    target->setGraphicsEffect(nullptr);
  });
  widget->show();
  animation->start(QAbstractAnimation::DeleteWhenStopped);
}

void FadeOut(QWidget* widget, int ms) {
  if (ReducedMotion() || !widget->isVisible() || widget->isWindow()) {
    StopFading(widget);
    widget->setGraphicsEffect(nullptr);
    widget->hide();
    return;
  }
  QPropertyAnimation* animation = Fade(widget, 1.0, 0.0, ms);
  QPointer<QWidget> target(widget);
  QObject::connect(animation, &QPropertyAnimation::finished, widget, [target] {
    if (target.isNull()) return;
    target->setProperty(kFadingProperty, QVariant());
    target->hide();
    target->setGraphicsEffect(nullptr);
  });
  animation->start(QAbstractAnimation::DeleteWhenStopped);
}

void GrowWidth(QWidget* widget, int width, int ms) {
  if (ReducedMotion()) {
    widget->setFixedWidth(width);
    widget->show();
    return;
  }
  widget->setFixedWidth(0);
  widget->show();
  auto* animation = new QVariantAnimation(widget);
  animation->setDuration(ms);
  animation->setStartValue(0);
  animation->setEndValue(width);
  animation->setEasingCurve(EaseOut());
  QPointer<QWidget> target(widget);
  QObject::connect(animation, &QVariantAnimation::valueChanged, widget,
                   [target](const QVariant& value) {
                     if (!target.isNull()) target->setFixedWidth(value.toInt());
                   });
  animation->start(QAbstractAnimation::DeleteWhenStopped);
}

}
