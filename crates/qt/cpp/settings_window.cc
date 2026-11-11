#include "settings_window.h"

#include <QFileDialog>
#include <QGridLayout>
#include <QHBoxLayout>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonValue>
#include <QLabel>
#include <QLocale>
#include <QPlainTextEdit>
#include <QScrollArea>
#include <QStyle>
#include <QTimer>
#include <QToolButton>
#include <QVBoxLayout>
#include <algorithm>
#include <cmath>

#include "app.h"
#include "engine_card.h"
#include "event_hub.h"
#include "fonts_page.h"
#include "opsis-qt/src/bridge.rs.h"
#include "update_card.h"
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

QString FromVersion(rust::Str text) {
  return QString::fromUtf8(text.data(), static_cast<qsizetype>(text.size()));
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
  auto* bar = new AppBar(this, true, false);
  bar->content()->addWidget(
      MakeLabel(QStringLiteral("Settings"), "title", bar));
  bar->content()->addStretch(1);
  connect(bar, &AppBar::ApiBadgeClicked, this, [this] { ShowPage(kApi); });
  layout->addWidget(bar);
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
      {QStringLiteral("General"), QStringLiteral("sliders-horizontal")},
      {QStringLiteral("Projects"), QStringLiteral("folder-kanban")},
      {QStringLiteral("Editor"), QStringLiteral("layout-grid")},
      {QStringLiteral("Engine"), QStringLiteral("cpu")},
      {QStringLiteral("Recording"), QStringLiteral("video")},
      {QStringLiteral("Fonts"), QStringLiteral("type")},
      {QStringLiteral("API"), QStringLiteral("plug")},
      {QStringLiteral("Licences"), QStringLiteral("scale")},
      {QStringLiteral("About"), QStringLiteral("info")},
  };
  for (const auto& [label, icon] : sections) {
    new QListWidgetItem(App::Get().Icon(icon, ink, 16), label, sidebar_);
  }
  pages_->addWidget(GeneralPage());
  pages_->addWidget(ProjectsPage());
  pages_->addWidget(EditorPage());
  pages_->addWidget(EnginePage());
  pages_->addWidget(RecordingPage());
  pages_->addWidget(FontsPage());
  pages_->addWidget(ApiPage());
  pages_->addWidget(LicencesPage());
  pages_->addWidget(AboutPage());
  connect(sidebar_, &QListWidget::currentRowChanged, pages_,
          &QStackedWidget::setCurrentIndex);
  sidebar_->setCurrentRow(0);
  connect(EventHub::Instance(), &EventHub::SettingsChanged, this,
          &SettingsWindow::Load);
  connect(EventHub::Instance(), &EventHub::ApiChanged, this,
          &SettingsWindow::Load);
  connect(EventHub::Instance(), &EventHub::FontsChanged, this, [this] {
    QWidget* old = pages_->widget(kLicences);
    QWidget* fresh = LicencesPage();
    const bool shown = pages_->currentIndex() == kLicences;
    pages_->insertWidget(kLicences, fresh);
    pages_->removeWidget(old);
    old->deleteLater();
    if (shown) pages_->setCurrentIndex(kLicences);
  });
  Load();
}

int SettingsWindow::page_count() const { return pages_->count(); }

void SettingsWindow::ShowPage(int index) { sidebar_->setCurrentRow(index); }

QWidget* SettingsWindow::Page(const QString& title, const QString& description,
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
  auto* about = MakeLabel(description, "muted");
  about->setWordWrap(true);
  column->addWidget(about);
  column->addSpacing(8);
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
  connect(
      encode_speed_, &Segmented::Changed, this, [this](const QString& value) {
        Patch(QStringLiteral("media"), QStringLiteral("encodeSpeed"), value);
      });
  working->body()->addWidget(MakeField(
      QStringLiteral("Speed vs file size"), Holder(encode_speed_),
      QStringLiteral("Faster makes bigger files. Files already made stay as "
                     "they are, and exports keep the Render node's own "
                     "setting.")));
  return Page(QStringLiteral("General"),
              QStringLiteral("What Opsis does when it opens and before it "
                             "closes."),
              {starting, working});
}

