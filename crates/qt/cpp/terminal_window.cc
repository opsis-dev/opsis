#include "terminal_window.h"

#include <QApplication>
#include <QClipboard>
#include <QCloseEvent>
#include <QHBoxLayout>
#include <QHash>
#include <QKeyEvent>
#include <QLabel>
#include <QMouseEvent>
#include <QPainter>
#include <QPointer>
#include <QPushButton>
#include <QVBoxLayout>
#include <QWheelEvent>
#include <cmath>
#include <utility>

#include "app.h"
#include "opsis-qt/src/bridge.rs.h"
#include "wake_hub.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr int kWindowWidth = 960;
constexpr int kWindowHeight = 640;
constexpr double kWheelStep = 120;

enum class Action : std::uint8_t { kPress = 0, kRelease = 1, kMove = 2 };

QHash<QString, QPointer<TerminalWindow>>& Windows() {
  static QHash<QString, QPointer<TerminalWindow>> windows;
  return windows;
}

}

TerminalCanvas::TerminalCanvas(QString id, QWidget* parent)
    : QWidget(parent), id_(std::move(id)) {
  setObjectName(QStringLiteral("TerminalCanvas"));
  setAccessibleName(QStringLiteral("Terminal"));
  setFocusPolicy(Qt::StrongFocus);
  setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Expanding);
}

void TerminalCanvas::SetFrame(QImage frame) {
  frame_ = std::move(frame);
  pictures_ += 1;
  update();
}

void TerminalCanvas::paintEvent(QPaintEvent*) {
  QPainter painter(this);
  painter.fillRect(rect(), App::Get().Token(QStringLiteral("background")));
  if (frame_.isNull()) return;
  painter.setRenderHint(QPainter::SmoothPixmapTransform);
  const QSize fitted = frame_.size().scaled(size(), Qt::KeepAspectRatio);
  painter.drawImage(QRect(QPoint((width() - fitted.width()) / 2,
                                 (height() - fitted.height()) / 2),
                          fitted),
                    frame_);
}

void TerminalCanvas::keyPressEvent(QKeyEvent* event) {
  const auto mods = event->modifiers();
  if (mods.testFlag(Qt::ControlModifier) && mods.testFlag(Qt::ShiftModifier) &&
      event->key() == Qt::Key_V) {
    App::Get().session().terminal_paste(
        ToRust(id_), ToRust(QApplication::clipboard()->text()));
    return;
  }
  App::Get().session().terminal_key(
      ToRust(id_), static_cast<std::uint32_t>(event->key()),
      ToRust(event->text()), static_cast<std::uint32_t>(mods.toInt()));
}

void TerminalCanvas::Pointer(std::uint8_t action, QMouseEvent* event) {
  const QPointF at = event->position();
  App::Get().session().terminal_pointer(
      ToRust(id_), action,
      static_cast<std::uint32_t>(
          action == static_cast<std::uint8_t>(Action::kMove) ? Qt::NoButton
                                                             : event->button()),
      at.x(), at.y(), width(), height());
}

void TerminalCanvas::mousePressEvent(QMouseEvent* event) {
  setFocus();
  Pointer(static_cast<std::uint8_t>(Action::kPress), event);
}

void TerminalCanvas::mouseReleaseEvent(QMouseEvent* event) {
  Pointer(static_cast<std::uint8_t>(Action::kRelease), event);
}

void TerminalCanvas::mouseMoveEvent(QMouseEvent* event) {
  if (event->buttons() == Qt::NoButton) return;
  Pointer(static_cast<std::uint8_t>(Action::kMove), event);
}

void TerminalCanvas::wheelEvent(QWheelEvent* event) {
  const double delta = -event->angleDelta().y();
  if (delta == 0) return;
  const int notches = static_cast<int>(std::copysign(
      std::max(1.0, std::round(std::abs(delta) / kWheelStep)), delta));
  const QPointF at = event->position();
  App::Get().session().terminal_wheel(ToRust(id_), at.x(), at.y(), width(),
                                      height(), notches);
}

