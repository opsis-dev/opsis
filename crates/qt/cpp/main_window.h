#ifndef OPSIS_QT_CPP_MAIN_WINDOW_H_
#define OPSIS_QT_CPP_MAIN_WINDOW_H_

#include <kddockwidgets/qtwidgets/views/DockWidget.h>
#include <kddockwidgets/qtwidgets/views/MainWindow.h>

#include <QAction>
#include <QHash>
#include <QPointer>
#include <QStackedWidget>
#include <QTimer>

namespace opsis {

class AppBar;
class GraphCanvas;
class InspectorPanel;
class LivePip;
class CategoryRail;
class TopBar;
class NodeList;
class SettingsWindow;
class StartScreen;

class MainWindow : public KDDockWidgets::QtWidgets::MainWindow {
  Q_OBJECT

 public:
  MainWindow();

  GraphCanvas* canvas() const { return canvas_; }
  StartScreen* start_screen() const { return start_; }
  CategoryRail* rail() const { return rail_; }
  TopBar* top_bar() const { return top_bar_; }
  SettingsWindow* OpenSettings();
  LivePip* live_pip() const { return live_pip_; }
  KDDockWidgets::QtWidgets::DockWidget* inspector_dock() const {
    return inspector_dock_;
  }
  KDDockWidgets::QtWidgets::DockWidget* nodes_dock() const {
    return nodes_dock_;
  }
  KDDockWidgets::QtWidgets::DockWidget* mixer_dock() const {
    return mixer_dock_;
  }
  KDDockWidgets::QtWidgets::DockWidget* timeline_dock() const {
    return timeline_dock_;
  }
  void OpenExample(const QString& dir);
  void OpenFolder(const QString& dir);
  void Generate(int count);
  void CloseProject();

  Q_INVOKABLE void RunCommand(const QString& id) { Dispatch(id); }

 protected:
  void closeEvent(QCloseEvent* event) override;
  void moveEvent(QMoveEvent* event) override;
  void resizeEvent(QResizeEvent* event) override;

 private:
  void BuildMenus();
  void BuildDocks();
  void ShowEditor();
  void ShowStart();
  void AddNode(const QString& type);
  void SyncRail();
  void SyncPanels();
  void ToggleBottom();
  bool BottomOpen() const;
  void QuickAdd();
  void SaveLayout();
  void SyncTitle();
  void ToggleDock(KDDockWidgets::QtWidgets::DockWidget* dock);
  void FocusDock(KDDockWidgets::QtWidgets::DockWidget* dock);
  void Dispatch(const QString& id);
  void NewProject(const QString& template_id, bool fixed);
  void RunRole(const QString& id);
  void SetEnabled(const QString& id, bool enabled);
  void ShowCloseDialog();
  void ApplyProjectWindow();
  void SaveWindow();
  void Guard(const std::function<void()>& action);
  KDDockWidgets::QtWidgets::DockWidget* MakeDock(const QString& id,
                                                 const QString& title,
                                                 QWidget* content);

  AppBar* app_bar_;
  QStackedWidget* stack_;
  CategoryRail* rail_;
  TopBar* top_bar_;
  LivePip* live_pip_ = nullptr;
  KDDockWidgets::QtWidgets::DockWidget* bottom_current_ = nullptr;
  StartScreen* start_;
  GraphCanvas* canvas_;
  KDDockWidgets::QtWidgets::DockWidget* nodes_dock_ = nullptr;
  KDDockWidgets::QtWidgets::DockWidget* inspector_dock_ = nullptr;
  KDDockWidgets::QtWidgets::DockWidget* timeline_dock_ = nullptr;
  KDDockWidgets::QtWidgets::DockWidget* mixer_dock_ = nullptr;
  KDDockWidgets::QtWidgets::DockWidget* run_dock_ = nullptr;
  bool docked_ = false;
  QHash<QString, QAction*> actions_;
  QTimer save_window_;
  QPointer<SettingsWindow> settings_;
};

}

#endif
