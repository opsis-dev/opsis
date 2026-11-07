#ifndef OPSIS_QT_CPP_ENGINE_CARD_H_
#define OPSIS_QT_CPP_ENGINE_CARD_H_

#include <QString>
#include <QWidget>
#include <functional>

class QLabel;
class QProgressBar;
class QPushButton;

namespace opsis {

class EngineCard : public QWidget {
  Q_OBJECT

 public:
  EngineCard(const QString& title, std::function<void()> on_done,
             QWidget* parent = nullptr);

 private:
  void Load();

  std::function<void()> on_done_;
  QLabel* state_;
  QLabel* browser_line_;
  QLabel* browser_warning_;
  QPushButton* choose_;
  QPushButton* download_;
  QPushButton* done_ = nullptr;
  QProgressBar* progress_;
  QLabel* progress_text_;
};

class SetupScreen : public QWidget {
  Q_OBJECT

 public:
  explicit SetupScreen(QWidget* parent = nullptr);

 signals:
  void Done();
};

}

#endif
