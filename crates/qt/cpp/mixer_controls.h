#ifndef OPSIS_QT_CPP_MIXER_CONTROLS_H_
#define OPSIS_QT_CPP_MIXER_CONTROLS_H_

#include <QFrame>
#include <QJsonArray>
#include <QJsonObject>
#include <QLineEdit>
#include <QPointer>
#include <QString>
#include <QWidget>
#include <functional>

namespace opsis {

struct KnobSpec {
  QString label;
  double min = 0;
  double max = 1;
  double step = 0.1;
  double reset = 0;
  bool log = false;
  QString format;
  QString unit;
};

using OnValue = std::function<void(double value)>;

QString MixText(const QString& kind, double value);
KnobSpec SpecFrom(const QJsonObject& range, const QString& label, double step,
                  const QString& format, const QString& unit, bool log = false);

class KnobDial : public QWidget {
  Q_OBJECT

 public:
  KnobDial(KnobSpec spec, double value, int size, OnValue on_value,
           QWidget* parent = nullptr);
  void SetValue(double value);
  double value() const { return value_; }

 protected:
  void paintEvent(QPaintEvent* event) override;
  void mousePressEvent(QMouseEvent* event) override;
  void mouseMoveEvent(QMouseEvent* event) override;
  void mouseReleaseEvent(QMouseEvent* event) override;
  void mouseDoubleClickEvent(QMouseEvent* event) override;
  void wheelEvent(QWheelEvent* event) override;
  void keyPressEvent(QKeyEvent* event) override;

 private:
  double Fraction(double value) const;
  void Commit(double value);
  void Nudge(double by);

  KnobSpec spec_;
  double value_;
  OnValue on_value_;
  bool dragging_ = false;
  bool started_ = false;
  double press_y_ = 0;
  double press_fraction_ = 0;
};

QWidget* MakeKnob(const KnobSpec& spec, double value, int size,
                  const OnValue& on_value, bool disabled = false,
                  QWidget* parent = nullptr);

class Fader : public QWidget {
  Q_OBJECT

 public:
  Fader(QString label, double value, double min, double max, OnValue on_value,
        QWidget* parent = nullptr);
  QSize sizeHint() const override { return {16, 96}; }
  double ThumbY() const;

 protected:
  void paintEvent(QPaintEvent* event) override;
  void mousePressEvent(QMouseEvent* event) override;
  void mouseMoveEvent(QMouseEvent* event) override;
  void mouseReleaseEvent(QMouseEvent* event) override;
  void mouseDoubleClickEvent(QMouseEvent* event) override;
  void keyPressEvent(QKeyEvent* event) override;

 private:
  double ValueAt(double y) const;
  double Position(double value) const;
  void Set(double value);

  QString label_;
  double value_;
  double min_;
  double max_;
  OnValue on_value_;
  bool dragging_ = false;
};

class Meter : public QWidget {
  Q_OBJECT

 public:
  Meter(QString target, bool reduction, QWidget* parent = nullptr);
  QSize sizeHint() const override { return {16, 96}; }

 protected:
  void paintEvent(QPaintEvent* event) override;

 private:
  QString target_;
  bool reduction_;
};

class EqCurve : public QWidget {
 public:
  EqCurve(QJsonObject eq, double highpass, QSize size,
          QWidget* parent = nullptr);
  void SetEq(const QJsonObject& eq);

 protected:
  void paintEvent(QPaintEvent* event) override;

 private:
  QJsonObject eq_;
  double highpass_;
};

class MixerPopover : public QFrame {
 public:
  static MixerPopover* Open(QWidget* anchor, const QString& name, int width);

 protected:
  void hideEvent(QHideEvent* event) override;

 private:
  MixerPopover(QWidget* parent, const QString& name, int width);
  bool ended_ = false;
};

void OpenEqPopover(QWidget* anchor, const QJsonObject& book, QJsonObject eq,
                   double highpass,
                   const std::function<void(const QJsonObject&)>& apply);
void OpenCompPopover(QWidget* anchor, const QJsonObject& book, QJsonObject comp,
                     const std::function<void(const QJsonObject&)>& apply);
void OpenDuckPopover(QWidget* anchor, const QJsonObject& book,
                     const QString& bus, const QJsonArray& buses,
                     QJsonObject duck,
                     const std::function<void(const QJsonObject&)>& apply);

}

#endif
