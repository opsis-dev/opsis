#include "main_window.h"

#include <kddockwidgets/LayoutSaver.h>

#include <QApplication>
#include <QCheckBox>
#include <QCloseEvent>
#include <QDialog>
#include <QFileDialog>
#include <QGuiApplication>
#include <QMenu>
#include <QMenuBar>
#include <QScreen>
#include <QTimer>
#include <QVBoxLayout>
#include <functional>

#include "app.h"
#include "event_hub.h"
#include "graph_canvas.h"
#include "inspector_panel.h"
#include "new_project_dialog.h"
#include "node_list.h"
#include "opsis-qt/src/bridge.rs.h"
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
constexpr int kSaveWindowAfterMs = 400;

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
  setCenterWidgetMargins(QMargins());
  stack_ = new QStackedWidget(this);
  start_ = new StartScreen(stack_);
  canvas_ = new GraphCanvas(stack_);
  stack_->addWidget(start_);
  stack_->addWidget(canvas_);
  setPersistentCentralWidget(stack_);
  rail_ = new CategoryRail(this);
  rail_->hide();
  internalLayout()->insertWidget(0, rail_);
  connect(rail_, &CategoryRail::ShowList, this,
          [this] { FocusDock(nodes_dock_); });
  connect(rail_, &CategoryRail::AddRequested, this, &MainWindow::AddNode);
  save_window_.setSingleShot(true);
  save_window_.setInterval(kSaveWindowAfterMs);
  connect(&save_window_, &QTimer::timeout, this, &MainWindow::SaveWindow);
  BuildMenus();

  connect(start_, &StartScreen::OpenRequested, this, &MainWindow::OpenFolder);
  connect(start_, &StartScreen::NewProjectRequested, this,
          &MainWindow::NewProject);
  connect(start_, &StartScreen::SettingsRequested, this,
          [this] { OpenSettings(); });
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
  connect(EventHub::Instance(), &EventHub::CloseAsked, this,
          &MainWindow::ShowCloseDialog);
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
  SaveWindow();
  SaveLayout();
  App::Get().session().close_project();
}

void MainWindow::BuildMenus() {
  QMenuBar* bar = app_bar_->menu_bar();
  for (const auto& section : App::Get().session().menu()) {
    QMenu* menu = bar->addMenu(ToQ(section.label));
    for (const auto& item : section.items) {
      if (item.kind == bridge::MenuItemKind::Separator) {
        menu->addSeparator();
        continue;
      }
      const QString id = ToQ(item.id);
      QAction* action = menu->addAction(ToQ(item.label));
      if (!item.shortcut.empty()) {
        action->setShortcut(QKeySequence(ToQ(item.shortcut)));
      }
      const bool role = item.kind == bridge::MenuItemKind::Role;
      connect(action, &QAction::triggered, this, [this, id, role] {
        if (role) {
          RunRole(id);
        } else {
          Dispatch(id);
        }
      });
      actions_.insert(id, action);
    }
  }
}

void MainWindow::NewProject(const QString& template_id, bool fixed) {
  NewProjectDialog dialog(template_id, fixed, this);
  dialog.exec();
}

void MainWindow::Dispatch(const QString& id) {
  if (id == QStringLiteral("project:new")) {
    NewProject(QString(), false);
  } else if (id == QStringLiteral("project:open")) {
    emit start_->BrowseRequested();
  } else if (id == QStringLiteral("view:settings")) {
    OpenSettings();
  } else if (id == QStringLiteral("view:projects")) {
    CloseProject();
  } else if (id == QStringLiteral("view:toggle-left")) {
    ToggleDock(nodes_dock_);
  } else if (id == QStringLiteral("view:toggle-right")) {
    ToggleDock(inspector_dock_);
  } else if (id == QStringLiteral("view:toggle-bottom")) {
    ToggleDock(timeline_dock_);
  } else if (id == QStringLiteral("view:toggle-minimap")) {
    canvas_->SetMinimapVisible(!canvas_->minimap_visible());
  } else if (id == QStringLiteral("view:focus-timeline")) {
    FocusDock(timeline_dock_);
  } else if (id == QStringLiteral("view:focus-mixer")) {
    FocusDock(mixer_dock_);
  } else if (id == QStringLiteral("view:focus-run")) {
    FocusDock(run_dock_);
  } else if (id == QStringLiteral("edit:select-all")) {
    canvas_->SelectAll();
  } else if (id == QStringLiteral("edit:add-node")) {
    FocusDock(nodes_dock_);
  } else {
    try {
      App::Get().session().menu_command(ToRust(id));
    } catch (const rust::Error& error) {
      canvas_->Toast(QString::fromUtf8(error.what()), true);
    }
  }
}