QWidget* SettingsWindow::ProjectsPage() {
  auto* folder = new Card(
      QStringLiteral("Default projects folder"),
      QStringLiteral("New projects are suggested here, and Open project "
                     "starts here. If you create a project somewhere else, "
                     "that place is suggested next time."));
  auto* row = new QWidget();
  auto* line = new QHBoxLayout(row);
  line->setContentsMargins(0, 0, 0, 0);
  line->setSpacing(8);
  default_dir_ = new QLineEdit();
  default_dir_->setAccessibleName(QStringLiteral("Folder"));
  auto* browse = MakeButton(QStringLiteral("Choose…"), "outline",
                            QStringLiteral("folder-open"));
  line->addWidget(default_dir_, 1);
  line->addWidget(browse);
  connect(default_dir_, &QLineEdit::editingFinished, this, [this] {
    if (!default_dir_->isModified()) return;
    default_dir_->setModified(false);
    const bridge::SettingsView view = App::Get().session().settings();
    const QString typed = default_dir_->text().trimmed();
    Patch(QStringLiteral("projects"), QStringLiteral("defaultDir"),
          typed == ToQ(view.default_dir_fallback) ? QString() : typed);
  });
  connect(browse, &QPushButton::clicked, this, [this] {
    const QString dir = QFileDialog::getExistingDirectory(
        this, QStringLiteral("Choose the default projects folder"),
        default_dir_->text());
    if (!dir.isEmpty()) {
      Patch(QStringLiteral("projects"), QStringLiteral("defaultDir"), dir);
    }
  });
  default_dir_note_ = MakeLabel(QString(), "muted");
  default_dir_note_->setWordWrap(true);
  auto* field = new QWidget();
  auto* stack = new QVBoxLayout(field);
  stack->setContentsMargins(0, 0, 0, 0);
  stack->setSpacing(6);
  stack->addWidget(row);
  stack->addWidget(default_dir_note_);
  folder->body()->addWidget(MakeField(QStringLiteral("Folder"), field));
  default_dir_reset_ = MakeButton(QStringLiteral("Use the default folder"),
                                  "outline", QStringLiteral("rotate-ccw"));
  connect(default_dir_reset_, &QPushButton::clicked, this, [this] {
    Patch(QStringLiteral("projects"), QStringLiteral("defaultDir"), QString());
  });
  folder->body()->addWidget(Holder(default_dir_reset_));
  return Page(QStringLiteral("Projects"),
              QStringLiteral("Where new projects go."), {folder});
}

QWidget* SettingsWindow::EditorPage() {
  auto* canvas = new Card(QStringLiteral("Canvas"), QString());
  canvas_mode_ = new Segmented();
  canvas_mode_->SetChoices(
      Choices(App::Get().session().settings().canvas_modes));
  connect(
      canvas_mode_, &Segmented::Changed, this, [this](const QString& value) {
        Patch(QStringLiteral("editor"), QStringLiteral("canvasMode"), value);
      });
  canvas->body()->addWidget(MakeField(
      QStringLiteral("Scrolling"), Holder(canvas_mode_),
      QStringLiteral("With a mouse, the wheel zooms. With a touchpad, two "
                     "fingers pan and a pinch zooms.")));
  return Page(QStringLiteral("Editor"),
              QStringLiteral("How the canvas follows your mouse or "
                             "touchpad."),
              {canvas});
}

QWidget* SettingsWindow::EnginePage() {
  return Page(QStringLiteral("Engine"),
              QStringLiteral("The recording browser."),
              {new EngineCard(QStringLiteral("Status"), {})});
}

