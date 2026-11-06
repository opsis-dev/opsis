#ifndef OPSIS_QT_SPIKE_CPP_INSPECTOR_PANEL_H_
#define OPSIS_QT_SPIKE_CPP_INSPECTOR_PANEL_H_

#include <QScrollArea>
#include <QVBoxLayout>
#include <QWidget>

namespace opsis {

namespace bridge {
struct FieldView;
}

class InspectorPanel : public QWidget {
  Q_OBJECT

 public:
  explicit InspectorPanel(QWidget* parent = nullptr);
  void Rebuild();

 private:
  QWidget* MakeControl(const bridge::FieldView& field);
  void Commit(const QString& key, const QString& json);
  bool Editing() const;

  QScrollArea* scroll_;
  QWidget* content_ = nullptr;
  QString node_;
  bool advanced_open_ = false;
};

}

#endif
