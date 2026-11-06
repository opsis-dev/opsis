#ifndef OPSIS_QT_CPP_LIST_FIELDS_H_
#define OPSIS_QT_CPP_LIST_FIELDS_H_

#include <QHash>
#include <QJsonArray>
#include <QLabel>
#include <QSet>
#include <QStringList>
#include <QVBoxLayout>
#include <QWidget>
#include <functional>

namespace opsis {

namespace bridge {
struct FieldView;
}

using CommitJson = std::function<void(const QString& json)>;

class RowGrip : public QLabel {
 public:
  RowGrip(QWidget* row, std::function<void(int from, int to)> on_move);

 protected:
  void mousePressEvent(QMouseEvent* event) override;
  void mouseReleaseEvent(QMouseEvent* event) override;

 private:
  QWidget* row_;
  std::function<void(int from, int to)> on_move_;
};

QWidget* MakeStringList(const bridge::FieldView& field,
                        const CommitJson& commit);
QWidget* MakeNumberList(const bridge::FieldView& field,
                        const CommitJson& commit);
QWidget* MakeColorList(const bridge::FieldView& field,
                       const QStringList& swatches, const CommitJson& commit);
QWidget* MakeObjectList(const bridge::FieldView& field,
                        const CommitJson& commit, QSet<QString>* closed,
                        QHash<QString, QVBoxLayout*>* bodies);
QString JsonText(const QJsonArray& array);

}

#endif