QWidget* SettingsWindow::RecordingPage() {
  const bridge::SettingsView view = App::Get().session().settings();
  auto* traces = new Card(
      QStringLiteral("Traces"),
      QStringLiteral("A step-by-step record of every take, for finding out "
                     "what went wrong. Open it from the Trace tab in Replay. "
                     "Stored compressed inside the project."));
  trace_retention_ = new Segmented();
  trace_retention_->setObjectName(QStringLiteral("TraceRetention"));
  trace_retention_->SetChoices(Choices(view.trace_retentions));
  connect(trace_retention_, &Segmented::Changed, this,
          [this](const QString& value) {
            Patch(QStringLiteral("traces"), QStringLiteral("retention"), value);
          });
  traces->body()->addWidget(
      MakeField(QStringLiteral("Keep traces"), Holder(trace_retention_)));
  keep_last_ = new QLineEdit();
  keep_last_->setObjectName(QStringLiteral("KeepLast"));
  keep_last_->setFixedWidth(96);
  keep_last_->setAccessibleName(QStringLiteral("Runs to keep per node"));
  connect(keep_last_, &QLineEdit::editingFinished, this, [this] {
    if (!keep_last_->isModified()) return;
    keep_last_->setModified(false);
    bool ok = false;
    const double typed =
        QLocale::c().toDouble(keep_last_->text().trimmed(), &ok);
    if (!ok) {
      Load();
      return;
    }
    const bridge::SettingsView now = App::Get().session().settings();
    const int whole = static_cast<int>(
        std::lround(std::clamp(typed, static_cast<double>(now.keep_last_min),
                               static_cast<double>(now.keep_last_max))));
    Patch(QStringLiteral("traces"), QStringLiteral("keepLast"), whole);
  });
  keep_last_row_ =
      MakeField(QStringLiteral("Runs to keep per node"), Holder(keep_last_));
  traces->body()->addWidget(keep_last_row_);
  auto* live = new Card(QStringLiteral("Live view"),
                        QStringLiteral("What the recording browser is doing, "
                                       "shown in the editor while a run is "
                                       "going."));
  auto* fps = new QWidget();
  auto* fps_row = new QHBoxLayout(fps);
  fps_row->setContentsMargins(0, 0, 0, 0);
  live_fps_ = new QSlider(Qt::Horizontal);
  live_fps_->setRange(static_cast<int>(view.live_fps_min),
                      static_cast<int>(view.live_fps_max));
  live_fps_->setAccessibleName(QStringLiteral("Live view frame rate"));
  live_fps_value_ = MakeLabel(QString(), "muted");
  live_fps_value_->setFixedWidth(52);
  live_fps_value_->setAlignment(Qt::AlignRight | Qt::AlignVCenter);
  fps_row->addWidget(live_fps_, 1);
  fps_row->addWidget(live_fps_value_);
  connect(live_fps_, &QSlider::valueChanged, this, [this](int value) {
    live_fps_value_->setText(QStringLiteral("%1 fps").arg(value));
  });
  connect(live_fps_, &QSlider::sliderReleased, this, [this] {
    Patch(QStringLiteral("live"), QStringLiteral("fps"), live_fps_->value());
  });
  connect(live_fps_, &QSlider::actionTriggered, this, [this](int action) {
    if (action == QAbstractSlider::SliderMove) return;
    QTimer::singleShot(0, this, [this] {
      Patch(QStringLiteral("live"), QStringLiteral("fps"), live_fps_->value());
    });
  });
  live->body()->addWidget(MakeField(
      QStringLiteral("Live view frame rate"), fps,
      QStringLiteral("Only affects the live view; recordings always use the "
                     "Record node's frame rate.")));
  live_auto_open_ = new ToggleSwitch();
  live_auto_open_->setObjectName(QStringLiteral("LiveAutoOpen"));
  connect(live_auto_open_, &ToggleSwitch::toggled, this, [this](bool on) {
    Patch(QStringLiteral("live"), QStringLiteral("autoOpen"), on);
  });
  live->body()->addWidget(
      MakeSwitchField(QStringLiteral("Open the live view when a take starts"),
                      live_auto_open_));
  return Page(QStringLiteral("Recording"),
              QStringLiteral("Traces of each take and the live view while a "
                             "run is going."),
              {traces, live});
}

