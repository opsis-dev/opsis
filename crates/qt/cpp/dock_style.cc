#include "dock_style.h"

#include <kddockwidgets/core/Group.h>
#include <kddockwidgets/core/TitleBar.h>
#include <kddockwidgets/qtcommon/View.h>
#include <kddockwidgets/qtwidgets/views/DockWidget.h>
#include <kddockwidgets/qtwidgets/views/FloatingWindow.h>
#include <kddockwidgets/qtwidgets/views/Group.h>
#include <kddockwidgets/qtwidgets/views/Separator.h>
#include <kddockwidgets/qtwidgets/views/Stack.h>
#include <kddockwidgets/qtwidgets/views/TitleBar.h>

#include <QCoreApplication>
#include <QGuiApplication>
#include <QHBoxLayout>
#include <QMouseEvent>
#include <QPainter>
#include <QPointer>
#include <QToolButton>
#include <QWindow>
#include <algorithm>

#include "app.h"
#include "widgets.h"
#include "window_chrome.h"

namespace opsis {

namespace {

namespace kdw = KDDockWidgets;

constexpr int kTitleHeight = 32;
constexpr int kCornerIcon = 14;
constexpr int kTitleButton = 24;
constexpr double kLineWidth = 2.0;
constexpr int kHandleShort = 12;
constexpr int kHandleLong = 16;
constexpr int kHandleIcon = 10;
constexpr double kHandleRadius = 2.0;
constexpr double kQuarterTurn = 90.0;
constexpr int kTitleIcon = 14;

bool OnWayland() {
  return QGuiApplication::platformName() == QStringLiteral("wayland");
}

class DockTitleBar : public kdw::QtWidgets::TitleBar {
 public:
  DockTitleBar(kdw::Core::TitleBar* controller, kdw::Core::View* parent)
      : kdw::QtWidgets::TitleBar(controller, parent), controller_(controller) {
    setFixedHeight(kTitleHeight);
  }

  void setVisible(bool visible) override {
    const kdw::Core::Group* group = controller_->group();
    const bool docked = group != nullptr && group->isInMainWindow();
    setFixedHeight(docked ? 0 : kTitleHeight);
    kdw::QtWidgets::TitleBar::setVisible(visible);
    const int side = docked ? 0 : kTitleButton;
    for (QToolButton* button : findChildren<QToolButton*>()) {
      button->setObjectName(QStringLiteral("DockTitleButton"));
      button->setFixedSize(side, side);
      button->setIconSize(QSize(kTitleIcon, kTitleIcon));
    }
  }

 protected:
  void showEvent(QShowEvent* event) override {
    kdw::QtWidgets::TitleBar::showEvent(event);
    removeEventFilter(this);
    installEventFilter(this);
  }

  bool eventFilter(QObject* watched, QEvent* event) override {
    if (watched == this && event->type() == QEvent::MouseButtonPress &&
        OnWayland() && controller_->isFloating()) {
      const auto* mouse = static_cast<QMouseEvent*>(event);
      QWindow* handle = QWidget::window()->windowHandle();
      if (mouse->button() == Qt::LeftButton && handle != nullptr &&
          childAt(mouse->position().toPoint()) == nullptr) {
        handle->startSystemMove();
        return true;
      }
    }
    return kdw::QtWidgets::TitleBar::eventFilter(watched, event);
  }

 protected:
  void paintEvent(QPaintEvent*) override {
    App& app = App::Get();
    QPainter painter(this);
    painter.fillRect(rect(), app.Token("sidebar"));
    painter.setPen(app.Token("border"));
    painter.drawLine(rect().bottomLeft(), rect().bottomRight());
    QFont font = painter.font();
    font.setWeight(QFont::Medium);
    painter.setFont(font);
    painter.setPen(app.Token("foreground"));
    painter.drawText(rect().adjusted(12, 0, -100, 0),
                     Qt::AlignVCenter | Qt::AlignLeft, controller_->title());
  }

 private:
  kdw::Core::TitleBar* controller_;
};

class DockGroup : public kdw::QtWidgets::Group {
 public:
  DockGroup(kdw::Core::Group* controller, QWidget* parent)
      : kdw::QtWidgets::Group(controller, parent) {}

