#include "stream_window.h"

#include <QCloseEvent>
#include <QHBoxLayout>
#include <QHash>
#include <QInputMethodEvent>
#include <QKeyEvent>
#include <QLabel>
#include <QMouseEvent>
#include <QPainter>
#include <QPointer>
#include <QPushButton>
#include <QToolButton>
#include <QVBoxLayout>
#include <QWheelEvent>
#include <utility>

#include "app.h"
#include "opsis-qt/src/bridge.rs.h"
#include "wake_hub.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr int kWindowWidth = 1024;
constexpr int kWindowHeight = 768;
constexpr int kTabWidth = 192;
constexpr double kWheelPixels = 100.0 / 120.0;

QHash<QString, QPointer<StreamWindow>>& Windows() {
  static QHash<QString, QPointer<StreamWindow>> windows;
  return windows;
}

std::uint32_t Mods(Qt::KeyboardModifiers modifiers) {
  return static_cast<std::uint32_t>(modifiers.toInt());
}

std::uint32_t Buttons(Qt::MouseButtons buttons) {
  return static_cast<std::uint32_t>(buttons.toInt());
}

}

StreamCanvas::StreamCanvas(QString id, QWidget* parent)
    : QWidget(parent), id_(std::move(id)) {
  setObjectName(QStringLiteral("StreamCanvas"));
  setAccessibleName(QStringLiteral("Web page"));
  setFocusPolicy(Qt::StrongFocus);
  setMouseTracking(true);
  setAttribute(Qt::WA_InputMethodEnabled);
  setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Expanding);
}

void StreamCanvas::SetFrame(QImage frame, QSize page) {
  frame_ = std::move(frame);
  if (!page.isEmpty()) page_ = page;
  update();
}

QRectF StreamCanvas::Target() const {
  const QSizeF fitted = QSizeF(page_).scaled(QSizeF(size()), Qt::KeepAspectRatio);
  return {QPointF((width() - fitted.width()) / 2.0,
                  (height() - fitted.height()) / 2.0),
          fitted};
}

QPointF StreamCanvas::ToPage(const QPointF& at) const {
  const QRectF target = Target();
  const double scale = target.width() / std::max(1, page_.width());
  return {(at.x() - target.x()) / scale, (at.y() - target.y()) / scale};
}

QPointF StreamCanvas::FromPage(const QPointF& at) const {
  const QRectF target = Target();
  const double scale = target.width() / std::max(1, page_.width());
  return {target.x() + at.x() * scale, target.y() + at.y() * scale};
}

void StreamCanvas::paintEvent(QPaintEvent*) {
  QPainter painter(this);
  painter.fillRect(rect(), App::Get().Token(QStringLiteral("muted")));
  if (frame_.isNull()) return;
  painter.setRenderHint(QPainter::SmoothPixmapTransform);
  painter.drawImage(Target(), frame_);
}

void StreamCanvas::Mouse(MouseKind kind, QMouseEvent* event, int clicks) {
  const QPointF at = ToPage(event->position());
  App::Get().session().view_mouse(
      ToRust(id_), static_cast<std::uint8_t>(kind), at.x(), at.y(),
      static_cast<std::uint32_t>(event->button()), Buttons(event->buttons()),
      clicks, Mods(event->modifiers()));
}

void StreamCanvas::mousePressEvent(QMouseEvent* event) {
  setFocus();
  Mouse(MouseKind::kDown, event, 1);
}

void StreamCanvas::mouseDoubleClickEvent(QMouseEvent* event) {
  Mouse(MouseKind::kDown, event, 2);
}

void StreamCanvas::mouseReleaseEvent(QMouseEvent* event) {
  Mouse(MouseKind::kUp, event, 1);
}

void StreamCanvas::mouseMoveEvent(QMouseEvent* event) {
  Mouse(MouseKind::kMove, event, 0);
}