void MainWindow::RunRole(const QString& id) {
  const QString role = id.mid(QStringLiteral("role:").size());
  QWidget* focus = QApplication::focusWidget();
  if (role == QStringLiteral("cut") || role == QStringLiteral("copy") ||
      role == QStringLiteral("paste") || role == QStringLiteral("selectAll")) {
    if (focus != nullptr) {
      QMetaObject::invokeMethod(focus, role.toLatin1().constData());
    }
  } else if (role == QStringLiteral("quit") ||
             role == QStringLiteral("close")) {
    close();
  } else if (role == QStringLiteral("minimize")) {
    showMinimized();
  } else if (role == QStringLiteral("toggleMaximize")) {
    if (isMaximized()) {
      showNormal();
    } else {
      showMaximized();
    }
  } else if (role == QStringLiteral("togglefullscreen")) {
    if (isFullScreen()) {
      showNormal();
    } else {
      showFullScreen();
    }
  }
}

void MainWindow::FocusDock(kdw::QtWidgets::DockWidget* dock) {
  if (dock == nullptr) return;
  if (!dock->isOpen()) dock->open();
  dock->setAsCurrentTab();
  dock->raise();
  dock->widget()->setFocus();
}

void MainWindow::ShowCloseDialog() {
  const bridge::CloseView asked = App::Get().session().close_view();
  QDialog dialog(this);
  dialog.setObjectName(QStringLiteral("CloseDialog"));
  dialog.setWindowTitle(QStringLiteral("Opsis"));
  dialog.setMinimumWidth(420);
  auto* layout = new QVBoxLayout(&dialog);
  layout->setContentsMargins(24, 22, 24, 20);
  layout->setSpacing(10);
  layout->addWidget(
      MakeLabel(asked.ask ? QStringLiteral("Are you sure you want to exit?")
                          : QStringLiteral("Some work hasn't finished yet"),
                "title", &dialog));
  const bool busy = !asked.busy.empty();
  auto* about = MakeLabel(
      busy ? QStringLiteral("Closing now stops it:")
           : QStringLiteral(
                 "Your project is saved. You can pick up where you left off."),
      "muted", &dialog);
  about->setWordWrap(true);
  layout->addWidget(about);
  for (const auto& item : asked.busy) {
    auto* line = MakeLabel(QStringLiteral("•  ") + ToQ(item), "muted", &dialog);
    line->setWordWrap(true);
    layout->addWidget(line);
  }
  QCheckBox* dont_ask = nullptr;
  if (asked.ask) {
    dont_ask = new QCheckBox(QStringLiteral("Don't ask again"), &dialog);
    layout->addWidget(dont_ask);
  }
  auto* buttons = new QHBoxLayout();
  buttons->addStretch(1);
  auto* cancel =
      MakeButton(QStringLiteral("Cancel"), nullptr, QString(), &dialog);
  auto* exit =
      MakeButton(busy ? QStringLiteral("Close anyway") : QStringLiteral("Exit"),
                 busy ? "destructive" : "primary", QString(), &dialog);
  buttons->addWidget(cancel);
  buttons->addWidget(exit);
  layout->addSpacing(8);
  layout->addLayout(buttons);
  connect(cancel, &QPushButton::clicked, &dialog, &QDialog::reject);
  connect(exit, &QPushButton::clicked, &dialog, &QDialog::accept);
  exit->setDefault(true);
  QTimer::singleShot(0, &dialog, [] { App::Get().session().close_shown(); });
  const bool closing = dialog.exec() == QDialog::Accepted;
  App::Get().session().decide_close(
      closing, closing && dont_ask != nullptr && dont_ask->isChecked());
}

void MainWindow::AddNode(const QString& type) {
  const QPointF at = canvas_->mapToScene(canvas_->viewport()->rect().center());
  Guard([&] {
    App::Get().session().add_node(ToRust(type), at.x() - 128, at.y() - 22);
  });
}