QWidget* SettingsWindow::FontsPage() {
  return Page(QStringLiteral("Fonts"),
              QStringLiteral("Your font library. Projects copy the fonts they "
                             "use."),
              {new FontsCard()});
}

QWidget* SettingsWindow::ApiPage() {
  auto* api = new Card(
      QStringLiteral("Local API"),
#if defined(Q_OS_WIN)
      QStringLiteral("Scripts and AI agents on this computer can read the open "
                     "project, change it and run it. It only listens on a "
                     "local pipe that only your user account can open, never "
                     "on the network. Edits show in the editor and can be "
                     "undone."));
#else
      QStringLiteral("Scripts and AI agents on this computer can read the open "
                     "project, change it and run it. It only listens on a "
                     "local socket that only your user account can open, "
                     "never on the network. Edits show in the editor and can "
                     "be undone."));
#endif
  api_state_ = MakeLabel(QString(), "badge");
  api->body()->addWidget(Holder(api_state_));
  api_enabled_ = new ToggleSwitch();
  api_enabled_->setObjectName(QStringLiteral("ApiEnabled"));
  connect(api_enabled_, &ToggleSwitch::toggled, this, [this](bool on) {
    Patch(QStringLiteral("api"), QStringLiteral("enabled"), on);
  });
  api->body()->addWidget(MakeSwitchField(
      QStringLiteral("Turn on the API"), api_enabled_,
      QStringLiteral("Stays on until you turn it off. The top bar shows API "
                     "while it's on.")));
  api_scripts_ = new ToggleSwitch();
  api_scripts_->setObjectName(QStringLiteral("ApiScripts"));
  connect(api_scripts_, &ToggleSwitch::toggled, this, [this](bool on) {
    Patch(QStringLiteral("api"), QStringLiteral("scripts"), on);
  });
  api_scripts_row_ = MakeSwitchField(
      QStringLiteral("Run Lua scripts"), api_scripts_,
      QStringLiteral("Lets the API run Lua scripts on the open project. A "
                     "script can change the project, run and export it, and "
                     "read and write files in the project folder, but nothing "
                     "outside it and never the internet."));
  api->body()->addWidget(api_scripts_row_);
  api_error_ = MakeLabel(QString(), "error");
  api_error_->setWordWrap(true);
  api->body()->addWidget(api_error_);
  api_address_ = MakeLabel(QString(), "kbd");
  api_address_->setWordWrap(true);
  api_address_->setTextInteractionFlags(Qt::TextSelectableByMouse);
  api->body()->addWidget(MakeField(QStringLiteral("Address"), api_address_));
  auto* usage = new Card(
      QStringLiteral("Using it"),
      QStringLiteral("Plain HTTP with JSON. The description of every endpoint "
                     "is at /openapi.json; node types are at /nodes and "
                     "/nodes/{type}. Runs started here open the websites in "
                     "the flow, just like pressing Run."));
  api_usage_ = usage->body();
  return Page(QStringLiteral("API"),
              QStringLiteral("Let scripts and AI agents on this computer read, "
                             "edit and run the open project."),
              {api, usage});
}

