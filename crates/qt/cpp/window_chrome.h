#ifndef OPSIS_QT_CPP_WINDOW_CHROME_H_
#define OPSIS_QT_CPP_WINDOW_CHROME_H_

#include <QHBoxLayout>
#include <QLabel>
#include <QMenuBar>
#include <QObject>
#include <QPoint>
#include <QPushButton>
#include <QToolButton>
#include <QWidget>

namespace opsis {

class AppBar : public QWidget {
  Q_OBJECT

 public:
  AppBar(QWidget* window, bool resizable, bool menu);
  QMenuBar* menu_bar() const { return menu_bar_; }
  QHBoxLayout* content() const { return content_; }

 signals:
  void ApiBadgeClicked();

 protected:
  void mousePressEvent(QMouseEvent* event) override;
  void mouseMoveEvent(QMouseEvent* event) override;
  void mouseReleaseEvent(QMouseEvent* event) override;
  void mouseDoubleClickEvent(QMouseEvent* event) override;
  bool eventFilter(QObject* watched, QEvent* event) override;

 private:
  void ToggleMaximized();
  void SyncMaximizeIcon();
  void SyncApiBadge();

  QWidget* window_;
  QMenuBar* menu_bar_ = nullptr;
  QHBoxLayout* content_;
  QPushButton* api_;
  QToolButton* maximize_ = nullptr;
  QPoint press_;
  bool pressed_ = false;
};

class WindowChrome : public QObject {
  Q_OBJECT

 public:
  explicit WindowChrome(QWidget* window);

 protected:
  bool eventFilter(QObject* watched, QEvent* event) override;

 private:
  Qt::Edges EdgesAt(const QPointF& position) const;
  void Sync();

  QWidget* window_;
  QWidget* border_;
};

}

#endif
