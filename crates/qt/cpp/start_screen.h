#ifndef OPSIS_QT_CPP_START_SCREEN_H_
#define OPSIS_QT_CPP_START_SCREEN_H_

#include <QPushButton>
#include <QVBoxLayout>
#include <QWidget>

namespace opsis {

class RowButton : public QPushButton {
 public:
  explicit RowButton(QWidget* parent = nullptr, const char* variant = "row");
  QHBoxLayout* row() const { return row_; }
  QSize sizeHint() const override;
  QSize minimumSizeHint() const override;
  bool hasHeightForWidth() const override;
  int heightForWidth(int width) const override;

 private:
  QHBoxLayout* row_;
};

class StartScreen : public QWidget {
  Q_OBJECT

 public:
  explicit StartScreen(QWidget* parent = nullptr);
  void Reload();

 signals:
  void OpenRequested(const QString& dir);
  void NewProjectRequested(const QString& template_id, bool fixed);
  void BrowseRequested();
  void SettingsRequested();
  void EngineRequested();

 private:
  QWidget* ActionList();
  QWidget* TemplateGrid();
  QWidget* EngineAlert();

  QVBoxLayout* recent_ = nullptr;
  QWidget* engine_ = nullptr;
};

}

#endif
