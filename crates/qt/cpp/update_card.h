#ifndef OPSIS_QT_CPP_UPDATE_CARD_H_
#define OPSIS_QT_CPP_UPDATE_CARD_H_

#include <QWidget>

class QLabel;
class QProgressBar;
class QPushButton;

namespace opsis {

class ToggleSwitch;

class UpdateCard : public QWidget {
  Q_OBJECT

 public:
  explicit UpdateCard(QWidget* parent = nullptr);

 private:
  void Load();

  QLabel* status_;
  QLabel* notes_;
  QProgressBar* progress_;
  QPushButton* check_;
  QPushButton* download_;
  QPushButton* cancel_;
  QPushButton* restart_;
  QPushButton* page_;
  ToggleSwitch* on_start_;
  QString page_url_;
};

}

#endif