void StreamCanvas::wheelEvent(QWheelEvent* event) {
  const QPointF at = ToPage(event->position());
  const QPoint pixels = event->pixelDelta();
  const QPoint angle = event->angleDelta();
  const double dx = pixels.isNull() ? -angle.x() * kWheelPixels : -pixels.x();
  const double dy = pixels.isNull() ? -angle.y() * kWheelPixels : -pixels.y();
  App::Get().session().view_wheel(ToRust(id_), at.x(), at.y(), dx, dy,
                                  Mods(event->modifiers()));
}

void StreamCanvas::keyPressEvent(QKeyEvent* event) {
  App::Get().session().view_key(
      ToRust(id_), true, static_cast<std::uint32_t>(event->key()),
      ToRust(event->text()), Mods(event->modifiers()), event->isAutoRepeat());
}

void StreamCanvas::keyReleaseEvent(QKeyEvent* event) {
  if (event->isAutoRepeat()) return;
  App::Get().session().view_key(
      ToRust(id_), false, static_cast<std::uint32_t>(event->key()),
      ToRust(event->text()), Mods(event->modifiers()), false);
}

void StreamCanvas::inputMethodEvent(QInputMethodEvent* event) {
  if (!event->commitString().isEmpty()) {
    App::Get().session().view_text(ToRust(id_), ToRust(event->commitString()));
  }
}

bool StreamCanvas::focusNextPrevChild(bool) { return false; }

void StreamWindow::Open(const QString& id, const QString& title) {
  if (auto existing = Windows().value(id); !existing.isNull()) {
    existing->raise();
    existing->activateWindow();
    return;
  }
  auto* window = new StreamWindow(id, title);
  Windows().insert(id, window);
  window->show();
  window->raise();
  window->activateWindow();
}

void StreamWindow::Close(const QString& id) {
  if (auto window = Windows().value(id); !window.isNull()) window->close();
}

StreamWindow::StreamWindow(QString id, const QString& title)
    : QWidget(nullptr),
      id_(std::move(id)),
      address_(new QLineEdit(this)),
      tabs_(new QWidget(this)),
      tab_row_(new QHBoxLayout(tabs_)),
      dialog_(new QWidget(this)),
      dialog_text_(MakeLabel(QString(), nullptr, dialog_)),
      dialog_prompt_(new QLineEdit(dialog_)),
      dialog_cancel_(MakeButton(QStringLiteral("Cancel"), "ghost", QString(),
                                dialog_)),
      canvas_(new StreamCanvas(id_, this)) {
  setObjectName(QStringLiteral("StreamWindow"));
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
  row->setContentsMargins(8, 6, 8, 6);
  row->setSpacing(4);
  const QString view = id_;
  auto* back = MakeIconButton(QStringLiteral("arrow-left"),
                              QStringLiteral("Back"), bar);
  connect(back, &QToolButton::clicked, this,
          [view] { App::Get().session().view_go(ToRust(view), -1); });
  auto* forward = MakeIconButton(QStringLiteral("arrow-right"),
                                 QStringLiteral("Forward"), bar);
  connect(forward, &QToolButton::clicked, this,
          [view] { App::Get().session().view_go(ToRust(view), 1); });
  auto* reload = MakeIconButton(QStringLiteral("rotate-cw"),
                                QStringLiteral("Reload"), bar);
  connect(reload, &QToolButton::clicked, this,
          [view] { App::Get().session().view_reload(ToRust(view)); });
  address_->setAccessibleName(QStringLiteral("Address"));
  connect(address_, &QLineEdit::returnPressed, this, [this] {
    if (!address_->text().trimmed().isEmpty()) {
      App::Get().session().view_navigate(ToRust(id_),
                                         ToRust(address_->text().trimmed()));
    }
    address_->setModified(false);
    canvas_->setFocus();
  });
  row->addWidget(back);
  row->addWidget(forward);
  row->addWidget(reload);
  row->addWidget(address_, 1);
  layout->addWidget(bar);
  tabs_->setObjectName(QStringLiteral("StreamTabs"));
  tab_row_->setContentsMargins(8, 4, 8, 4);
  tab_row_->setSpacing(4);
  tabs_->hide();
  layout->addWidget(tabs_);
  dialog_->setObjectName(QStringLiteral("StreamDialog"));
  dialog_->setAttribute(Qt::WA_StyledBackground);
  auto* asked = new QHBoxLayout(dialog_);
  asked->setContentsMargins(12, 8, 12, 8);
  asked->setSpacing(8);
  dialog_text_->setSizePolicy(QSizePolicy::Ignored, QSizePolicy::Preferred);
  asked->addWidget(dialog_text_, 1);
  dialog_prompt_->setFixedWidth(kTabWidth);
  asked->addWidget(dialog_prompt_);
  asked->addWidget(dialog_cancel_);
  auto* ok = MakeButton(QStringLiteral("OK"), "primary", QString(), dialog_);
  asked->addWidget(ok);
  connect(dialog_cancel_, &QPushButton::clicked, this, [this] {
    App::Get().session().view_answer(ToRust(id_), false,
                                     ToRust(dialog_prompt_->text()));
    Sync();
  });
  connect(ok, &QPushButton::clicked, this, [this] {
    App::Get().session().view_answer(ToRust(id_), true,
                                     ToRust(dialog_prompt_->text()));
    Sync();
  });
  dialog_->hide();
  layout->addWidget(dialog_);
  layout->addWidget(canvas_, 1);
  target_ = App::Get().session().view_attach(ToRust(id_));
  WakeHub::Get().Listen(target_, this, [this] { Sync(); });
  Sync();
  canvas_->setFocus();
}

