#ifndef OPSIS_QT_CPP_FILE_PLAYER_H_
#define OPSIS_QT_CPP_FILE_PLAYER_H_

#include <QFrame>
#include <QImage>
#include <QLabel>
#include <QPixmap>
#include <QPointer>
#include <QString>
#include <QTimer>
#include <QToolButton>
#include <QWidget>
#include <cstdint>

class QSlider;
class QStackedWidget;

namespace opsis {

class PlayerSurface : public QWidget {
  Q_OBJECT

 public:
  explicit PlayerSurface(QWidget* parent = nullptr);
  void SetFrame(QImage frame);
  void SetAspect(int width, int height);
  bool hasHeightForWidth() const override { return true; }
  int heightForWidth(int width) const override;
  QSize sizeHint() const override;

 signals:
  void Clicked();
  void DoubleClicked();

 protected:
  void paintEvent(QPaintEvent* event) override;
  void mousePressEvent(QMouseEvent* event) override;
  void mouseDoubleClickEvent(QMouseEvent* event) override;

 private:
  QImage frame_;
  int aspect_width_ = 16;
  int aspect_height_ = 9;
};

class SeekBar : public QWidget {
  Q_OBJECT

 public:
  explicit SeekBar(QWidget* parent = nullptr);
  void SetRange(double duration);
  void SetPosition(double seconds);
  bool Dragging() const { return dragging_; }
  QSize sizeHint() const override { return {120, 14}; }
  QSize minimumSizeHint() const override { return {40, 14}; }

 signals:
  void Seeked(double seconds);

 protected:
  void paintEvent(QPaintEvent* event) override;
  void mousePressEvent(QMouseEvent* event) override;
  void mouseMoveEvent(QMouseEvent* event) override;
  void mouseReleaseEvent(QMouseEvent* event) override;
  void keyPressEvent(QKeyEvent* event) override;

 private:
  double SecondsAt(double x) const;

  double duration_ = 0;
  double position_ = 0;
  bool dragging_ = false;
};

class FilePlayer : public QFrame {
  Q_OBJECT

 public:
  FilePlayer(const QString& file, const QString& label, bool video,
             QWidget* parent = nullptr);
  ~FilePlayer() override;
  FilePlayer(const FilePlayer&) = delete;
  FilePlayer& operator=(const FilePlayer&) = delete;
  FilePlayer(FilePlayer&&) = delete;
  FilePlayer& operator=(FilePlayer&&) = delete;
  std::uint64_t id() const { return id_; }

 protected:
  void showEvent(QShowEvent* event) override;
  void hideEvent(QHideEvent* event) override;
  void keyPressEvent(QKeyEvent* event) override;

 private:
  void Tick();
  void TogglePlay();
  void SetFullscreen(bool on);
  void ShowSpeedMenu();
  void ApplyVolume();

  std::uint64_t id_;
  bool video_;
  QString label_;
  QTimer timer_;
  QStackedWidget* stage_ = nullptr;
  PlayerSurface* surface_ = nullptr;
  QWidget* failure_ = nullptr;
  QLabel* loading_ = nullptr;
  SeekBar* seek_;
  QToolButton* play_;
  QToolButton* mute_;
  QSlider* volume_;
  QLabel* time_;
  QToolButton* speed_;
  QToolButton* fullscreen_ = nullptr;
  std::uint64_t serial_ = 0;
  bool playing_ = false;
  bool failed_ = false;
  bool muted_ = false;
  double rate_ = 1;
  QPointer<QWidget> placeholder_;
};

class ImagePreview : public QLabel {
  Q_OBJECT

 public:
  ImagePreview(const QString& file, const QString& label, int width,
               QWidget* parent = nullptr);

 protected:
  void mousePressEvent(QMouseEvent* event) override;
  void keyPressEvent(QKeyEvent* event) override;

 private:
  void Enlarge();

  QString file_;
  QString label_;
};

}

#endif
