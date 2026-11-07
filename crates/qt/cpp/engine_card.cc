#include "engine_card.h"

#include <QHBoxLayout>
#include <QJsonDocument>
#include <QJsonObject>
#include <QLabel>
#include <QProgressBar>
#include <QPushButton>
#include <QScrollArea>
#include <QStyle>
#include <QVBoxLayout>
#include <cmath>
#include <utility>

#include "app.h"
#include "event_hub.h"
#include "opsis-qt/src/bridge.rs.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr int kSetupWidth = 576;

}

EngineCard::EngineCard(const QString& title, std::function<void()> on_done,
                       QWidget* parent)
    : QWidget(parent),
      on_done_(std::move(on_done)),
      state_(MakeLabel(QString(), "badge")),
      browser_line_(MakeLabel(QString(), "muted")),
      browser_warning_(MakeLabel(QString(), "muted")),
      choose_(MakeButton(QStringLiteral("Choose file…"), "outline",
                         QStringLiteral("file-search"))),
      download_(MakeButton(QStringLiteral("Download engine"), "primary",
                           QStringLiteral("download"))),
      progress_(new QProgressBar()),
      progress_text_(MakeLabel(QString(), "muted")) {
  setObjectName(QStringLiteral("EngineCard"));
  auto* layout = new QVBoxLayout(this);
  layout->setContentsMargins(0, 0, 0, 0);
  auto* card = new Card(
      title,
      QStringLiteral("The recording browser Opsis uses to record takes and "
                     "draw exports. Download it, or choose one already on "
                     "this computer; Opsis checks it before using it. Editing "
                     "and preview work without it."),
      this);
  layout->addWidget(card);
  state_->setObjectName(QStringLiteral("EngineState"));
  auto* badge = new QHBoxLayout();
  badge->addWidget(state_);
  badge->addStretch(1);
  card->body()->addLayout(badge);
  auto* browser = new QWidget(card);
  auto* row = new QHBoxLayout(browser);
  row->setContentsMargins(0, 0, 0, 0);
  row->setSpacing(12);
  auto* words = new QVBoxLayout();
  words->setSpacing(2);
  words->addWidget(MakeLabel(QStringLiteral("Recording browser"), "title-small"));
  browser_line_->setWordWrap(true);
  browser_line_->setTextInteractionFlags(Qt::TextSelectableByMouse);
  words->addWidget(browser_line_);
  browser_warning_->setWordWrap(true);
  words->addWidget(browser_warning_);
  row->addLayout(words, 1);
  connect(choose_, &QPushButton::clicked, this,
          [] { App::Get().session().choose_browser(); });
  row->addWidget(choose_, 0, Qt::AlignTop);
  card->body()->addWidget(browser);
  progress_->setRange(0, 100);
  card->body()->addWidget(progress_);
  progress_text_->setWordWrap(true);
  card->body()->addWidget(progress_text_);
  auto* buttons = new QHBoxLayout();
  buttons->setSpacing(8);
  connect(download_, &QPushButton::clicked, this,
          [] { App::Get().session().install_engine(); });
  buttons->addWidget(download_);
  if (on_done_) {
    done_ = MakeButton(QString(), "ghost", QString());
    connect(done_, &QPushButton::clicked, this, [this] { on_done_(); });
    buttons->addWidget(done_);
  }
  buttons->addStretch(1);
  card->body()->addLayout(buttons);
  connect(EventHub::Instance(), &EventHub::EngineChanged, this, &EngineCard::Load);
  Load();
}

void EngineCard::Load() {
  const bridge::EngineView view = App::Get().session().engine_view();
  state_->setText(view.complete ? QStringLiteral("Set up")
                                : QStringLiteral("Not set up"));
  QString line;
  if (view.checking) {
    line = QStringLiteral("Checking…");
  } else if (!view.problem.empty()) {
    line = ToQ(view.problem);
  } else if (view.path.empty()) {
    line = QStringLiteral("Not set up");
  } else {
    line = (view.version.empty()
                ? QString()
                : QStringLiteral("Version %1 · ").arg(ToQ(view.version))) +
           ToQ(view.path);
  }
  if (!view.browser_error.empty()) {
    line += QStringLiteral("\n") + ToQ(view.browser_error);
  }
  browser_line_->setText(line);
  browser_line_->setProperty(
      "role", view.problem.empty() && view.browser_error.empty() ? "muted"
                                                                 : "error");
  browser_line_->style()->unpolish(browser_line_);
  browser_line_->style()->polish(browser_line_);
  browser_warning_->setVisible(!view.warning.empty());
  browser_warning_->setText(ToQ(view.warning));
  choose_->setEnabled(!view.installing && !view.choosing && !view.checking);
  download_->setEnabled(!view.installing);
  download_->setText(view.complete ? QStringLiteral("Download again")
                                   : QStringLiteral("Download engine"));
  download_->setProperty("variant", view.complete ? "outline" : "primary");
  download_->style()->unpolish(download_);
  download_->style()->polish(download_);
  if (done_ != nullptr) {
    done_->setText(view.complete ? QStringLiteral("Continue")
                                 : QStringLiteral("Skip for now"));
  }
  const bool busy = view.installing || !view.error.empty();
  progress_->setVisible(view.installing && view.error.empty() &&
                        view.progress >= 0);
  progress_->setValue(static_cast<int>(std::lround(view.progress * 100)));
  progress_text_->setVisible(busy);
  progress_text_->setText(!view.error.empty()
                              ? ToQ(view.error)
                              : (view.message.empty() ? QStringLiteral("Starting…")
                                                      : ToQ(view.message)));
}

SetupScreen::SetupScreen(QWidget* parent) : QWidget(parent) {
  setObjectName(QStringLiteral("SetupScreen"));
  setAttribute(Qt::WA_StyledBackground);
  auto* outer = new QVBoxLayout(this);
  outer->setContentsMargins(0, 0, 0, 0);
  auto* scroll = new QScrollArea(this);
  scroll->setWidgetResizable(true);
  scroll->setFrameShape(QFrame::NoFrame);
  scroll->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
  outer->addWidget(scroll);
  auto* page = new QWidget();
  auto* center = new QHBoxLayout(page);
  center->setContentsMargins(24, 40, 24, 40);
  auto* column = new QVBoxLayout();
  column->setSpacing(24);
  column->addStretch(1);
  auto* heading = new QVBoxLayout();
  heading->setSpacing(4);
  heading->addWidget(MakeLabel(QStringLiteral("Welcome to Opsis"), "heading", page));
  auto* about = MakeLabel(
      QStringLiteral("Recording and exporting need a recording browser. "
                     "Download it once, or choose your own."),
      "muted", page);
  about->setWordWrap(true);
  heading->addWidget(about);
  column->addLayout(heading);
  column->addWidget(new EngineCard(QStringLiteral("Engine"), [this] {
    QJsonObject app{{QStringLiteral("setupSkipped"), true}};
    try {
      App::Get().session().patch_settings(ToRust(QString::fromUtf8(
          QJsonDocument(QJsonObject{{QStringLiteral("app"), app}})
              .toJson(QJsonDocument::Compact))));
    } catch (const rust::Error& error) {
      App::Get().ShowError(QStringLiteral("Couldn't save that choice. ") +
                           QString::fromUtf8(error.what()));
    }
    emit Done();
  }, page));
  column->addStretch(1);
  auto* holder = new QWidget(page);
  holder->setMaximumWidth(kSetupWidth);
  holder->setLayout(column);
  center->addStretch(1);
  center->addWidget(holder, 10);
  center->addStretch(1);
  scroll->setWidget(page);
}

}