void StreamWindow::Sync() {
  auto& session = App::Get().session();
  const bridge::StreamViewState state = session.view_state(ToRust(id_));
  if (state.serial != serial_) {
    serial_ = state.serial;
    if (!address_->hasFocus() || !address_->isModified()) {
      address_->setText(ToQ(state.url));
    }
    while (QLayoutItem* item = tab_row_->takeAt(0)) {
      delete item->widget();
      delete item;
    }
    for (std::size_t i = 0; i < state.tab_ids.size(); ++i) {
      const QString tab = ToQ(state.tab_ids.at(i));
      auto* button = MakeButton(ToQ(state.tab_urls.at(i)), "toggle-outline",
                                QString(), tabs_);
      button->setCheckable(true);
      button->setChecked(i == state.active);
      button->setMaximumWidth(kTabWidth);
      connect(button, &QPushButton::clicked, this, [this, tab] {
        App::Get().session().view_switch_tab(ToRust(id_), ToRust(tab));
      });
      tab_row_->addWidget(button);
    }
    tab_row_->addStretch(1);
    tabs_->setVisible(state.tab_ids.size() > 1);
    const QString kind = ToQ(state.dialog);
    dialog_->setVisible(!kind.isEmpty());
    dialog_text_->setText(ToQ(state.message));
    dialog_prompt_->setVisible(kind == QStringLiteral("prompt"));
    dialog_prompt_->setText(ToQ(state.prompt));
    dialog_cancel_->setVisible(kind != QStringLiteral("alert"));
  }
  if (state.frame != frame_) {
    const rust::Box<bridge::FrameRef> frame = session.view_frame(ToRust(id_));
    frame_ = frame->serial();
    const auto bytes = frame->bytes();
    QImage picture = QImage::fromData(
        QByteArrayView(bytes.data(), static_cast<qsizetype>(bytes.size())),
        "JPEG");
    if (!picture.isNull()) {
      canvas_->SetFrame(std::move(picture),
                        QSize(static_cast<int>(frame->width()),
                              static_cast<int>(frame->height())));
    }
  }
}

void StreamWindow::closeEvent(QCloseEvent* event) {
  WakeHub::Get().Forget(target_, this);
  Windows().remove(id_);
  App::Get().session().view_closed(ToRust(id_));
  QWidget::closeEvent(event);
}

}
