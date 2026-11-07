#include "start_screen.h"

#include <QFontMetrics>
#include <QGridLayout>
#include <QHBoxLayout>
#include <QKeySequence>
#include <QLabel>
#include <QScrollArea>
#include <array>

#include "app.h"
#include "event_hub.h"
#include "opsis-qt/src/bridge.rs.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr int kMaxWidth = 1024;
constexpr int kTemplateIcon = 32;

QLabel* SectionTitle(const QString& text, QWidget* parent) {
  return MakeLabel(text.toUpper(), "section", parent);
}

QString ShortcutFor(const QString& id) {
  for (const auto& section : App::Get().session().menu()) {
    for (const auto& item : section.items) {
      if (ToQ(item.id) == id && !item.shortcut.empty()) {
        return QKeySequence(ToQ(item.shortcut))
            .toString(QKeySequence::NativeText);
      }
    }
  }
  return {};
}

QLabel* Passive(QLabel* label) {
  label->setAttribute(Qt::WA_TransparentForMouseEvents);
  return label;
}

class ElidedLabel : public QLabel {
 public:
  ElidedLabel(const QString& text, QWidget* parent)
      : QLabel(parent), full_(text) {
    setProperty("role", "muted");
    setMinimumWidth(40);
    setSizePolicy(QSizePolicy::Ignored, QSizePolicy::Preferred);
    setAttribute(Qt::WA_TransparentForMouseEvents);
    setToolTip(text);
  }

 protected:
  void resizeEvent(QResizeEvent* event) override {
    QLabel::resizeEvent(event);
    setText(fontMetrics().elidedText(full_, Qt::ElideLeft, width()));
  }

 private:
  QString full_;
};

}

RowButton::RowButton(QWidget* parent, const char* variant)
    : QPushButton(parent), row_(new QHBoxLayout(this)) {
  setProperty("variant", variant);
  setCursor(Qt::PointingHandCursor);
  row_->setContentsMargins(8, 6, 8, 6);
  row_->setSpacing(12);
  QSizePolicy policy(QSizePolicy::Preferred, QSizePolicy::Preferred);
  policy.setHeightForWidth(true);
  setSizePolicy(policy);
}

QSize RowButton::sizeHint() const { return row_->sizeHint(); }

QSize RowButton::minimumSizeHint() const { return row_->minimumSize(); }

bool RowButton::hasHeightForWidth() const { return row_->hasHeightForWidth(); }

int RowButton::heightForWidth(int width) const {
  return row_->totalHeightForWidth(width);
}

StartScreen::StartScreen(QWidget* parent) : QWidget(parent) {
  setObjectName(QStringLiteral("StartScreen"));
  setAttribute(Qt::WA_StyledBackground);
  auto* outer = new QVBoxLayout(this);
  outer->setContentsMargins(0, 0, 0, 0);
  auto* scroll = new QScrollArea(this);
  scroll->setWidgetResizable(true);
  scroll->setFrameShape(QFrame::NoFrame);
  outer->addWidget(scroll);
  auto* page = new QWidget();
  auto* center = new QHBoxLayout(page);
  center->setContentsMargins(40, 56, 40, 56);
  auto* holder = new QWidget(page);
  holder->setMaximumWidth(kMaxWidth);
  auto* grid = new QHBoxLayout(holder);
  grid->setContentsMargins(0, 0, 0, 0);
  grid->setSpacing(48);

  auto* left = new QVBoxLayout();
  left->setSpacing(40);
  left->addWidget(ActionList());
  auto* recent = new QWidget(holder);
  recent_ = new QVBoxLayout(recent);
  recent_->setContentsMargins(0, 0, 0, 0);
  recent_->setSpacing(2);
  left->addWidget(recent);
  left->addStretch(1);

  auto* right = new QVBoxLayout();
  right->setSpacing(40);
  engine_ = EngineAlert();
  right->addWidget(engine_);
  right->addWidget(TemplateGrid());
  right->addStretch(1);

  auto* left_column = new QWidget(holder);
  left_column->setLayout(left);
  auto* right_column = new QWidget(holder);
  right_column->setLayout(right);
  for (QWidget* column : {left_column, right_column}) {
    column->setSizePolicy(QSizePolicy::Ignored, QSizePolicy::Preferred);
  }
  grid->addWidget(left_column, 100);
  grid->addWidget(right_column, 115);
  holder->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Preferred);
  center->addStretch(1);
  center->addWidget(holder, 1000, Qt::AlignVCenter);
  center->addStretch(1);
  scroll->setWidget(page);
  Reload();
}

QWidget* StartScreen::ActionList() {
  auto* section = new QWidget(this);
  auto* layout = new QVBoxLayout(section);
  layout->setContentsMargins(0, 0, 0, 0);
  layout->setSpacing(2);
  layout->addWidget(SectionTitle(QStringLiteral("Start"), section));
  struct Action {
    QString icon;
    QString label;
    QString command;
  };
  const std::array<Action, 3> actions = {{
      {QStringLiteral("plus"), QStringLiteral("New project…"),
       QStringLiteral("project:new")},
      {QStringLiteral("folder-open"), QStringLiteral("Open project…"),
       QStringLiteral("project:open")},
      {QStringLiteral("settings"), QStringLiteral("Settings"),
       QStringLiteral("view:settings")},
  }};
  App& app = App::Get();
  for (const Action& action : actions) {
    auto* row = new RowButton(section);
    auto* icon = Passive(new QLabel(row));
    icon->setPixmap(app.Glyph(action.icon, app.Token("muted-foreground"), 16));
    row->row()->addWidget(icon);
    row->row()->addWidget(Passive(MakeLabel(action.label, nullptr, row)), 1);
    const QString keys = ShortcutFor(action.command);
    if (!keys.isEmpty())
      row->row()->addWidget(Passive(MakeLabel(keys, "kbd", row)));
    layout->addWidget(row);
    const QString command = action.command;
    connect(row, &QPushButton::clicked, this, [this, command] {
      if (command == QStringLiteral("project:new")) {
        emit NewProjectRequested(QString(), false);
      } else if (command == QStringLiteral("project:open")) {
        emit BrowseRequested();
      } else {
        emit SettingsRequested();
      }
    });
  }
  return section;
}

