#ifndef OPSIS_QT_CPP_SCENARIO_H_
#define OPSIS_QT_CPP_SCENARIO_H_

#include <QDir>
#include <QObject>
#include <QString>
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
  };

  Scenario(MainWindow* window, const QString& name, const QDir& shots);
  void Add(const QString& name, std::function<void()> action,
           int wait_ms = 450);
  void BuildTour();
  void BuildBench(int count);
  void Next();
  void Shoot(const QString& name);
  void Drag(QWidget* target, QPoint from, QPoint to, int steps = 12);

  MainWindow* window_;
  QString name_;
  QDir shots_;
  std::vector<Step> steps_;
  std::size_t index_ = 0;
};

}

#endif
