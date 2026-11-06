#ifndef OPSIS_QT_CPP_LOOK_STYLE_H_
#define OPSIS_QT_CPP_LOOK_STYLE_H_

#include <QPointer>
#include <QProxyStyle>

class QFocusFrame;

namespace opsis {

class LookStyle : public QProxyStyle {
 public:
  explicit LookStyle(QStyle* base);

  void drawControl(ControlElement element, const QStyleOption* option,
                   QPainter* painter,
                   const QWidget* widget = nullptr) const override;
  void drawPrimitive(PrimitiveElement element, const QStyleOption* option,
                     QPainter* painter,
                     const QWidget* widget = nullptr) const override;
  int pixelMetric(PixelMetric metric, const QStyleOption* option = nullptr,
                  const QWidget* widget = nullptr) const override;
};

class FocusRing : public QObject {
 public:
  explicit FocusRing(QObject* parent);

 protected:
  bool eventFilter(QObject* watched, QEvent* event) override;

 private:
  QPointer<QFocusFrame> frame_;
};

}

#endif
