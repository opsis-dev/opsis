#include "dock_style.h"

#include <QPainter>
#include <kddockwidgets/core/Group.h>
#include <kddockwidgets/core/TitleBar.h>
#include <kddockwidgets/qtcommon/View.h>
#include <kddockwidgets/qtwidgets/views/Group.h>
#include <kddockwidgets/qtwidgets/views/Separator.h>
#include <kddockwidgets/qtwidgets/views/TitleBar.h>

#include "app.h"

namespace opsis {

namespace {

namespace kdw = KDDockWidgets;

constexpr int kTitleHeight = 32;

class DockTitleBar : public kdw::QtWidgets::TitleBar {
 public:
  DockTitleBar(kdw::Core::TitleBar* controller, kdw::Core::View* parent)
      : kdw::QtWidgets::TitleBar(controller, parent), controller_(controller) {
    setFixedHeight(kTitleHeight);
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

class DockSeparator : public kdw::QtWidgets::Separator {
 public:
  DockSeparator(kdw::Core::Separator* controller, kdw::Core::View* parent)
      : kdw::QtWidgets::Separator(controller, parent) {}

 protected:
  void paintEvent(QPaintEvent*) override {
    QPainter painter(this);
    painter.fillRect(rect(), App::Get().Token(
                                 underMouse() ? "ring" : "window-border"));
  }
};

}

kdw::Core::View* DockViewFactory::createTitleBar(
    kdw::Core::TitleBar* controller, kdw::Core::View* parent) const {
  return new DockTitleBar(controller, parent);
}

kdw::Core::View* DockViewFactory::createGroup(kdw::Core::Group* controller,
                                              kdw::Core::View* parent) const {
  return new DockGroup(controller, kdw::QtCommon::View_qt::asQWidget(parent));
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
