#ifndef OPSIS_QT_CPP_LIVE_VIEW_H_
#define OPSIS_QT_CPP_LIVE_VIEW_H_

#include <QFrame>
#include <QImage>
#include <QPointer>
#include <QString>
#include <QTimer>
#include <QWidget>
#include <cstdint>
#include <memory>

class QHBoxLayout;
class QLabel;
class QPushButton;

namespace opsis {

namespace bridge {
struct LiveCaptureView;
}

class LiveSurface : public QWidget {
  Q_OBJECT

 public:
  explicit LiveSurface(bool overlays, QWidget* parent = nullptr);
  ~LiveSurface() override;
  LiveSurface(const LiveSurface&) = delete;
  LiveSurface& operator=(const LiveSurface&) = delete;
  LiveSurface(LiveSurface&&) = delete;
  LiveSurface& operator=(LiveSurface&&) = delete;
  void Show(const bridge::LiveCaptureView* capture);
  bool HasPicture() const { return !frame_.isNull(); }
  bool hasHeightForWidth() const override { return true; }
  int heightForWidth(int width) const override;

 signals:
  void Clicked();

 protected:
  void paintEvent(QPaintEvent* event) override;
  void mousePressEvent(QMouseEvent* event) override;

 private:
  bool overlays_;
  QString capture_;
  std::uint64_t serial_ = 0;
  QImage frame_;
  std::unique_ptr<bridge::LiveCaptureView> shown_;
  QTimer ripples_;
};

class LiveWindow : public QWidget {
  Q_OBJECT

 public:
  static void Open();
  static bool IsOpen();
  LiveSurface* surface() const { return surface_; }

 protected:
  void closeEvent(QCloseEvent* event) override;

 private:
  LiveWindow();
  void Sync();

  LiveSurface* surface_;
};

class LivePip : public QFrame {
  Q_OBJECT

 public:
  LivePip(QWidget* host, QPushButton* toggle);
  LiveSurface* surface() const { return surface_; }
  void Sync();

 protected:
  bool eventFilter(QObject* watched, QEvent* event) override;

 private:
  void Place();

  QWidget* host_;
  QPointer<QPushButton> toggle_;
  QLabel* title_;
  QLabel* count_;
  LiveSurface* surface_;
  QWidget* switcher_;
  QHBoxLayout* switcher_row_;
  QString switcher_key_;
};

}

#endif
