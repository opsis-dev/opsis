#include "settings_window.h"

#include <QFileDialog>
#include <QHBoxLayout>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonValue>
#include <QLabel>
#include <QScrollArea>
#include <QVBoxLayout>

#include "app.h"
#include "event_hub.h"
#include "opsis-qt/src/bridge.rs.h"
#include "widgets.h"
#include "window_chrome.h"

namespace opsis {

namespace {

constexpr int kSidebarWidth = 200;
constexpr int kPageWidth = 640;

std::vector<std::pair<QString, QString>> Choices(
    const rust::Vec<bridge::Choice>& list) {
  std::vector<std::pair<QString, QString>> out;
  for (const auto& choice : list) {
    out.emplace_back(ToQ(choice.value), ToQ(choice.label));
  }
  return out;
}

QWidget* Holder(QWidget* inner) {
  auto* holder = new QWidget();
  auto* row = new QHBoxLayout(holder);
  row->setContentsMargins(0, 0, 0, 0);
  row->addWidget(inner);
  row->addStretch(1);
  return holder;
}

}

SettingsWindow::SettingsWindow(QWidget* parent) : QWidget(parent, Qt::Window) {
  setObjectName(QStringLiteral("SettingsWindow"));
  setAttribute(Qt::WA_StyledBackground);
  setWindowTitle(QStringLiteral("Settings"));
  resize(940, 680);
  new WindowChrome(this);
  auto* layout = new QVBoxLayout(this);
  layout->setContentsMargins(0, 0, 0, 0);
  layout->setSpacing(0);
  layout->addWidget(new AppBar(this, QStringLiteral("Settings"), true));
  auto* body = new QHBoxLayout();
  body->setSpacing(0);
  layout->addLayout(body, 1);

  auto* side = new QWidget(this);
  side->setObjectName(QStringLiteral("SettingsSidebar"));
  side->setAttribute(Qt::WA_StyledBackground);
  side->setFixedWidth(kSidebarWidth);
  auto* side_layout = new QVBoxLayout(side);
  side_layout->setContentsMargins(8, 12, 8, 12);
  sidebar_ = new QListWidget(side);
  sidebar_->setIconSize(QSize(16, 16));
  side_layout->addWidget(sidebar_);
  body->addWidget(side);

  pages_ = new QStackedWidget(this);
  body->addWidget(pages_, 1);
  const QColor ink = App::Get().Token("foreground");
  const std::vector<std::pair<QString, QString>> sections = {
      {QStringLiteral("General"), QStringLiteral("settings")},
      {QStringLiteral("Projects"), QStringLiteral("folder-open")},
      {QStringLiteral("Editor"), QStringLiteral("layout-grid")},
      {QStringLiteral("Local API"), QStringLiteral("code")},
      {QStringLiteral("About"), QStringLiteral("box")},
  };
  for (const auto& [label, icon] : sections) {
    new QListWidgetItem(App::Get().Icon(icon, ink, 16), label, sidebar_);
  }
  pages_->addWidget(GeneralPage());
  pages_->addWidget(ProjectsPage());
  pages_->addWidget(EditorPage());
  pages_->addWidget(ApiPage());
  pages_->addWidget(AboutPage());
  connect(sidebar_, &QListWidget::currentRowChanged, pages_,
          &QStackedWidget::setCurrentIndex);
  sidebar_->setCurrentRow(0);
  connect(EventHub::Instance(), &EventHub::SettingsChanged, this,
          &SettingsWindow::Load);
  Load();
}

int SettingsWindow::page_count() const { return pages_->count(); }

void SettingsWindow::ShowPage(int index) { sidebar_->setCurrentRow(index); }

QWidget* SettingsWindow::Page(const QString& title,
                              const QList<QWidget*>& cards) {
  auto* scroll = new QScrollArea();
  scroll->setWidgetResizable(true);
  scroll->setFrameShape(QFrame::NoFrame);
  auto* inner = new QWidget();
  auto* center = new QHBoxLayout(inner);
  center->setContentsMargins(32, 28, 32, 32);
  auto* column = new QVBoxLayout();
  column->setSpacing(16);
  column->addWidget(MakeLabel(title, "heading"));
  for (QWidget* card : cards) column->addWidget(card);
  column->addStretch(1);
  auto* holder = new QWidget();
  holder->setMaximumWidth(kPageWidth);
  holder->setLayout(column);
  center->addWidget(holder, 1);
  center->addStretch(0);
  scroll->setWidget(inner);
  return scroll;
}

QWidget* SettingsWindow::GeneralPage() {
  const bridge::SettingsView view = App::Get().session().settings();
  auto* starting = new Card(QStringLiteral("Starting and closing"), QString());
  on_start_ = new Segmented();
  on_start_->SetChoices(Choices(view.start_options));
  connect(on_start_, &Segmented::Changed, this, [this](const QString& value) {
    Patch(QStringLiteral("app"), QStringLiteral("onStart"), value);
  });
  starting->body()->addWidget(
      MakeField(QStringLiteral("When Opsis starts"), Holder(on_start_)));
  confirm_exit_ = new ToggleSwitch();
  connect(confirm_exit_, &ToggleSwitch::toggled, this, [this](bool on) {
    Patch(QStringLiteral("app"), QStringLiteral("confirmExit"), on);
  });
  starting->body()->addWidget(MakeSwitchField(
      QStringLiteral("Ask before closing"), confirm_exit_,
      QStringLiteral("When this is off, Opsis still asks if a run, an engine "
                     "setup or a save hasn't finished.")));
  auto* working = new Card(
      QStringLiteral("Working files"),
      QStringLiteral("The videos nodes such as Trim, Reverse, Speed, Convert "
                     "and Apply grade make along the way."));
  encode_speed_ = new Segmented();
  encode_speed_->SetChoices(Choices(view.encode_speeds));
  connect(encode_speed_, &Segmented::Changed, this, [this](const QString& value) {
    Patch(QStringLiteral("media"), QStringLiteral("encodeSpeed"), value);
  });
  working->body()->addWidget(MakeField(
      QStringLiteral("Speed vs file size"), Holder(encode_speed_),
      QStringLiteral("Faster makes bigger files. Files already made stay as "
                     "they are, and exports keep the Render node's own "
                     "setting.")));
  return Page(QStringLiteral("General"), {starting, working});
}

QWidget* SettingsWindow::ProjectsPage() {
  auto* folder = new Card(QStringLiteral("Projects folder"),
                          QStringLiteral("New projects go here unless you pick "
                                         "another folder."));
  auto* row = new QWidget();
  auto* line = new QHBoxLayout(row);
  line->setContentsMargins(0, 0, 0, 0);
  line->setSpacing(8);
  default_dir_ = new QLineEdit();
  auto* browse = MakeButton(QStringLiteral("Choose…"));
  line->addWidget(default_dir_, 1);
  line->addWidget(browse);
  connect(default_dir_, &QLineEdit::editingFinished, this, [this] {
    Patch(QStringLiteral("projects"), QStringLiteral("defaultDir"),
          default_dir_->text());
  });
  connect(browse, &QPushButton::clicked, this, [this] {
    const QString dir = QFileDialog::getExistingDirectory(
        this, QStringLiteral("Projects folder"), default_dir_->text());
    if (!dir.isEmpty()) {
      Patch(QStringLiteral("projects"), QStringLiteral("defaultDir"), dir);
    }
  });
  folder->body()->addWidget(MakeField(QStringLiteral("Folder"), row));
  auto* data = new Card(QStringLiteral("App data"),
                        QStringLiteral("Settings, recent projects and the "
                                       "engine live in this folder."));
  auto* where = MakeLabel(ToQ(App::Get().session().settings().data_dir), "muted");
  where->setTextInteractionFlags(Qt::TextSelectableByMouse);
  where->setFont(QFont(QStringLiteral("JetBrains Mono"), 11));
  data->body()->addWidget(where);
  return Page(QStringLiteral("Projects"), {folder, data});
}

QWidget* SettingsWindow::EditorPage() {
  auto* canvas = new Card(QStringLiteral("Canvas"), QString());
  canvas_mode_ = new Segmented();
  canvas_mode_->SetChoices(Choices(App::Get().session().settings().canvas_modes));
  connect(canvas_mode_, &Segmented::Changed, this, [this](const QString& value) {
    Patch(QStringLiteral("editor"), QStringLiteral("canvasMode"), value);
  });
  canvas->body()->addWidget(MakeField(
      QStringLiteral("Scrolling"), Holder(canvas_mode_),
      QStringLiteral("With a mouse, the wheel zooms. With a touchpad, two "
                     "fingers pan and a pinch zooms.")));
  return Page(QStringLiteral("Editor"), {canvas});
}

QWidget* SettingsWindow::ApiPage() {
  auto* api = new Card(QStringLiteral("Local API"),
                       QStringLiteral("Lets programs on this computer drive "
                                      "Opsis. Nothing outside it can reach "
                                      "the API."));
  api_enabled_ = new ToggleSwitch();
  connect(api_enabled_, &ToggleSwitch::toggled, this, [this](bool on) {
    Patch(QStringLiteral("api"), QStringLiteral("enabled"), on);
  });
  api->body()->addWidget(
      MakeSwitchField(QStringLiteral("Turn on the Local API"), api_enabled_));
  return Page(QStringLiteral("Local API"), {api});
}

QWidget* SettingsWindow::AboutPage() {
  auto* about = new Card(QStringLiteral("Opsis"),
                         QStringLiteral("The native editor, built with Qt "
                                        "6.12 Widgets and KDDockWidgets "
                                        "2.5.1."));
  auto* logo = new QLabel();
  logo->setPixmap(App::Get().Logo(64));
  about->body()->addWidget(logo);
  return Page(QStringLiteral("About"), {about});
}

void SettingsWindow::Load() {
  const bridge::SettingsView view = App::Get().session().settings();
  for (Segmented* segmented : {on_start_, encode_speed_, canvas_mode_}) {
    segmented->blockSignals(true);
  }
  on_start_->SetValue(ToQ(view.on_start));
  encode_speed_->SetValue(ToQ(view.encode_speed));
  canvas_mode_->SetValue(ToQ(view.canvas_mode));
  for (Segmented* segmented : {on_start_, encode_speed_, canvas_mode_}) {
    segmented->blockSignals(false);
  }
  confirm_exit_->SetCheckedQuietly(view.confirm_exit);
  api_enabled_->SetCheckedQuietly(view.api_enabled);
  if (!default_dir_->hasFocus()) default_dir_->setText(ToQ(view.default_dir));
}

void SettingsWindow::Patch(const QString& section, const QString& key,
                           const QJsonValue& value) {
  QJsonObject inner;
  inner.insert(key, value);
  QJsonObject patch;
  patch.insert(section, inner);
  try {
    App::Get().session().patch_settings(
        ToRust(QString::fromUtf8(QJsonDocument(patch).toJson(QJsonDocument::Compact))));
  } catch (const rust::Error& error) {
    App::Get().ShowError(QString::fromUtf8(error.what()));
  }
}

}