QWidget* SettingsWindow::LicencesPage() {
  const auto rows = [](Card* card, const rust::Vec<bridge::LicenceView>& list) {
    for (const auto& entry : list) {
      auto* row = new QWidget();
      auto* column = new QVBoxLayout(row);
      column->setContentsMargins(0, 0, 0, 0);
      column->setSpacing(4);
      auto* head = new QHBoxLayout();
      auto* words = new QVBoxLayout();
      words->setSpacing(0);
      auto* title = new QHBoxLayout();
      title->setSpacing(8);
      title->addWidget(MakeLabel(ToQ(entry.title), "title-small"));
      if (!entry.badge.empty()) {
        title->addWidget(MakeLabel(ToQ(entry.badge), "badge"));
      }
      title->addStretch(1);
      words->addLayout(title);
      words->addWidget(MakeLabel(ToQ(entry.summary), "muted"));
      head->addLayout(words, 1);
      const QString file = ToQ(entry.file);
      if (!file.isEmpty()) {
        auto* open = MakeButton(QStringLiteral("Open"), "outline",
                                QStringLiteral("external-link"));
        QObject::connect(open, &QPushButton::clicked, open, [file] {
          try {
            App::Get().session().open_licence_file(ToRust(file));
          } catch (const rust::Error& error) {
            App::Get().ShowError(QString::fromUtf8(error.what()));
          }
        });
        head->addWidget(open, 0, Qt::AlignTop);
        column->addLayout(head);
      } else {
        auto* text = new QPlainTextEdit(ToQ(entry.text));
        text->setReadOnly(true);
        text->setFixedHeight(220);
        QFont mono(QStringLiteral("JetBrains Mono"));
        mono.setPixelSize(11);
        text->setFont(mono);
        text->hide();
        auto* toggle = MakeIconButton(QStringLiteral("chevron-right"),
                                      QStringLiteral("Show the licence"));
        QObject::connect(toggle, &QToolButton::clicked, text, [text, toggle] {
          text->setVisible(!text->isVisible());
          toggle->setIcon(App::Get().Icon(
              text->isVisible() ? QStringLiteral("chevron-down")
                                : QStringLiteral("chevron-right"),
              App::Get().Token(QStringLiteral("muted-foreground"))));
        });
        head->addWidget(toggle, 0, Qt::AlignTop);
        column->addLayout(head);
        column->addWidget(text);
      }
      card->body()->addWidget(row);
    }
  };
  auto* software = new Card(
      QStringLiteral("Software"),
      QStringLiteral("Opsis and the programs it ships with or sets up. The "
                     "recording browser appears once the engine is set up."));
  rows(software, App::Get().session().licences());
  auto* fonts = new Card(QStringLiteral("Built-in fonts"),
                         QStringLiteral("Fonts that come with Opsis."));
  rows(fonts, App::Get().session().font_licences(false));
  QList<QWidget*> cards{software, fonts};
  const rust::Vec<bridge::LicenceView> library =
      App::Get().session().font_licences(true);
  if (!library.empty()) {
    auto* yours = new Card(QStringLiteral("Your fonts"),
                           QStringLiteral("Fonts in your library, with the "
                                          "licence they came with."));
    rows(yours, library);
    cards.append(yours);
  }
  return Page(QStringLiteral("Licences"),
              QStringLiteral("Licences of Opsis and everything it includes."),
              cards);
}