 protected:
  void paintEvent(QPaintEvent*) override {
    QPainter painter(this);
    painter.fillRect(rect(), App::Get().Token("sidebar"));
  }
};

class DockStack : public kdw::QtWidgets::Stack {
 public:
  DockStack(kdw::Core::Stack* controller, QWidget* parent)
      : kdw::QtWidgets::Stack(controller, parent) {
    auto* corner = new QWidget(this);
    auto* row = new QHBoxLayout(corner);
    row->setContentsMargins(0, 0, 4, 0);
    row->setSpacing(2);
    pop_out_ = MakeIconButton(QStringLiteral("square-arrow-out-up-right"),
                              QString(), corner, kCornerIcon);
    auto* collapse = MakeIconButton(QStringLiteral("chevron-down"),
                                    QStringLiteral("Hide the bottom panel"),
                                    corner, kCornerIcon);
    for (QToolButton* button : {pop_out_, collapse}) {
      button->setFocusPolicy(Qt::NoFocus);
      row->addWidget(button);
    }
    connect(pop_out_, &QToolButton::clicked, this, [this] {
      if (auto* dock =
              qobject_cast<kdw::QtWidgets::DockWidget*>(currentWidget())) {
        dock->setFloating(true);
      }
    });
    connect(collapse, &QToolButton::clicked, this, [this] {
      QMetaObject::invokeMethod(
          QWidget::window(), "RunCommand",
          Q_ARG(QString, QStringLiteral("view:toggle-bottom")));
    });
    connect(this, &QTabWidget::currentChanged, this, &DockStack::SyncCorner);
    setCornerWidget(corner, Qt::TopRightCorner);
    corner->hide();
  }

 protected:
  void tabInserted(int index) override {
    kdw::QtWidgets::Stack::tabInserted(index);
    SyncCorner();
  }
  void tabRemoved(int index) override {
    kdw::QtWidgets::Stack::tabRemoved(index);
    SyncCorner();
  }

 private:
  void SyncCorner() {
    QWidget* corner = cornerWidget(Qt::TopRightCorner);
    if (corner == nullptr) return;
    corner->setVisible(count() > 1);
    pop_out_->setToolTip(QStringLiteral("Open %1 in a new window")
                             .arg(tabText(currentIndex()).toLower()));
  }

  QToolButton* pop_out_;
};

bool Across(const QWidget* separator) {
  return separator->width() >= separator->height();
}

class DockSeparator;

class SplitterHandle : public QWidget {
 public:
  explicit SplitterHandle(DockSeparator* separator);
  bool inside() const { return inside_; }
  void Follow();

 protected:
  bool eventFilter(QObject* watched, QEvent* event) override;
  void paintEvent(QPaintEvent*) override;
  void enterEvent(QEnterEvent*) override;
  void leaveEvent(QEvent*) override;
  void mousePressEvent(QMouseEvent* event) override { Forward(event); }
  void mouseMoveEvent(QMouseEvent* event) override { Forward(event); }
  void mouseReleaseEvent(QMouseEvent* event) override { Forward(event); }
  void mouseDoubleClickEvent(QMouseEvent* event) override { Forward(event); }

 private:
  void Watch(QWidget* parent);
  void Forward(QMouseEvent* event);

  DockSeparator* separator_;
  bool inside_ = false;
};

class DockSeparator : public kdw::QtWidgets::Separator {
 public:
  DockSeparator(kdw::Core::Separator* controller, kdw::Core::View* parent)
      : kdw::QtWidgets::Separator(controller, parent),
        handle_(new SplitterHandle(this)) {
    setAttribute(Qt::WA_Hover);
    handle_->Follow();
  }
  ~DockSeparator() override { delete handle_.data(); }
  DockSeparator(const DockSeparator&) = delete;
  DockSeparator& operator=(const DockSeparator&) = delete;
  DockSeparator(DockSeparator&&) = delete;
  DockSeparator& operator=(DockSeparator&&) = delete;

  bool Hot() const {
    return inside_ || (handle_ != nullptr && handle_->inside());
  }

  void Refresh() {
    update();
    if (handle_ != nullptr) handle_->update();
  }

  void raise() override {
    kdw::QtWidgets::Separator::raise();
    if (handle_ != nullptr) handle_->raise();
  }

 protected:
  bool event(QEvent* event) override {
    const bool handled = kdw::QtWidgets::Separator::event(event);
    switch (event->type()) {
      case QEvent::Move:
      case QEvent::Resize:
      case QEvent::Show:
      case QEvent::Hide:
      case QEvent::ParentChange:
        if (handle_ != nullptr) handle_->Follow();
        break;
      default:
        break;
    }
    return handled;
  }

  void enterEvent(QEnterEvent* event) override {
    kdw::QtWidgets::Separator::enterEvent(event);
    inside_ = true;
    Refresh();
  }

  void leaveEvent(QEvent* event) override {
    kdw::QtWidgets::Separator::leaveEvent(event);
    inside_ = false;
    Refresh();
  }

  void paintEvent(QPaintEvent*) override {
    App& app = App::Get();
    QPainter painter(this);
    painter.fillRect(rect(), app.Token("sidebar"));
    const QRectF area = rect();
    const QRectF line =
        Across(this) ? QRectF(area.left(), (area.height() - kLineWidth) / 2.0,
                              area.width(), kLineWidth)
                     : QRectF((area.width() - kLineWidth) / 2.0, area.top(),
                              kLineWidth, area.height());
    painter.fillRect(line, app.Token(Hot() ? "ring" : "window-border"));
  }

