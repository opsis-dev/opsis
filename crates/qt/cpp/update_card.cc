#include "update_card.h"

#include <QDesktopServices>
#include <QHBoxLayout>
#include <QJsonDocument>
#include <QJsonObject>
#include <QLabel>
#include <QProgressBar>
#include <QPushButton>
#include <QStyle>
#include <QUrl>
#include <QVBoxLayout>
#include <cmath>

#include "app.h"
#include "event_hub.h"
#include "opsis-qt/src/bridge.rs.h"
#include "widgets.h"

namespace opsis {

namespace {

QString StatusText(const bridge::UpdateView& view) {
  if (!view.installed) {
    return QStringLiteral(
        "This copy of Opsis wasn't installed from a release, so it doesn't "
        "check for updates.");
  }
  if (!view.error.empty()) return ToQ(view.error);
  if (view.checking) return QStringLiteral("Checking for updates…");
  if (view.downloaded) {
    return QStringLiteral(
               "Version %1 is ready. It installs when Opsis closes, or "
               "now with Restart to update.")
        .arg(ToQ(view.available));
  }
  if (view.downloading) {
    return QStringLiteral("Downloading version %1…").arg(ToQ(view.available));
  }
  if (!view.available.empty()) {
    return view.self_updating
               ? QStringLiteral("Version %1 is available.")
                     .arg(ToQ(view.available))
               : QStringLiteral(
                     "Version %1 is available. Install it the same way you "
                     "installed Opsis.")
                     .arg(ToQ(view.available));
  }
  if (view.checked) return QStringLiteral("Opsis is up to date.");
  return QStringLiteral("Opsis hasn't checked for updates yet.");
}

void SaveCheckOnStart(bool on) {
  QJsonObject updates{{QStringLiteral("checkOnStart"), on}};
  try {
    App::Get().session().patch_settings(ToRust(QString::fromUtf8(
        QJsonDocument(QJsonObject{{QStringLiteral("updates"), updates}})
            .toJson(QJsonDocument::Compact))));
  } catch (const rust::Error& error) {
    App::Get().ShowError(QString::fromUtf8(error.what()));
  }
}

}

UpdateCard::UpdateCard(QWidget* parent)
    : QWidget(parent),
      status_(MakeLabel(QString(), "muted")),
      notes_(new QLabel()),
      progress_(new QProgressBar()),
      check_(MakeButton(QStringLiteral("Check for updates"), "outline",
                        QStringLiteral("refresh-cw"))),
      download_(MakeButton(QStringLiteral("Download update"), "primary",
                           QStringLiteral("download"))),
      cancel_(MakeButton(QStringLiteral("Stop"), "ghost", QString())),
      restart_(MakeButton(QStringLiteral("Restart to update"), "primary",
                          QStringLiteral("rotate-cw"))),
      page_(MakeButton(QStringLiteral("Open the release page"), "outline",
                       QStringLiteral("external-link"))),
      on_start_(new ToggleSwitch()) {
  setObjectName(QStringLiteral("UpdateCard"));
  auto* layout = new QVBoxLayout(this);
  layout->setContentsMargins(0, 0, 0, 0);
  auto* card = new Card(
      QStringLiteral("Updates"),
      QStringLiteral("Opsis goes online for updates only when you check, or "
                     "when it starts if you turn that on. Every update is "
                     "checked for Opsis's signature before it's kept."),
      this);
  layout->addWidget(card);
  status_->setObjectName(QStringLiteral("UpdateStatus"));
  status_->setWordWrap(true);
  card->body()->addWidget(status_);
  notes_->setTextFormat(Qt::MarkdownText);
  notes_->setWordWrap(true);
  notes_->setOpenExternalLinks(true);
  notes_->setTextInteractionFlags(Qt::TextBrowserInteraction);
  card->body()->addWidget(notes_);
  progress_->setRange(0, 100);
  card->body()->addWidget(progress_);
  auto* buttons = new QHBoxLayout();
  buttons->setSpacing(8);
  for (QPushButton* button : {check_, download_, cancel_, restart_, page_}) {
    buttons->addWidget(button);
  }
  buttons->addStretch(1);
  card->body()->addLayout(buttons);
  connect(check_, &QPushButton::clicked, this,
          [] { App::Get().session().check_updates(); });
  connect(download_, &QPushButton::clicked, this,
          [] { App::Get().session().download_update(); });
  connect(cancel_, &QPushButton::clicked, this,
          [] { App::Get().session().cancel_update_download(); });
  connect(restart_, &QPushButton::clicked, this,
          [] { App::Get().session().restart_to_update(); });
  connect(page_, &QPushButton::clicked, this,
          [this] { QDesktopServices::openUrl(QUrl(page_url_)); });
  on_start_->setObjectName(QStringLiteral("UpdatesOnStart"));
  connect(on_start_, &ToggleSwitch::toggled, this, &SaveCheckOnStart);
  card->body()->addWidget(MakeSwitchField(
      QStringLiteral("Check for updates when Opsis starts"), on_start_));
  connect(EventHub::Instance(), &EventHub::UpdateChanged, this,
          &UpdateCard::Load);
  connect(EventHub::Instance(), &EventHub::SettingsChanged, this,
          &UpdateCard::Load);
  Load();
}

void UpdateCard::Load() {
  const bridge::UpdateView view = App::Get().session().update_view();
  status_->setText(StatusText(view));
  status_->setProperty("role", view.error.empty() ? "muted" : "error");
  status_->style()->unpolish(status_);
  status_->style()->polish(status_);
  const bool offered = !view.available.empty();
  const bool busy = view.checking || view.downloading;
  notes_->setVisible(offered && !view.notes.empty());
  notes_->setText(ToQ(view.notes));
  progress_->setVisible(view.downloading && view.progress >= 0);
  progress_->setValue(static_cast<int>(std::lround(view.progress * 100)));
  page_url_ = ToQ(view.page);
  check_->setVisible(view.installed && !view.downloaded);
  check_->setEnabled(!busy);
  download_->setVisible(offered && view.self_updating && !view.downloaded);
  download_->setEnabled(!busy);
  download_->setText(
      QStringLiteral("Download version %1").arg(ToQ(view.available)));
  cancel_->setVisible(view.downloading);
  restart_->setVisible(view.downloaded);
  page_->setVisible(offered && !view.self_updating);
  on_start_->setEnabled(view.installed);
  on_start_->SetCheckedQuietly(view.check_on_start);
}

}
