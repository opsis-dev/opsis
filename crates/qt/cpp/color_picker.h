#ifndef OPSIS_QT_CPP_COLOR_PICKER_H_
#define OPSIS_QT_CPP_COLOR_PICKER_H_

#include <QFrame>
#include <QLineEdit>
#include <QPointer>
#include <QStringList>
#include <QToolButton>
#include <QWidget>
#include <functional>
#include <optional>

class QPainter;

namespace opsis {

class Segmented;

struct Hsva {
  double h = 0.0;
  double s = 0.0;
  double v = 0.0;
  double a = 1.0;
};

std::optional<Hsva> ParseColor(const QString& text);
QString FormatColor(const Hsva& color, const QString& format);
QString FormatOf(const QString& text);
QColor ToQColor(const Hsva& color);
void PaintColorChip(QPainter* painter, const QRectF& rect, const QString& value,
                    qreal radius);

class SaturationArea : public QWidget {
  Q_OBJECT

 public:
  explicit SaturationArea(QWidget* parent = nullptr);
  void SetColor(const Hsva& color);
  QSize sizeHint() const override { return {224, 144}; }

 signals:
  void Picked(double s, double v);

 protected:
  void paintEvent(QPaintEvent* event) override;
  void mousePressEvent(QMouseEvent* event) override;
  void mouseMoveEvent(QMouseEvent* event) override;
  void keyPressEvent(QKeyEvent* event) override;

 private:
  void PickAt(const QPointF& point);

  Hsva color_;
};

class ChannelSlider : public QWidget {
  Q_OBJECT

 public:
  ChannelSlider(const QString& label, double maximum, double step, bool alpha,
                QWidget* parent = nullptr);
  void SetColor(const Hsva& color);
  double value() const { return value_; }
  QSize sizeHint() const override { return {224, 14}; }

 signals:
  void Moved(double value);

 protected:
  void paintEvent(QPaintEvent* event) override;
  void mousePressEvent(QMouseEvent* event) override;
  void mouseMoveEvent(QMouseEvent* event) override;
  void keyPressEvent(QKeyEvent* event) override;

 private:
  void MoveTo(double value);
  void PickAt(qreal x);

  double maximum_;
  double step_;
  bool alpha_;
  double value_ = 0.0;
  Hsva color_;
};

class ColorPicker : public QFrame {
  Q_OBJECT

 public:
  using OnChange = std::function<void(const QString&)>;

  static ColorPicker* Open(QWidget* anchor, const QString& value,
                           const QString& fallback, const QStringList& swatches,
                           OnChange on_change);
  static bool IsOpen();
  void SetValue(const QString& value);

 signals:
  void Closed();

 protected:
  void mousePressEvent(QMouseEvent* event) override;
  void hideEvent(QHideEvent* event) override;

 private:
  ColorPicker(QWidget* parent, const QString& value, const QString& fallback,
              const QStringList& swatches, OnChange on_change);
  void Pick(const Hsva& color, const QString& format);
  void Show(const Hsva& color, const QString& text);
  void SampleScreen(const QPoint& global);

  OnChange on_change_;
  Hsva color_;
  QString format_;
  SaturationArea* area_;
  ChannelSlider* hue_;
  ChannelSlider* alpha_;
  QLineEdit* text_;
  Segmented* formats_;
  QToolButton* dropper_ = nullptr;
  bool sampling_ = false;
  bool ended_ = false;
  static QPointer<ColorPicker> open_;
};

class ColorChip : public QToolButton {
 public:
  explicit ColorChip(QString value, int size, QWidget* parent = nullptr);
  void SetValue(const QString& value);

 protected:
  void paintEvent(QPaintEvent* event) override;

 private:
  QString value_;
};

class ColorInput : public QWidget {
  Q_OBJECT

 public:
  ColorInput(const QString& label, const QString& value,
             const QString& empty_label, const QStringList& swatches,
             QWidget* parent = nullptr);

 signals:
  void Changed(const QString& value);
  void PickerClosed();

 private:
  void Show(const QString& value);

  ColorChip* chip_;
  QLineEdit* line_;
  QToolButton* clear_ = nullptr;
  QString empty_label_;
};

}

#endif
