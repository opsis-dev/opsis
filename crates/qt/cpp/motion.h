#ifndef OPSIS_QT_CPP_MOTION_H_
#define OPSIS_QT_CPP_MOTION_H_

#include <QEasingCurve>

class QWidget;

namespace opsis {

inline constexpr int kMotionMs = 220;
inline constexpr int kFadeOutMs = 80;
inline constexpr int kRisePx = 6;

bool ReducedMotion();
QEasingCurve EaseOut();
int MotionMs(int ms);
void FadeIn(QWidget* widget, int ms = kMotionMs, int rise = 0);
void FadeOut(QWidget* widget, int ms = kFadeOutMs);
void GrowWidth(QWidget* widget, int width, int ms = kMotionMs);

}

#endif
