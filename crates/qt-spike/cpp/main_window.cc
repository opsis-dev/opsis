#include "main_window.h"

#include <QApplication>
#include <QCloseEvent>
#include <QFileDialog>
#include <QMenu>
#include <QMenuBar>
#include <QVBoxLayout>
#include <functional>
#include <kddockwidgets/LayoutSaver.h>

#include "app.h"
#include "event_hub.h"
#include "graph_canvas.h"
#include "inspector_panel.h"
#include "node_list.h"
#include "opsis-qt-spike/src/bridge.rs.h"
#include "settings_window.h"
#include "start_screen.h"
#include "widgets.h"
#include "window_chrome.h"

namespace opsis {

namespace {

namespace kdw = KDDockWidgets;

constexpr int kSideWidth = 260;
constexpr int kInspectorWidth = 340;
constexpr int kBottomHeight = 240;

QWidget* Placeholder(const QString& text) {
  auto* panel = new QWidget();
  panel->setObjectName(QStringLiteral("Panel"));
  panel->setAttribute(Qt::WA_StyledBackground);
  auto* layout = new QVBoxLayout(panel);
  auto* label = MakeLabel(text, "muted", panel);
  label->setAlignment(Qt::AlignCenter);
  layout->addWidget(label);
  return panel;
}

}

MainWindow::MainWindow()
    : kdw::QtWidgets::MainWindow(
          QStringLiteral("Opsis"),
          kdw::MainWindowOptions(kdw::MainWindowOption_HasCentralWidget) |
              kdw::MainWindowOption_CentralWidgetGetsAllExtraSpace) {
  setObjectName(QStringLiteral("Window"));
  setAttribute(Qt::WA_StyledBackground);
  setWindowTitle(QStringLiteral("Opsis"));
  resize(1440, 900);
  new WindowChrome(this);
  app_bar_ = new AppBar(this, QStringLiteral("Opsis"), true);
  setMenuWidget(app_bar_);
  stack_ = new QStackedWidget(this);
  start_ = new StartScreen(stack_);
  canvas_ = new GraphCanvas(stack_);
  stack_->addWidget(start_);
  stack_->addWidget(canvas_);
  setPersistentCentralWidget(stack_);
  BuildMenus();

  connect(start_, &StartScreen::OpenRequested, this,
          [this](const QString& dir, bool example) {
            if (example) {
              OpenExample(dir);
            } else {
              OpenFolder(dir);
            }
          });
  connect(start_, &StartScreen::CreateRequested, this, [this](const QString& name) {
    Guard([&] { App::Get().session().create_project(ToRust(name)); });
  });
  connect(start_, &StartScreen::BrowseRequested, this, [this] {
    const QString dir =
        QFileDialog::getExistingDirectory(this, QStringLiteral("Open project"));
    if (!dir.isEmpty()) OpenFolder(dir);
  });
  connect(EventHub::Instance(), &EventHub::ProjectChanged, this, [this] {
    if (App::Get().session().has_project()) {
      ShowEditor();
    } else {
      ShowStart();
    }
  });
  connect(EventHub::Instance(), &EventHub::DocumentChanged, this,
          &MainWindow::SyncTitle);
  connect(EventHub::Instance(), &EventHub::Saved, this, &MainWindow::SyncTitle);
  ShowStart();
}

void MainWindow::Guard(const std::function<void()>& action) {
  try {
    action();
  } catch (const rust::Error& error) {
    App::Get().ShowError(QString::fromUtf8(error.what()));
  }
}

void MainWindow::OpenExample(const QString& dir) {
  Guard([&] { App::Get().session().open_example(ToRust(dir)); });
}

void MainWindow::OpenFolder(const QString& dir) {
  Guard([&] { App::Get().session().open_project(ToRust(dir)); });
}

void MainWindow::Generate(int count) {
  Guard([&] {
    App::Get().session().generate_project(static_cast<std::uint32_t>(count));
  });
}

void MainWindow::CloseProject() {
  SaveLayout();
  App::Get().session().close_project();
}

void MainWindow::BuildMenus() {
  QMenuBar* bar = app_bar_->menu_bar();
  QMenu* file = bar->addMenu(QStringLiteral("File"));
  file->addAction(QStringLiteral("New project…"), QKeySequence::New, this,
                  [this] { emit start_->CreateRequested(QStringLiteral("Untitled")); });
  file->addAction(QStringLiteral("Open project…"), QKeySequence::Open, this,
                  [this] { emit start_->BrowseRequested(); });
  file->addAction(QStringLiteral("Save"), QKeySequence::Save, this,
                  [this] { Guard([] { App::Get().session().save(); }); });
  file->addAction(QStringLiteral("Close project"), QKeySequence(Qt::CTRL | Qt::SHIFT | Qt::Key_W),
                  this, &MainWindow::CloseProject);
  file->addSeparator();
  file->addAction(QStringLiteral("Settings"), QKeySequence(Qt::CTRL | Qt::Key_Comma),
                  this, [this] { OpenSettings(); });
  file->addSeparator();
  file->addAction(QStringLiteral("Quit"), QKeySequence::Quit, this, &QWidget::close);

  QMenu* edit = bar->addMenu(QStringLiteral("Edit"));
  undo_ = edit->addAction(QStringLiteral("Undo"), QKeySequence::Undo, this,
                          [] { App::Get().session().undo(); });
  redo_ = edit->addAction(QStringLiteral("Redo"), QKeySequence(Qt::CTRL | Qt::SHIFT | Qt::Key_Z),
                          this, [] { App::Get().session().redo(); });

  QMenu* view = bar->addMenu(QStringLiteral("View"));
  view->addAction(QStringLiteral("Nodes"), QKeySequence(Qt::CTRL | Qt::Key_B), this,
                  [this] { ToggleDock(nodes_dock_); });
  view->addAction(QStringLiteral("Inspector"),
                  QKeySequence(Qt::CTRL | Qt::ALT | Qt::Key_B), this,
                  [this] { ToggleDock(inspector_dock_); });
  view->addAction(QStringLiteral("Bottom panel"), QKeySequence(Qt::CTRL | Qt::Key_J),
                  this, [this] { ToggleDock(timeline_dock_); });
  view->addSeparator();
  view->addAction(QStringLiteral("Fit to view"), QKeySequence(Qt::SHIFT | Qt::Key_1),
                  this, [this] { canvas_->FitAll(); });
  view->addAction(QStringLiteral("Minimap"), this, [this] {
    canvas_->SetMinimapVisible(!canvas_->minimap_visible());
  });

  QMenu* help = bar->addMenu(QStringLiteral("Help"));
  help->addAction(QStringLiteral("About Opsis"), this, [this] {
    SettingsWindow* window = OpenSettings();
    window->ShowPage(window->page_count() - 1);
  });
}

kdw::QtWidgets::DockWidget* MainWindow::MakeDock(const QString& id,
                                                 const QString& title,
                                                 QWidget* content) {
  auto* dock = new kdw::QtWidgets::DockWidget(id);
  dock->setTitle(title);
  dock->setWidget(content);
  return dock;
}

void MainWindow::BuildDocks() {
  auto* nodes = new NodeList();
  connect(nodes, &NodeList::AddRequested, this, [this](const QString& type) {
    const QPointF at =
        canvas_->mapToScene(canvas_->viewport()->rect().center());
    Guard([&] {
      App::Get().session().add_node(ToRust(type), at.x() - 128, at.y() - 22);
    });
  });
  nodes_dock_ = MakeDock(QStringLiteral("nodes"), QStringLiteral("Nodes"), nodes);
  inspector_dock_ = MakeDock(QStringLiteral("inspector"),
                             QStringLiteral("Inspector"), new InspectorPanel());
  timeline_dock_ = MakeDock(
      QStringLiteral("timeline"), QStringLiteral("Timeline"),
      Placeholder(QStringLiteral("The timeline isn't part of this spike.")));
  mixer_dock_ = MakeDock(
      QStringLiteral("mixer"), QStringLiteral("Mixer"),
      Placeholder(QStringLiteral("The mixer isn't part of this spike.")));
  run_dock_ = MakeDock(QStringLiteral("run"), QStringLiteral("Run"),
                       Placeholder(QStringLiteral("Runs aren't part of this spike.")));
  addDockWidget(timeline_dock_, kdw::Location_OnBottom, nullptr,
                kdw::InitialOption(kdw::Size(0, kBottomHeight)));
  addDockWidget(nodes_dock_, kdw::Location_OnLeft, nullptr,
                kdw::InitialOption(kdw::Size(kSideWidth, 0)));
  addDockWidget(inspector_dock_, kdw::Location_OnRight, nullptr,
                kdw::InitialOption(kdw::Size(kInspectorWidth, 0)));
  timeline_dock_->addDockWidgetAsTab(mixer_dock_);
  timeline_dock_->addDockWidgetAsTab(run_dock_);
  timeline_dock_->setAsCurrentTab();
  docked_ = true;
  const QString layout = ToQ(App::Get().session().load_layout());
  if (!layout.isEmpty()) {
    kdw::LayoutSaver saver;
    if (!saver.restoreLayout(layout.toUtf8())) {
      App::Get().Log(QStringLiteral("the saved layout didn't restore"));
    }
  }
}

void MainWindow::ShowEditor() {
  stack_->setCurrentWidget(canvas_);
  if (!docked_) {
    BuildDocks();
  } else {
    for (auto* dock : {nodes_dock_, inspector_dock_, timeline_dock_}) {
      if (!dock->isOpen()) dock->open();
    }
  }
  canvas_->Sync();
  canvas_->FitAll();
  SyncTitle();
}

void MainWindow::ShowStart() {
  if (docked_) {
    SaveLayout();
    for (auto* dock :
         {nodes_dock_, inspector_dock_, timeline_dock_, mixer_dock_, run_dock_}) {
      dock->forceClose();
    }
  }
  start_->Reload();
  stack_->setCurrentWidget(start_);
  app_bar_->SetSubtitle(QString());
  undo_->setEnabled(false);
  redo_->setEnabled(false);
}

void MainWindow::SaveLayout() {
  if (!docked_ || !App::Get().session().has_project()) return;
  kdw::LayoutSaver saver;
  const QByteArray layout = saver.serializeLayout();
  App::Get().session().save_layout(ToRust(QString::fromUtf8(layout)));
}

void MainWindow::SyncTitle() {
  const bridge::GraphView graph = App::Get().session().graph();
  if (graph.name.empty()) return;
  app_bar_->SetSubtitle(ToQ(graph.name) + QStringLiteral("  ·  ") +
                        (graph.dirty ? QStringLiteral("Unsaved changes")
                                     : QStringLiteral("Saved")));
  undo_->setEnabled(graph.can_undo);
  redo_->setEnabled(graph.can_redo);
}

void MainWindow::ToggleDock(kdw::QtWidgets::DockWidget* dock) {
  if (dock == nullptr) return;
  if (dock->isOpen()) {
    dock->forceClose();
  } else {
    dock->open();
  }
}

SettingsWindow* MainWindow::OpenSettings() {
  if (settings_ == nullptr) {
    settings_ = new SettingsWindow();
    settings_->setAttribute(Qt::WA_DeleteOnClose);
  }
  settings_->show();
  settings_->raise();
  settings_->activateWindow();
  return settings_;
}

void MainWindow::closeEvent(QCloseEvent* event) {
  SaveLayout();
  Guard([] { App::Get().session().save(); });
  if (settings_ != nullptr) settings_->close();
  event->accept();
  QApplication::quit();
}

}
