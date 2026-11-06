#ifndef OPSIS_QT_CPP_NEW_PROJECT_DIALOG_H_
#define OPSIS_QT_CPP_NEW_PROJECT_DIALOG_H_

#include <QButtonGroup>
#include <QDialog>
#include <QLabel>
#include <QLineEdit>
#include <QPushButton>
#include <QWidget>

namespace opsis {

class NewProjectDialog : public QDialog {
  Q_OBJECT

 public:
  NewProjectDialog(const QString& template_id, bool fixed, QWidget* parent);

 private:
  void SyncFolder();
  void Create();
  QString ChosenTemplate() const;

  QLineEdit* name_;
  QLineEdit* location_;
  QLabel* folder_;
  QLabel* problem_;
  QPushButton* create_;
  QButtonGroup templates_;
  QWidget* choices_;
  QWidget* chosen_;
  QStringList ids_;
};

}

#endif
