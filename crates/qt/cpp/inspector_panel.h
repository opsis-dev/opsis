#ifndef OPSIS_QT_CPP_INSPECTOR_PANEL_H_
#define OPSIS_QT_CPP_INSPECTOR_PANEL_H_

#include <QHash>
#include <QJsonObject>
#include <QScrollArea>
#include <QSet>
#include <QStringList>
#include <QVBoxLayout>
#include <QWidget>

#include "rust/cxx.h"

namespace opsis {

namespace bridge {
struct FieldView;
struct InspectorView;
}

class InspectorPanel : public QWidget {
  Q_OBJECT

 public:
  explicit InspectorPanel(QWidget* parent = nullptr);
  void Rebuild();

 private:
  QWidget* MakeControl(const bridge::FieldView& field);
  void AddHeader(QVBoxLayout* layout, const bridge::InspectorView& view,
                 QWidget* content);
  void AddProblems(QVBoxLayout* layout, const rust::Vec<rust::String>& problems,
                   bool errors, QWidget* content);
  void AddActions(QVBoxLayout* layout, const bridge::InspectorView& view,
                  QWidget* content);
  QWidget* WithPresets(const bridge::FieldView& field, QWidget* row,
                       QWidget* content);
  void AddTabs(QVBoxLayout* layout, const QList<QWidget*>& basic,
               const QList<QWidget*>& advanced, QWidget* content);
  void Commit(const QString& key, const QString& json);
  void CommitPatch(const QJsonObject& patch);
  bool Editing() const;

  QScrollArea* scroll_;
  QWidget* content_ = nullptr;
  QString node_;
  bool advanced_open_ = false;
  QSet<QString> closed_items_;
  QHash<QString, QVBoxLayout*> bodies_;
  QStringList swatches_;
  QHash<QString, int> gradient_stops_;
};

}

#endif