void TerminalCanvas::focusInEvent(QFocusEvent* event) {
  QWidget::focusInEvent(event);
  App::Get().session().terminal_focus(ToRust(id_), true);
}

void TerminalCanvas::focusOutEvent(QFocusEvent* event) {
  QWidget::focusOutEvent(event);
  App::Get().session().terminal_focus(ToRust(id_), false);
}

bool TerminalCanvas::focusNextPrevChild(bool) { return false; }

void TerminalWindow::Open(const QString& id, const QString& title) {
  if (auto existing = Windows().value(id); !existing.isNull()) {
    existing->raise();
    existing->activateWindow();
    return;
  }
  auto* window = new TerminalWindow(id, title);
  Windows().insert(id, window);
  window->show();
  window->raise();
  window->activateWindow();
  window->canvas()->setFocus();
}

void TerminalWindow::Close(const QString& id) {
  if (auto window = Windows().value(id); !window.isNull()) window->close();
}

TerminalWindow::TerminalWindow(QString id, const QString& title)
    : QWidget(nullptr),
      id_(std::move(id)),
      status_(MakeLabel(QString(), "muted", this)),
      canvas_(new TerminalCanvas(id_, this)) {
  setObjectName(QStringLiteral("TerminalWindow"));
  setAttribute(Qt::WA_DeleteOnClose);
  setAttribute(Qt::WA_StyledBackground);
  setWindowTitle(title);
  resize(kWindowWidth, kWindowHeight);
  auto* layout = new QVBoxLayout(this);
  layout->setContentsMargins(0, 0, 0, 0);
  layout->setSpacing(0);
  auto* bar = new QWidget(this);
  bar->setObjectName(QStringLiteral("TimelineToolbar"));
  bar->setAttribute(Qt::WA_StyledBackground);
  auto* row = new QHBoxLayout(bar);
  row->setContentsMargins(12, 6, 8, 6);
  row->setSpacing(8);
  status_->setSizePolicy(QSizePolicy::Ignored, QSizePolicy::Preferred);
  row->addWidget(status_, 1);
  auto* cancel = MakeButton(QStringLiteral("Cancel"), "ghost", QString(), bar);
  connect(cancel, &QPushButton::clicked, this, [this] { Finish(false); });
  row->addWidget(cancel);
  auto* save =
      MakeButton(QStringLiteral("Save tape"), "primary", QString(), bar);
  connect(save, &QPushButton::clicked, this, [this] { Finish(true); });
  row->addWidget(save);
  layout->addWidget(bar);
  layout->addWidget(canvas_, 1);
  target_ = App::Get().session().terminal_attach(ToRust(id_));
  WakeHub::Get().Listen(target_, this, [this] { Sync(); });
  Sync();
}

void TerminalWindow::Sync() {
  auto& session = App::Get().session();
  const bridge::TerminalWindowView view = session.terminal_view(ToRust(id_));
  status_->setText(ToQ(view.status));
  if (view.frame == frame_) return;
  const rust::Box<bridge::FrameRef> frame = session.terminal_frame(ToRust(id_));
  frame_ = frame->serial();
  const auto bytes = frame->bytes();
  QImage picture = QImage::fromData(
      QByteArrayView(bytes.data(), static_cast<qsizetype>(bytes.size())),
      "JPEG");
  if (!picture.isNull()) canvas_->SetFrame(std::move(picture));
}

void TerminalWindow::Finish(bool save) {
  try {
    App::Get().session().terminal_finish(ToRust(id_), save);
  } catch (const rust::Error& error) {
    App::Get().ShowError(QString::fromUtf8(error.what()));
  }
}

void TerminalWindow::closeEvent(QCloseEvent* event) {
  WakeHub::Get().Forget(target_, this);
  Windows().remove(id_);
  App::Get().session().terminal_closed(ToRust(id_));
  QWidget::closeEvent(event);
}

}
