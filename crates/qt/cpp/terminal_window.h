#ifndef OPSIS_QT_CPP_TERMINAL_WINDOW_H_
#define OPSIS_QT_CPP_TERMINAL_WINDOW_H_

#include <QImage>
#include <QString>
#include <QWidget>
#include <cstdint>

class QLabel;

namespace opsis {

class TerminalCanvas : public QWidget {
  Q_OBJECT

 public:
  explicit TerminalCanvas(QString id, QWidget* parent = nullptr);
  void SetFrame(QImage frame);
  bool HasPicture() const { return !frame_.isNull(); }
  std::uint64_t pictures() const { return pictures_; }

 protected:
  void paintEvent(QPaintEvent* event) override;
  void keyPressEvent(QKeyEvent* event) override;
  void mousePressEvent(QMouseEvent* event) override;
  void mouseReleaseEvent(QMouseEvent* event) override;
  void mouseMoveEvent(QMouseEvent* event) override;
  void wheelEvent(QWheelEvent* event) override;
  void focusInEvent(QFocusEvent* event) override;
  void focusOutEvent(QFocusEvent* event) override;
  bool focusNextPrevChild(bool next) override;

 private:
  void Pointer(std::uint8_t action, QMouseEvent* event);

  QString id_;
  QImage frame_;
  std::uint64_t pictures_ = 0;
};

class TerminalWindow : public QWidget {
  Q_OBJECT

 public:
  static void Open(const QString& id, const QString& title);
  static void Close(const QString& id);
  TerminalCanvas* canvas() const { return canvas_; }

 protected:
  void closeEvent(QCloseEvent* event) override;

 private:
  TerminalWindow(QString id, const QString& title);
  void Sync();
  void Finish(bool save);

  QString id_;
  std::uint64_t target_ = 0;
  std::uint64_t frame_ = 0;
  QLabel* status_;
  TerminalCanvas* canvas_;
};

}

#endif
