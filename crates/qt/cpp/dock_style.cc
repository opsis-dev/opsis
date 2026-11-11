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

#include <QGuiApplication>
#include <QHBoxLayout>
#include <QMouseEvent>
#include <QPainter>
#include <QToolButton>
#include <QWindow>

#include "app.h"
#include "widgets.h"
#include "window_chrome.h"

namespace opsis {

namespace {

namespace kdw = KDDockWidgets;

constexpr int kTitleHeight = 32;
constexpr int kCornerIcon = 14;
constexpr int kTitleButton = 24;
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
    for (QToolButton* button : findChildren<QToolButton*>()) {
      button->setObjectName(QStringLiteral("DockTitleButton"));
      button->setFixedSize(kTitleButton, kTitleButton);
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

class DockSeparator : public kdw::QtWidgets::Separator {
 public:
  DockSeparator(kdw::Core::Separator* controller, kdw::Core::View* parent)
      : kdw::QtWidgets::Separator(controller, parent) {}

 protected:
  void paintEvent(QPaintEvent*) override {
    QPainter painter(this);
    painter.fillRect(rect(),
                     App::Get().Token(underMouse() ? "ring" : "window-border"));
  }
};

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
