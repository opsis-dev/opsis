#ifndef OPSIS_QT_CPP_WIDGETS_H_
#define OPSIS_QT_CPP_WIDGETS_H_

#include <QAbstractButton>
#include <QButtonGroup>
#include <QComboBox>
#include <QFrame>
#include <QLabel>
#include <QLayout>
#include <QPushButton>
#include <QToolButton>
#include <QVBoxLayout>
#include <QVariantAnimation>
#include <utility>
#include <vector>

namespace opsis {

class FlowLayout : public QLayout {
 public:
  explicit FlowLayout(QWidget* parent, int spacing);
  FlowLayout(const FlowLayout&) = delete;
  FlowLayout& operator=(const FlowLayout&) = delete;
  FlowLayout(FlowLayout&&) = delete;
  FlowLayout& operator=(FlowLayout&&) = delete;
  ~FlowLayout() override;

  void addItem(QLayoutItem* item) override;
  int count() const override;
  QLayoutItem* itemAt(int index) const override;
  QLayoutItem* takeAt(int index) override;
  Qt::Orientations expandingDirections() const override { return {}; }
  void SetGap(int gap) { gap_ = gap; }
  bool hasHeightForWidth() const override { return true; }
  int heightForWidth(int width) const override;
  QSize minimumSize() const override;
  QSize sizeHint() const override;
  void setGeometry(const QRect& rect) override;

 private:
  int Arrange(const QRect& rect, bool apply) const;

  QList<QLayoutItem*> items_;
  int gap_;
};

class ClampedLabel : public QLabel {
 public:
  ClampedLabel(const QString& text, int lines, QWidget* parent = nullptr);
  bool hasHeightForWidth() const override { return true; }
  int heightForWidth(int width) const override;
  QSize sizeHint() const override;
  QSize minimumSizeHint() const override;

 protected:
  void resizeEvent(QResizeEvent* event) override;

 private:
  QString Clamped(int width) const;

  QString full_;
  int lines_;
};

class Combo : public QComboBox {
 public:
  using QComboBox::QComboBox;

 protected:
  void paintEvent(QPaintEvent* event) override;
};

class ToggleSwitch : public QAbstractButton {
  Q_OBJECT

 public:
  explicit ToggleSwitch(QWidget* parent = nullptr);
  QSize sizeHint() const override;
  void SetCheckedQuietly(bool checked);

 protected:
  void paintEvent(QPaintEvent* event) override;
  void checkStateSet() override;
  void nextCheckState() override;

 private:
  void Animate();

  QVariantAnimation animation_;
  qreal knob_ = 0.0;
};

class Segmented : public QFrame {
  Q_OBJECT

 public:
  explicit Segmented(QWidget* parent = nullptr);
  void SetChoices(const std::vector<std::pair<QString, QString>>& choices);
  void SetOutline();
  void SetValue(const QString& value);
  QString value() const;

 signals:
  void Changed(const QString& value);

 private:
  QButtonGroup group_;
  std::vector<QString> values_;
};

class Card : public QFrame {
 public:
  Card(const QString& title, const QString& description,
       QWidget* parent = nullptr);
  QVBoxLayout* body() const { return body_; }

 private:
  QVBoxLayout* body_;
};

QLabel* MakeLabel(const QString& text, const char* role = nullptr,
                  QWidget* parent = nullptr);
QFrame* MakeRule(QWidget* parent = nullptr);
QToolButton* MakeIconButton(const QString& icon, const QString& tip,
                            QWidget* parent = nullptr, int size = 16);
QPushButton* MakeButton(const QString& text, const char* variant = nullptr,
                        const QString& icon = QString(),
                        QWidget* parent = nullptr);
QWidget* MakeField(const QString& label, QWidget* control,
                   const QString& description = QString(),
                   QWidget* parent = nullptr);
QWidget* MakeSwitchField(const QString& label, ToggleSwitch* toggle,
                         const QString& description = QString(),
                         QWidget* parent = nullptr);
QLabel* MakeBadge(const QString& glyph, const QColor& color, int size,
                  QWidget* parent = nullptr);

}

#endif