void MainWindow::SyncRail() {
  const bool editing = stack_->currentWidget() == canvas_;
  rail_->setVisible(editing && nodes_dock_ != nullptr &&
                    !nodes_dock_->isOpen());
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
  connect(nodes, &NodeList::AddRequested, this, &MainWindow::AddNode);
  nodes_dock_ =
      MakeDock(QStringLiteral("nodes"), QStringLiteral("Nodes"), nodes);
  connect(nodes_dock_, &kdw::QtWidgets::DockWidget::isOpenChanged, this,
          &MainWindow::SyncRail);
  inspector_dock_ = MakeDock(QStringLiteral("inspector"),
                             QStringLiteral("Inspector"), new InspectorPanel());
  timeline_dock_ = MakeDock(
      QStringLiteral("timeline"), QStringLiteral("Timeline"),
      Placeholder(
          QStringLiteral("The timeline isn't in the native editor yet.")));
  mixer_dock_ = MakeDock(
      QStringLiteral("mixer"), QStringLiteral("Mixer"),
      Placeholder(QStringLiteral("The mixer isn't in the native editor yet.")));
  run_dock_ = MakeDock(
      QStringLiteral("run"), QStringLiteral("Run"),
      Placeholder(QStringLiteral("Runs aren't in the native editor yet.")));
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
  ApplyProjectWindow();
  if (!docked_) {
    BuildDocks();
  } else {
    for (auto* dock : {nodes_dock_, inspector_dock_, timeline_dock_}) {
      if (!dock->isOpen()) dock->open();
    }
  }
  SyncRail();
  canvas_->Sync();
  canvas_->FitAll();
  SyncTitle();
}

void MainWindow::ShowStart() {
  if (docked_) {
    SaveLayout();
    for (auto* dock : {nodes_dock_, inspector_dock_, timeline_dock_,
                       mixer_dock_, run_dock_}) {
      dock->forceClose();
    }
  }
  start_->Reload();
  stack_->setCurrentWidget(start_);
  SyncRail();
  app_bar_->SetSubtitle(QString());
  SetEnabled(QStringLiteral("edit:undo"), false);
  SetEnabled(QStringLiteral("edit:redo"), false);
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
  SetEnabled(QStringLiteral("edit:undo"), graph.can_undo);
  SetEnabled(QStringLiteral("edit:redo"), graph.can_redo);
}

void MainWindow::SetEnabled(const QString& id, bool enabled) {
  if (QAction* action = actions_.value(id)) action->setEnabled(enabled);
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

void MainWindow::ApplyProjectWindow() {
  rust::Vec<bridge::ScreenArea> areas;
  for (QScreen* screen : QGuiApplication::screens()) {
    const QRect area = screen->availableGeometry();
    areas.push_back({area.x(), area.y(), area.width(), area.height()});
  }
  const bridge::WindowPlacement placement =
      App::Get().session().project_window(std::move(areas));
  if (isFullScreen()) return;
  if (placement.known) {
    if (isMaximized()) showNormal();
    QWidget::setGeometry(
        QRect(placement.x, placement.y, placement.width, placement.height));
  }
  if (placement.maximized) showMaximized();
}

void MainWindow::SaveWindow() {
  if (!App::Get().session().has_project() || isMinimized() || isFullScreen()) {
    return;
  }
  const QRect bounds = isMaximized() ? normalGeometry() : geometry();
  App::Get().session().save_window({true, bounds.x(), bounds.y(),
                                    bounds.width(), bounds.height(),
                                    isMaximized()});
}

void MainWindow::moveEvent(QMoveEvent* event) {
  kdw::QtWidgets::MainWindow::moveEvent(event);
  save_window_.start();
}

void MainWindow::resizeEvent(QResizeEvent* event) {
  kdw::QtWidgets::MainWindow::resizeEvent(event);
  save_window_.start();
}

void MainWindow::closeEvent(QCloseEvent* event) {
  SaveWindow();
  if (App::Get().session().close_allowed()) {
    if (settings_ != nullptr) settings_->close();
    event->accept();
    return;
  }
  SaveLayout();
  event->ignore();
  App::Get().session().request_close();
}

}
