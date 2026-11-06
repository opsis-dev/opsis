#ifndef OPSIS_QT_SPIKE_CPP_START_SCREEN_H_
#define OPSIS_QT_SPIKE_CPP_START_SCREEN_H_

#include <QListWidget>
#include <QWidget>

namespace opsis {

class StartScreen : public QWidget {
  Q_OBJECT

 public:
  explicit StartScreen(QWidget* parent = nullptr);
  void Reload();

 signals:
  void OpenRequested(const QString& dir, bool example);
  void CreateRequested(const QString& name);
  void BrowseRequested();

 private:
  QListWidget* recent_;
  QListWidget* examples_;
};

}

#endif
