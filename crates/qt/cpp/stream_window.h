#ifndef OPSIS_QT_CPP_STREAM_WINDOW_H_
#define OPSIS_QT_CPP_STREAM_WINDOW_H_

#include <QImage>
#include <QLineEdit>
#include <QPointF>
#include <QSize>
#include <QString>
#include <QWidget>
#include <cstdint>

class QHBoxLayout;
class QLabel;
class QPushButton;

namespace opsis {

enum class MouseKind : std::uint8_t { kDown = 0, kUp = 1, kMove = 2 };

class StreamCanvas : public QWidget {
  Q_OBJECT

 public:
  explicit StreamCanvas(QString id, QWidget* parent = nullptr);
  void SetFrame(QImage frame, QSize page);
  QSize page() const { return page_; }
  QPointF ToPage(const QPointF& at) const;
  QPointF FromPage(const QPointF& at) const;
  bool HasPicture() const { return !frame_.isNull(); }

 protected:
  void paintEvent(QPaintEvent* event) override;
  void mousePressEvent(QMouseEvent* event) override;
  void mouseReleaseEvent(QMouseEvent* event) override;
  void mouseMoveEvent(QMouseEvent* event) override;
  void mouseDoubleClickEvent(QMouseEvent* event) override;
  void wheelEvent(QWheelEvent* event) override;
  void keyPressEvent(QKeyEvent* event) override;
  void keyReleaseEvent(QKeyEvent* event) override;
  void inputMethodEvent(QInputMethodEvent* event) override;
  bool focusNextPrevChild(bool next) override;

 private:
  void Mouse(MouseKind kind, QMouseEvent* event, int clicks);
  QRectF Target() const;

  QString id_;
  QImage frame_;
  QSize page_{1, 1};
};

class StreamWindow : public QWidget {
  Q_OBJECT

 public:
  static void Open(const QString& id, const QString& title);
  static void Close(const QString& id);
  StreamCanvas* canvas() const { return canvas_; }

 protected:
  void closeEvent(QCloseEvent* event) override;

 private:
  StreamWindow(QString id, const QString& title);
  void Sync();

  QString id_;
  std::uint64_t target_ = 0;
  std::uint64_t serial_ = 0;
  std::uint64_t frame_ = 0;
  QLineEdit* address_;
  QWidget* tabs_;
  QHBoxLayout* tab_row_;
  QWidget* dialog_;
  QLabel* dialog_text_;
  QLineEdit* dialog_prompt_;
  QPushButton* dialog_cancel_;
  StreamCanvas* canvas_;
};

}

#endif