QWidget* StartScreen::EngineAlert() {
  auto* card = new QFrame(this);
  card->setProperty("role", "alert");
  auto* row = new QHBoxLayout(card);
  row->setContentsMargins(16, 14, 16, 16);
  row->setSpacing(12);
  auto* icon = new QLabel(card);
  icon->setPixmap(App::Get().Glyph(QStringLiteral("download"),
                                   App::Get().Token("foreground"), 16));
  row->addWidget(icon, 0, Qt::AlignTop);
  auto* texts = new QVBoxLayout();
  texts->setSpacing(6);
  texts->addWidget(
      MakeLabel(QStringLiteral("Set up the recording engine"), "title", card));
  auto* body = MakeLabel(
      QStringLiteral(
          "Editing works now. Recording and exporting need the engine."),
      "muted", card);
  body->setWordWrap(true);
  texts->addWidget(body);
  auto* open =
      MakeButton(QStringLiteral("Open Settings"), nullptr, QString(), card);
  connect(open, &QPushButton::clicked, this, &StartScreen::EngineRequested);
  connect(EventHub::Instance(), &EventHub::EngineChanged, card,
          [card] { card->setVisible(App::Get().session().engine_missing()); });
  auto* buttons = new QHBoxLayout();
  buttons->addWidget(open);
  buttons->addStretch(1);
  texts->addLayout(buttons);
  row->addLayout(texts, 1);
  return card;
}

QWidget* StartScreen::TemplateGrid() {
  auto* section = new QWidget(this);
  auto* layout = new QVBoxLayout(section);
  layout->setContentsMargins(0, 0, 0, 0);
  layout->setSpacing(8);
  layout->addWidget(
      SectionTitle(QStringLiteral("Start from a template"), section));
  auto* grid = new QGridLayout();
  grid->setSpacing(8);
  grid->setColumnStretch(0, 1);
  grid->setColumnStretch(1, 1);
  App& app = App::Get();
  int index = 0;
  for (const auto& entry : app.session().templates()) {
    auto* card = new RowButton(section, "template");
    QSizePolicy fill(QSizePolicy::Ignored, QSizePolicy::Preferred);
    fill.setHeightForWidth(true);
    card->setSizePolicy(fill);
    QHBoxLayout* row = card->row();
    row->setContentsMargins(12, 12, 12, 12);
    auto* icon = Passive(new QLabel(card));
    icon->setObjectName(QStringLiteral("TemplateIcon"));
    icon->setFixedSize(kTemplateIcon, kTemplateIcon);
    icon->setAlignment(Qt::AlignCenter);
    icon->setPixmap(
        app.Glyph(ToQ(entry.icon), app.Token("muted-foreground"), 16));
    row->addWidget(icon, 0, Qt::AlignTop);
    auto* texts = new QVBoxLayout();
    texts->setSpacing(2);
    auto* title = Passive(MakeLabel(ToQ(entry.title), nullptr, card));
    QFont font = title->font();
    font.setWeight(QFont::Medium);
    title->setFont(font);
    texts->addWidget(title);
    auto* about = Passive(new ClampedLabel(ToQ(entry.description), 2, card));
    about->setProperty("role", "small");
    texts->addWidget(about);
    texts->addStretch(1);
    row->addLayout(texts, 1);
    grid->addWidget(card, index / 2, index % 2);
    const QString id = ToQ(entry.id);
    connect(card, &QPushButton::clicked, this,
            [this, id] { emit NewProjectRequested(id, true); });
    ++index;
  }
  layout->addLayout(grid);
  return section;
}

void StartScreen::Reload() {
  while (QLayoutItem* item = recent_->takeAt(0)) {
    delete item->widget();
    delete item;
  }
  recent_->addWidget(SectionTitle(QStringLiteral("Recent"), this));
  const auto entries = App::Get().session().recent();
  if (entries.empty()) {
    auto* empty = MakeLabel(
        QStringLiteral("No projects yet. Start from a template to see a "
                       "recording turn into a video."),
        "muted", this);
    empty->setWordWrap(true);
    recent_->addWidget(empty);
  }
  for (const auto& entry : entries) {
    auto* row = new RowButton(this);
    row->setToolTip(ToQ(entry.dir));
    auto* name = Passive(MakeLabel(ToQ(entry.name), nullptr, row));
    QFont font = name->font();
    font.setWeight(QFont::Medium);
    name->setFont(font);
    row->row()->addWidget(name);
    row->row()->addWidget(new ElidedLabel(ToQ(entry.dir), row), 1);
    row->row()->addWidget(Passive(MakeLabel(ToQ(entry.detail), "small", row)));
    const QString dir = ToQ(entry.dir);
    connect(row, &QPushButton::clicked, this,
            [this, dir] { emit OpenRequested(dir); });
    recent_->addWidget(row);
  }
  engine_->setVisible(App::Get().session().engine_missing());
}

}
