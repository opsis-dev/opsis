#ifndef OPSIS_QT_CPP_DOCK_STYLE_H_
#define OPSIS_QT_CPP_DOCK_STYLE_H_

#include <kddockwidgets/qtwidgets/ViewFactory.h>

namespace opsis {

class DockViewFactory : public KDDockWidgets::QtWidgets::ViewFactory {
  Q_OBJECT

 public:
  DockViewFactory() = default;

  KDDockWidgets::Core::View* createTitleBar(
      KDDockWidgets::Core::TitleBar* controller,
      KDDockWidgets::Core::View* parent) const override;
  KDDockWidgets::Core::View* createFloatingWindow(
      KDDockWidgets::Core::FloatingWindow* controller,
      KDDockWidgets::Core::MainWindow* parent = nullptr,
      Qt::WindowFlags flags = {}) const override;
  KDDockWidgets::Core::View* createGroup(
      KDDockWidgets::Core::Group* controller,
      KDDockWidgets::Core::View* parent) const override;
  KDDockWidgets::Core::View* createStack(
      KDDockWidgets::Core::Stack* controller,
      KDDockWidgets::Core::View* parent) const override;
  KDDockWidgets::Core::View* createSeparator(
      KDDockWidgets::Core::Separator* controller,
      KDDockWidgets::Core::View* parent) const override;
  QIcon iconForButtonType(KDDockWidgets::TitleBarButtonType type,
                          qreal ratio) const override;
};

}

#endif
