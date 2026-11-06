#ifndef OPSIS_QT_CPP_VISUAL_FIELDS_H_
#define OPSIS_QT_CPP_VISUAL_FIELDS_H_

#include <QJsonObject>
#include <QPixmap>
#include <QString>
#include <QStringList>
#include <QWidget>
#include <functional>

namespace opsis {

namespace bridge {
struct FieldView;
}

using CommitValue = std::function<void(const QString& json)>;
using CommitPatch = std::function<void(const QJsonObject& patch)>;

class SpotFrame : public QWidget {
  Q_OBJECT

 public:
  SpotFrame(QString label, QString x_key, QString y_key, double x, double y,
            double aspect, const QString& picture, CommitPatch commit,
            QWidget* parent = nullptr);
  bool hasHeightForWidth() const override { return true; }
  int heightForWidth(int width) const override;
  QSize sizeHint() const override;
  QPointF Spot() const { return {x_, y_}; }

 protected:
  void paintEvent(QPaintEvent* event) override;
  void mousePressEvent(QMouseEvent* event) override;
  void mouseMoveEvent(QMouseEvent* event) override;
  void mouseReleaseEvent(QMouseEvent* event) override;
  void keyPressEvent(QKeyEvent* event) override;

 private:
  void Place(double x, double y);
  void PlaceAt(const QPointF& point);

  QString label_;
  QString x_key_;
  QString y_key_;
  double x_;
  double y_;
  double aspect_;
  QPixmap picture_;
  CommitPatch commit_;
  bool dragging_ = false;
};

QWidget* MakeOffsetsField(const bridge::FieldView& field,
                          const std::function<void(const QString&)>& commit);
QWidget* MakeSpotField(const bridge::FieldView& field,
                       const CommitPatch& commit);

QWidget* MakeThemeField(const bridge::FieldView& field,
                        const QStringList& swatches, bool open,
                        const std::function<void(bool)>& on_open,
                        const CommitValue& commit);

}

#endif
