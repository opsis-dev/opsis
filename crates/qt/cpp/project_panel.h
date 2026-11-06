#ifndef OPSIS_QT_CPP_PROJECT_PANEL_H_
#define OPSIS_QT_CPP_PROJECT_PANEL_H_

#include <QJsonObject>
#include <QWidget>

class QVBoxLayout;

namespace opsis {

namespace bridge {
struct ProjectView;
}

class ProjectPanel : public QWidget {
  Q_OBJECT

 public:
  explicit ProjectPanel(QWidget* parent = nullptr);

 private:
  void AddSelection(QVBoxLayout* layout, const bridge::ProjectView& view);
  void AddCanvas(QVBoxLayout* layout, const bridge::ProjectView& view);
  void AddFonts(QVBoxLayout* layout, const bridge::ProjectView& view);
  void AddVariables(QVBoxLayout* layout, const bridge::ProjectView& view);
  void AddSavedResults(QVBoxLayout* layout, const bridge::ProjectView& view);
  void AddProblems(QVBoxLayout* layout, const bridge::ProjectView& view);
  static void Edit(const QJsonObject& edit);
  static void EditJson(const QString& json);
  static void SetSettings(const QJsonObject& patch);
};

}

#endif
