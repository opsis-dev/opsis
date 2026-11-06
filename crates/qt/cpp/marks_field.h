#ifndef OPSIS_QT_CPP_MARKS_FIELD_H_
#define OPSIS_QT_CPP_MARKS_FIELD_H_

#include <QHash>
#include <QJsonArray>
#include <QJsonObject>
#include <QString>
#include <QVBoxLayout>
#include <QWidget>
#include <functional>
#include <optional>
#include <vector>

#include "visual_fields.h"

namespace opsis {

struct MarksState {
  int selected = -1;
  QString tool = QStringLiteral("arrow");
};

struct MarkLine {
  double x = 0;
  double y = 0;
  double x2 = 0;
  double y2 = 0;
};

class MarksFrame : public PictureFrame {
  Q_OBJECT

 public:
  MarksFrame(QJsonArray marks, std::vector<std::optional<int>> steps,
             int first_step, QJsonObject fresh, int most, double aspect,
             const QString& picture, QWidget* parent = nullptr);
  void SetTool(const QString& tool);
  void Select(int index);
  int selected() const { return selected_; }
  const QJsonArray& marks() const { return marks_; }
  QPointF ToWidget(double x, double y) const;

 signals:
  void Changed(const QJsonArray& marks, int selected);
  void Selected(int index);

 protected:
  void paintEvent(QPaintEvent* event) override;
  void mousePressEvent(QMouseEvent* event) override;
  void mouseMoveEvent(QMouseEvent* event) override;
  void mouseReleaseEvent(QMouseEvent* event) override;
  void keyPressEvent(QKeyEvent* event) override;

 private:
  void PaintMark(QPainter* painter, const QJsonObject& mark,
                 const MarkLine& line, std::optional<int> step) const;
  QPointF FractionAt(const QPointF& point) const;
  int MarkAt(const QPointF& point) const;
  int HandleAt(const QPointF& point) const;
  MarkLine Shown(int index) const;
  double Scale() const;

  QJsonArray marks_;
  std::vector<std::optional<int>> steps_;
  int first_step_;
  QJsonObject fresh_;
  int most_;
  QString tool_ = QStringLiteral("arrow");
  int selected_ = -1;
  enum class Gesture { kNone, kMove, kEnd, kDraw };
  Gesture gesture_ = Gesture::kNone;
  bool second_end_ = false;
  QPointF start_;
  MarkLine origin_;
  MarkLine live_;
};

QWidget* MakeMarksField(const bridge::FieldView& field,
                        QHash<QString, QVBoxLayout*>* bodies,
                        const MarksState& state,
                        const std::function<void(const MarksState&)>& on_state,
                        const CommitValue& commit);

}

#endif