QWidget* SettingsWindow::AboutPage() {
  const bridge::AboutView about = App::Get().session().about();
  auto* card = new Card(QStringLiteral("Opsis ") + ToQ(about.app),
                        QStringLiteral("Everything runs on this computer."));
  auto* logo = new QLabel();
  logo->setPixmap(App::Get().Logo(64));
  card->body()->addWidget(logo);
  auto* grid = new QGridLayout();
  grid->setHorizontalSpacing(16);
  grid->setVerticalSpacing(8);
  QList<std::pair<QString, QString>> rows = {
      {QStringLiteral("App"),
       about.development ? ToQ(about.app) + QStringLiteral(" (development)")
                         : ToQ(about.app)},
      {QStringLiteral("Interface"), ToQ(about.shell)},
      {QStringLiteral("Docking"),
       QStringLiteral("KDDockWidgets ") + FromVersion(bridge::kddw_version())},
      {QStringLiteral("Platform"), ToQ(about.platform)},
      {QStringLiteral("Build target"), ToQ(about.target)},
  };
  if (!about.commit.empty()) {
    rows.append({QStringLiteral("Commit"), ToQ(about.commit)});
  }
  rows.append({QStringLiteral("Installed from"),
               about.installer.empty() ? QStringLiteral("Not installed")
                                       : ToQ(about.installer)});
  rows.append({QStringLiteral("Recording browser"), ToQ(about.browser)});
  rows.append({QStringLiteral("Engine downloads"), ToQ(about.downloads)});
  rows.append({QStringLiteral("App data"), ToQ(about.user_data)});
  QFont mono(QStringLiteral("JetBrains Mono"));
  mono.setPixelSize(11);
  int row = 0;
  for (const auto& [name, value] : rows) {
    grid->addWidget(MakeLabel(name, "muted"), row, 0, Qt::AlignTop);
    auto* text = MakeLabel(value);
    text->setFont(mono);
    text->setWordWrap(true);
    text->setTextInteractionFlags(Qt::TextSelectableByMouse);
    grid->addWidget(text, row, 1);
    row += 1;
  }
  grid->setColumnStretch(1, 1);
  card->body()->addLayout(grid);
  return Page(QStringLiteral("About"),
              QStringLiteral("Versions, folders and updates."),
              {card, new UpdateCard()});
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
  api_scripts_->SetCheckedQuietly(view.api_scripts);
  api_scripts_row_->setEnabled(view.api_enabled && view.api_scripts_available);
  api_scripts_row_->setToolTip(
      view.api_scripts_available
          ? QString()
          : QStringLiteral("Not available on Windows yet: the API's pipe "
                           "isn't limited to your user account."));
  const bool custom = !view.default_dir.empty();
  if (!default_dir_->hasFocus()) {
    default_dir_->setText(custom ? ToQ(view.default_dir)
                                 : ToQ(view.default_dir_fallback));
  }
  default_dir_note_->setText(
      custom ? QStringLiteral("Your own folder.")
             : QStringLiteral("The default, in your home folder. It's created "
                              "with your first project."));
  default_dir_reset_->setVisible(custom);
  trace_retention_->blockSignals(true);
  trace_retention_->SetValue(ToQ(view.trace_retention));
  trace_retention_->blockSignals(false);
  keep_last_row_->setVisible(ToQ(view.trace_retention) ==
                             QStringLiteral("last-n"));
  if (!keep_last_->hasFocus()) {
    keep_last_->setText(QString::number(view.keep_last));
  }
  if (!live_fps_->isSliderDown()) {
    live_fps_->blockSignals(true);
    live_fps_->setValue(static_cast<int>(view.live_fps));
    live_fps_->blockSignals(false);
  }
  live_fps_value_->setText(QStringLiteral("%1 fps").arg(view.live_fps));
  live_auto_open_->SetCheckedQuietly(view.live_auto_open);
  api_state_->setText(!view.api_enabled    ? QStringLiteral("Off")
                      : view.api_listening ? QStringLiteral("On")
                                           : QStringLiteral("Not running"));
  api_error_->setVisible(view.api_enabled && !view.api_error.empty());
  api_error_->setText(QStringLiteral("The API couldn't start. ") +
                      ToQ(view.api_error));
  api_address_->setText(ToQ(view.api_address));
  while (QLayoutItem* item = api_usage_->takeAt(0)) {
    delete item->widget();
    delete item;
  }
#if defined(Q_OS_WIN)
  auto* pipe = MakeLabel(
      QStringLiteral("On Windows the API is a named pipe. Use an HTTP client "
                     "that can connect to a named pipe, with the address "
                     "above."),
      "muted");
  pipe->setWordWrap(true);
  api_usage_->addWidget(pipe);
#else
  for (const QString& route :
       {QStringLiteral("/openapi.json"), QStringLiteral("/status"),
        QStringLiteral("/project")}) {
    auto* line =
        MakeLabel(QStringLiteral("curl --unix-socket '%1' http://localhost%2")
                      .arg(ToQ(view.api_address), route),
                  "kbd");
    line->setWordWrap(true);
    line->setTextInteractionFlags(Qt::TextSelectableByMouse);
    api_usage_->addWidget(line);
  }
#endif
}

void SettingsWindow::Patch(const QString& section, const QString& key,
                           const QJsonValue& value) {
  QJsonObject inner;
  inner.insert(key, value);
  QJsonObject patch;
  patch.insert(section, inner);
  try {
    App::Get().session().patch_settings(ToRust(QString::fromUtf8(
        QJsonDocument(patch).toJson(QJsonDocument::Compact))));
  } catch (const rust::Error& error) {
    App::Get().ShowError(QString::fromUtf8(error.what()));
  }
}

}
