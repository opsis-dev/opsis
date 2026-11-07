#ifndef OPSIS_QT_CPP_SCENARIO_H_
#define OPSIS_QT_CPP_SCENARIO_H_

#include <QAbstractButton>
#include <QDir>
#include <QObject>
#include <QString>
#include <QStringList>
#include <functional>
#include <vector>

namespace opsis {

class MainWindow;

class Scenario : public QObject {
  Q_OBJECT

 public:
  static void MaybeStart(MainWindow* window);

 private:
  struct Step {
    QString name;
    std::function<void()> action;
    int wait_ms;
    std::function<QString()> check;
  };

  Scenario(MainWindow* window, const QString& name, const QDir& shots);
  void Add(const QString& name, std::function<void()> action, int wait_ms = 450,
           std::function<QString()> check = {});
  void BuildTour();
  void BuildBench(int count);
  void Next();
  void Shoot(const QString& name);
  void BuildMixer(MainWindow* window);
  void BuildPlayer();
  static QString Overflow();
  void Drag(QWidget* target, QPoint from, QPoint to, int steps = 12);
  void Miss(const QString& what) { misses_ << what; }
  QAbstractButton* Find(QWidget* root, const QString& name);
  void Click(QWidget* root, const QString& name);
  void ClickLater(QWidget* root, const QString& name);
  void ClickMenu(const QString& section, const QString& item);
  void ClickAt(QWidget* target, QPoint at);

  MainWindow* window_;
  QString name_;
  QDir shots_;
  std::vector<Step> steps_;
  std::size_t index_ = 0;
  bool shooting_ = true;
  QStringList misses_;
  int checked_ = 0;
  int failed_ = 0;
};

}

#endif
