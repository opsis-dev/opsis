#ifndef OPSIS_QT_CPP_VISUAL_FIELDS_H_
#define OPSIS_QT_CPP_VISUAL_FIELDS_H_

#include <QJsonObject>
#include <QPixmap>
#include <QString>
#include <QStringList>
#include <QWidget>
#include <functional>
#include <vector>

namespace opsis {

namespace bridge {
struct FieldView;
}

using CommitValue = std::function<void(const QString& json)>;
using CommitPatch = std::function<void(const QJsonObject& patch)>;

class PictureFrame : public QWidget {
  Q_OBJECT

 public:
  PictureFrame(double aspect, const QString& picture, QWidget* parent);
  bool hasHeightForWidth() const override { return true; }
  int heightForWidth(int width) const override;
  QSize sizeHint() const override;

 protected:
  QRectF PaintPicture(QPainter* painter);

 private:
  double aspect_;
  QPixmap picture_;
};

class SpotFrame : public PictureFrame {
  Q_OBJECT

 public:
  SpotFrame(QString label, QString x_key, QString y_key, double x, double y,
            double aspect, const QString& picture, CommitPatch commit,
            QWidget* parent = nullptr);
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
  CommitPatch commit_;
  bool dragging_ = false;
};

struct Area {
  double x = 0;
  double y = 0;
  double width = 0;
  double height = 0;
};

class BoxesFrame : public PictureFrame {
  Q_OBJECT

 public:
  BoxesFrame(std::vector<Area> areas, int most, double aspect,
             const QString& picture, QWidget* parent = nullptr);
  const std::vector<Area>& areas() const { return areas_; }
  QRectF ToWidget(const Area& area) const;

 signals:
  void Changed(const std::vector<Area>& areas);

 protected:
  void paintEvent(QPaintEvent* event) override;
  void mousePressEvent(QMouseEvent* event) override;
  void mouseMoveEvent(QMouseEvent* event) override;
  void mouseReleaseEvent(QMouseEvent* event) override;

 private:
  QPointF FractionAt(const QPointF& point) const;
  QRectF RemoveButton(const Area& area) const;

  std::vector<Area> areas_;
  int most_;
  int moving_ = -1;
  bool drawing_ = false;
  QPointF start_;
  Area origin_;
  Area draft_;
};

QWidget* MakeOffsetsField(const bridge::FieldView& field,
                          const std::function<void(const QString&)>& commit);
QWidget* MakeSpotField(const bridge::FieldView& field,
                       const CommitPatch& commit);
QWidget* MakeBoxesField(const bridge::FieldView& field,
                        const CommitValue& commit, const CommitPatch& patch);

QWidget* MakeThemeField(const bridge::FieldView& field,
                        const QStringList& swatches, bool open,
                        const std::function<void(bool)>& on_open,
                        const CommitValue& commit);

}

#endif
