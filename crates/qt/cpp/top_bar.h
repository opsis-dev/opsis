#ifndef OPSIS_QT_CPP_TOP_BAR_H_
#define OPSIS_QT_CPP_TOP_BAR_H_

#include <QLabel>
#include <QPushButton>
#include <QTimer>
#include <QToolButton>
#include <QWidget>

namespace opsis {

class Spinner : public QWidget {
 public:
  explicit Spinner(QWidget* parent = nullptr);

 protected:
  void paintEvent(QPaintEvent* event) override;
  void showEvent(QShowEvent* event) override;
  void hideEvent(QHideEvent* event) override;

 private:
  QTimer timer_;
  double angle_ = 0;
};

class TopBar : public QWidget {
  Q_OBJECT

 public:
  explicit TopBar(QWidget* parent = nullptr);
  void Sync();
  void SetPanels(bool left, bool bottom, bool right);
  QString Problems() const;

 signals:
  void Command(const QString& id);

 private:
  QToolButton* Tool(const QString& icon, const QString& label,
                    const QString& command);
  QToolButton* PanelToggle(const QString& icon, const QString& label,
                           const QString& command);

  QLabel* name_;
  QLabel* saved_;
  QToolButton* undo_;
  QToolButton* redo_;
  QToolButton* left_;
  QToolButton* bottom_;
  QToolButton* right_;
  QPushButton* run_;
  QWidget* running_;
};

QString ShortcutFor(const QString& command);

}

#endif
