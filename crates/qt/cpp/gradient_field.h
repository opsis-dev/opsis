#ifndef OPSIS_QT_CPP_GRADIENT_FIELD_H_
#define OPSIS_QT_CPP_GRADIENT_FIELD_H_

#include <QJsonObject>
#include <QLabel>
#include <QLineEdit>
#include <QStringList>
#include <QToolButton>
#include <QWidget>
#include <functional>
#include <utility>
#include <vector>

namespace opsis {

class ColorInput;
class Segmented;

struct GradientStop {
  QString color;
  double at = 0.0;
};

class StopBar : public QWidget {
  Q_OBJECT

 public:
  explicit StopBar(QWidget* parent = nullptr);
  void SetStops(std::vector<GradientStop> stops, bool can_add, int selected);
  QSize sizeHint() const override { return {240, 44}; }
  QRectF Track() const;
  QPointF HandleOf(int index) const;

 signals:
  void Selected(int index);
  void Added(double at);
  void Moved(int index, double at);
  void Dropped(int index);

 protected:
  void paintEvent(QPaintEvent* event) override;
  void mousePressEvent(QMouseEvent* event) override;
  void mouseMoveEvent(QMouseEvent* event) override;
  void mouseReleaseEvent(QMouseEvent* event) override;
  void keyPressEvent(QKeyEvent* event) override;

 private:
  double AtOf(qreal x) const;
  int HandleAt(const QPointF& point) const;

  std::vector<GradientStop> stops_;
  bool can_add_ = false;
  int selected_ = 0;
  int dragging_ = -1;
};

class GradientField : public QWidget {
  Q_OBJECT

 public:
  GradientField(const QString& value, const QStringList& swatches, int selected,
                std::function<void(int)> on_select, QWidget* parent = nullptr);
  StopBar* bar() const { return bar_; }
  int StopCount() const { return static_cast<int>(stops_.size()); }

 signals:
  void Changed(const QString& css);

 private:
  void Load(const QString& css);
  void Edit(const QJsonObject& edit);
  void Show();
  void Select(int index);

  QString css_;
  bool valid_ = false;
  bool linear_ = true;
  double degrees_ = 0.0;
  bool can_add_ = false;
  bool can_remove_ = false;
  std::vector<GradientStop> stops_;
  int selected_;
  std::function<void(int)> on_select_;
  QWidget* editor_;
  StopBar* bar_;
  Segmented* kind_;
  QWidget* angle_box_;
  QLineEdit* angle_;
  ColorInput* stop_color_;
  QLineEdit* at_;
  QToolButton* remove_;
  QLabel* note_;
  QLineEdit* raw_;
};

}

#endif
