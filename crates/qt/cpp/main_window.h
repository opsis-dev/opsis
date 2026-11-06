#ifndef OPSIS_QT_SPIKE_CPP_MAIN_WINDOW_H_
#define OPSIS_QT_SPIKE_CPP_MAIN_WINDOW_H_

#include <kddockwidgets/qtwidgets/views/DockWidget.h>
#include <kddockwidgets/qtwidgets/views/MainWindow.h>

#include <QAction>
#include <QHash>
#include <QPointer>
#include <QStackedWidget>

namespace opsis {

class AppBar;
class GraphCanvas;
class InspectorPanel;
class NodeList;
class SettingsWindow;
class StartScreen;

class MainWindow : public KDDockWidgets::QtWidgets::MainWindow {
  Q_OBJECT

 public:
  MainWindow();

  GraphCanvas* canvas() const { return canvas_; }
  StartScreen* start_screen() const { return start_; }
  SettingsWindow* OpenSettings();
  KDDockWidgets::QtWidgets::DockWidget* inspector_dock() const {
    return inspector_dock_;
  }
  KDDockWidgets::QtWidgets::DockWidget* nodes_dock() const {
    return nodes_dock_;
  }
  void OpenExample(const QString& dir);
  void OpenFolder(const QString& dir);
  void Generate(int count);
  void CloseProject();

 protected:
  void closeEvent(QCloseEvent* event) override;

 private:
  void BuildMenus();
  void BuildDocks();
  void ShowEditor();
  void ShowStart();
  void SaveLayout();
  void SyncTitle();
  void ToggleDock(KDDockWidgets::QtWidgets::DockWidget* dock);
  void FocusDock(KDDockWidgets::QtWidgets::DockWidget* dock);
  void Dispatch(const QString& id);
  void RunRole(const QString& id);
  void SetEnabled(const QString& id, bool enabled);
  void ShowCloseDialog();
  void Guard(const std::function<void()>& action);
  KDDockWidgets::QtWidgets::DockWidget* MakeDock(const QString& id,
                                                 const QString& title,
                                                 QWidget* content);

  AppBar* app_bar_;
  QStackedWidget* stack_;
  StartScreen* start_;
  GraphCanvas* canvas_;
  KDDockWidgets::QtWidgets::DockWidget* nodes_dock_ = nullptr;
  KDDockWidgets::QtWidgets::DockWidget* inspector_dock_ = nullptr;
  KDDockWidgets::QtWidgets::DockWidget* timeline_dock_ = nullptr;
  KDDockWidgets::QtWidgets::DockWidget* mixer_dock_ = nullptr;
  KDDockWidgets::QtWidgets::DockWidget* run_dock_ = nullptr;
  bool docked_ = false;
  QHash<QString, QAction*> actions_;
  QPointer<SettingsWindow> settings_;
};

}

#endif