 private:
  QPointer<SplitterHandle> handle_;
  bool inside_ = false;
};

SplitterHandle::SplitterHandle(DockSeparator* separator)
    : QWidget(separator->parentWidget()), separator_(separator) {
  setObjectName(QStringLiteral("SplitterHandle"));
  setAttribute(Qt::WA_Hover);
  setMouseTracking(true);
  Watch(parentWidget());
}

void SplitterHandle::Watch(QWidget* parent) {
  if (parent != nullptr) parent->installEventFilter(this);
}

void SplitterHandle::Follow() {
  QWidget* parent = separator_->parentWidget();
  if (parent != parentWidget()) {
    if (parentWidget() != nullptr) parentWidget()->removeEventFilter(this);
    setParent(parent);
    Watch(parent);
  }
  const bool across = Across(separator_);
  QRect box(QPoint(), across ? QSize(kHandleLong, kHandleShort)
                             : QSize(kHandleShort, kHandleLong));
  box.moveCenter(separator_->geometry().center());
  setGeometry(box);
  setCursor(across ? Qt::SizeVerCursor : Qt::SizeHorCursor);
  setVisible(parent != nullptr && separator_->isVisible() &&
             std::max(separator_->width(), separator_->height()) > kHandleLong);
  raise();
}

bool SplitterHandle::eventFilter(QObject* watched, QEvent* event) {
  if (watched == parentWidget() && event->type() == QEvent::ChildAdded) {
    QMetaObject::invokeMethod(this, &QWidget::raise, Qt::QueuedConnection);
  }
  return QWidget::eventFilter(watched, event);
}

void SplitterHandle::paintEvent(QPaintEvent*) {
  App& app = App::Get();
  const bool hot = separator_->Hot();
  QPainter painter(this);
  painter.setRenderHint(QPainter::Antialiasing);
  painter.setPen(QPen(app.Token(hot ? "ring" : "window-border"), 1.0));
  painter.setBrush(app.Token("window-border"));
  painter.drawRoundedRect(QRectF(rect()).adjusted(0.5, 0.5, -0.5, -0.5),
                          kHandleRadius, kHandleRadius);
  const QPixmap grip = app.Glyph(
      QStringLiteral("grip-vertical"),
      app.Token(hot ? "foreground" : "muted-foreground"), kHandleIcon);
  painter.translate(QRectF(rect()).center());
  if (Across(separator_)) painter.rotate(kQuarterTurn);
  painter.drawPixmap(QPointF(-kHandleIcon / 2.0, -kHandleIcon / 2.0), grip);
}

void SplitterHandle::enterEvent(QEnterEvent*) {
  inside_ = true;
  separator_->Refresh();
}

void SplitterHandle::leaveEvent(QEvent*) {
  inside_ = false;
  separator_->Refresh();
}

void SplitterHandle::Forward(QMouseEvent* event) {
  QMouseEvent mapped(
      event->type(),
      separator_->QWidget::mapFromGlobal(event->globalPosition()),
      event->scenePosition(), event->globalPosition(), event->button(),
      event->buttons(), event->modifiers());
  QCoreApplication::sendEvent(separator_, &mapped);
}

}

kdw::Core::View* DockViewFactory::createTitleBar(
    kdw::Core::TitleBar* controller, kdw::Core::View* parent) const {
  return new DockTitleBar(controller, parent);
}

kdw::Core::View* DockViewFactory::createFloatingWindow(
    kdw::Core::FloatingWindow* controller, kdw::Core::MainWindow* parent,
    Qt::WindowFlags flags) const {
  kdw::Core::View* view = kdw::QtWidgets::ViewFactory::createFloatingWindow(
      controller, parent, flags);
  if (OnWayland()) new WindowChrome(kdw::QtCommon::View_qt::asQWidget(view));
  return view;
}

kdw::Core::View* DockViewFactory::createGroup(kdw::Core::Group* controller,
                                              kdw::Core::View* parent) const {
  return new DockGroup(controller, kdw::QtCommon::View_qt::asQWidget(parent));
}

kdw::Core::View* DockViewFactory::createStack(kdw::Core::Stack* controller,
                                              kdw::Core::View* parent) const {
  return new DockStack(controller, kdw::QtCommon::View_qt::asQWidget(parent));
}

kdw::Core::View* DockViewFactory::createSeparator(
    kdw::Core::Separator* controller, kdw::Core::View* parent) const {
  return new DockSeparator(controller, parent);
}

QIcon DockViewFactory::iconForButtonType(kdw::TitleBarButtonType type,
                                         qreal) const {
  QString name;
  switch (type) {
    case kdw::TitleBarButtonType::Close:
      name = QStringLiteral("x");
      break;
    case kdw::TitleBarButtonType::Float:
      name = QStringLiteral("copy");
      break;
    case kdw::TitleBarButtonType::Minimize:
      name = QStringLiteral("minus");
      break;
    case kdw::TitleBarButtonType::Maximize:
      name = QStringLiteral("maximize-2");
      break;
    default:
      name = QStringLiteral("square");
      break;
  }
  return App::Get().Icon(name, App::Get().Token("muted-foreground"), 14);
}

}
