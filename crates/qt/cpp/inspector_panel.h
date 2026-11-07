#ifndef OPSIS_QT_CPP_INSPECTOR_PANEL_H_
#define OPSIS_QT_CPP_INSPECTOR_PANEL_H_

#include <QHash>
#include <QJsonObject>
#include <QPointer>
#include <QScrollArea>
#include <QSet>
#include <QStringList>
#include <QVBoxLayout>
#include <QWidget>
#include <memory>

#include "marks_field.h"
#include "rust/cxx.h"

namespace opsis {

class FilePlayer;

namespace bridge {
struct FieldView;
struct InspectorView;
}

class InspectorPanel : public QWidget {
  Q_OBJECT

 public:
  explicit InspectorPanel(QWidget* parent = nullptr);
  ~InspectorPanel() override;
  InspectorPanel(const InspectorPanel&) = delete;
  InspectorPanel& operator=(const InspectorPanel&) = delete;
  InspectorPanel(InspectorPanel&&) = delete;
  InspectorPanel& operator=(InspectorPanel&&) = delete;
  void Rebuild();
  void Refresh();
  int builds() const { return builds_; }
  QWidget* content() const { return content_; }

 private:
  void Build(const bridge::InspectorView& view);
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
  void EditKeyframes(const QJsonObject& edit);
  bool Editing() const;

  QScrollArea* scroll_;
  QWidget* content_ = nullptr;
  QString node_;
  bool advanced_open_ = false;
  QSet<QString> closed_items_;
  QHash<QString, QVBoxLayout*> bodies_;
  QStringList swatches_;
  QHash<QString, int> gradient_stops_;
  QHash<QString, MarksState> marks_state_;
  QHash<QString, QPointer<FilePlayer>> players_;
  std::unique_ptr<bridge::InspectorView> last_;
  int builds_ = 0;
};

}

#endif
