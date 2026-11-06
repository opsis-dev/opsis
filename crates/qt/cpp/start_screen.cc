#include "start_screen.h"

#include <QHBoxLayout>
#include <QInputDialog>
#include <QLabel>
#include <QVBoxLayout>
#include <algorithm>

#include "app.h"
#include "opsis-qt/src/bridge.rs.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr int kColumnWidth = 360;
constexpr int kDirRole = Qt::UserRole + 1;

QListWidget* MakeProjectList(QWidget* parent) {
  auto* list = new QListWidget(parent);
  list->setIconSize(QSize(18, 18));
  list->setSpacing(1);
  list->setVerticalScrollMode(QAbstractItemView::ScrollPerPixel);
  list->setCursor(Qt::PointingHandCursor);
  return list;
}

}

StartScreen::StartScreen(QWidget* parent) : QWidget(parent) {
  setObjectName(QStringLiteral("StartScreen"));
  setAttribute(Qt::WA_StyledBackground);
  auto* outer = new QHBoxLayout(this);
  outer->addStretch(1);
  auto* column = new QVBoxLayout();
  column->setSpacing(10);
  column->addStretch(1);
  auto* logo = new QLabel(this);
  logo->setPixmap(App::Get().Logo(56));
  column->addWidget(logo, 0, Qt::AlignLeft);
  column->addSpacing(8);
  column->addWidget(MakeLabel(QStringLiteral("Opsis"), "heading", this));
  auto* tagline = MakeLabel(
      QStringLiteral("Record browser and terminal takes, then turn them into "
                     "finished videos."),
      "muted", this);
  tagline->setWordWrap(true);
  column->addWidget(tagline);
  column->addSpacing(12);
  auto* actions = new QHBoxLayout();
  actions->setSpacing(8);
  auto* create = MakeButton(QStringLiteral("New project"), "primary",
                            QStringLiteral("file-plus"), this);
  auto* open = MakeButton(QStringLiteral("Open project"), nullptr,
                          QStringLiteral("folder-open"), this);
  actions->addWidget(create);
  actions->addWidget(open);
  actions->addStretch(1);
  column->addLayout(actions);
  column->addSpacing(16);
  column->addWidget(MakeLabel(QStringLiteral("RECENT"), "section", this));
  recent_ = MakeProjectList(this);
  column->addWidget(recent_);
  column->addSpacing(8);
  column->addWidget(MakeLabel(QStringLiteral("EXAMPLES"), "section", this));
  examples_ = MakeProjectList(this);
  examples_->setMaximumHeight(230);
  column->addWidget(examples_);
  column->addStretch(2);
  auto* holder = new QWidget(this);
  holder->setFixedWidth(kColumnWidth);
  holder->setLayout(column);
  outer->addWidget(holder);
  outer->addStretch(1);

  connect(create, &QPushButton::clicked, this, [this] {
    bool ok = false;
    const QString name = QInputDialog::getText(
        this, QStringLiteral("New project"), QStringLiteral("Name"),
        QLineEdit::Normal, QStringLiteral("Untitled"), &ok);
    if (ok) emit CreateRequested(name);
  });
  connect(open, &QPushButton::clicked, this, &StartScreen::BrowseRequested);
  connect(recent_, &QListWidget::itemClicked, this,
          [this](QListWidgetItem* item) {
            emit OpenRequested(item->data(kDirRole).toString(), false);
          });
  connect(examples_, &QListWidget::itemClicked, this,
          [this](QListWidgetItem* item) {
            emit OpenRequested(item->data(kDirRole).toString(), true);
          });
  Reload();
}

void StartScreen::Reload() {
  App& app = App::Get();
  const QIcon folder = app.Icon(QStringLiteral("folder-open"),
                                app.Token("muted-foreground"), 18);
  const QIcon example =
      app.Icon(QStringLiteral("play"), app.Token("muted-foreground"), 18);
  recent_->clear();
  recent_->setFixedHeight(0);
  for (const auto& entry : app.session().recent()) {
    auto* item = new QListWidgetItem(folder, ToQ(entry.name), recent_);
    item->setData(kDirRole, ToQ(entry.dir));
    item->setToolTip(ToQ(entry.detail));
  }
  if (recent_->count() == 0) {
    auto* item =
        new QListWidgetItem(QStringLiteral("No recent projects"), recent_);
    item->setFlags(Qt::NoItemFlags);
  }
  recent_->setFixedHeight(std::min(170, recent_->count() * 34 + 6));
  examples_->clear();
  for (const auto& entry : app.session().examples()) {
    auto* item = new QListWidgetItem(
        example, ToQ(entry.name) + QStringLiteral("  ·  ") + ToQ(entry.detail),
        examples_);
    item->setData(kDirRole, ToQ(entry.dir));
  }
}

}
