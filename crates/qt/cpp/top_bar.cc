#include "top_bar.h"

#include <QHBoxLayout>
#include <QPainter>

#include "app.h"
#include "motion.h"
#include "opsis-qt/src/bridge.rs.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr int kSpinnerSize = 16;
constexpr int kSpinnerFrameMs = 33;
constexpr double kSpinnerStep = 12.0;
constexpr int kSpinnerSpan = 270 * 16;
constexpr int kNameMaxWidth = 280;
constexpr int kAddIcon = 16;
constexpr int kBadgeMaxHeight = 28;

QFrame* VerticalRule(QWidget* parent) {
  auto* rule = new QFrame(parent);
  rule->setProperty("role", "vrule");
  rule->setFixedSize(1, 20);
  return rule;
}

QString Tip(const QString& label, const QString& command) {
  const QString keys = ShortcutFor(command);
  return keys.isEmpty() ? label : label + QStringLiteral("  ") + keys;
}

}

QString ShortcutFor(const QString& command) {
  for (const auto& section : App::Get().session().menu()) {
    for (const auto& item : section.items) {
      if (ToQ(item.id) == command) return ToQ(item.shortcut);
    }
  }
  return {};
}

Spinner::Spinner(QWidget* parent) : QWidget(parent) {
  setFixedSize(kSpinnerSize, kSpinnerSize);
  timer_.setInterval(kSpinnerFrameMs);
  connect(&timer_, &QTimer::timeout, this, [this] {
    angle_ = std::fmod(angle_ + kSpinnerStep, 360.0);
    update();
  });
}

void Spinner::paintEvent(QPaintEvent*) {
  QPainter painter(this);
  painter.setRenderHint(QPainter::Antialiasing);
  painter.setPen(QPen(App::Get().Token("muted-foreground"), 2, Qt::SolidLine,
                      Qt::RoundCap));
  const QRectF box = QRectF(rect()).adjusted(2, 2, -2, -2);
  painter.drawArc(box, static_cast<int>(-angle_ * 16), kSpinnerSpan);
}

void Spinner::showEvent(QShowEvent*) {
  if (!ReducedMotion()) timer_.start();
}

void Spinner::hideEvent(QHideEvent*) { timer_.stop(); }

TopBar::TopBar(QWidget* parent) : QWidget(parent) {
  auto* layout = new QHBoxLayout(this);
  layout->setContentsMargins(0, 0, 0, 0);
  layout->setSpacing(4);
  layout->addWidget(Tool(QStringLiteral("arrow-left"),
                         QStringLiteral("All projects"),
                         QStringLiteral("view:projects")));
  name_ = new QLabel(this);
  name_->setProperty("role", "title");
  name_->setMaximumWidth(kNameMaxWidth);
  layout->addWidget(name_);
  saved_ = new QLabel(this);
  saved_->setProperty("role", "badge");
  layout->addWidget(saved_, 0, Qt::AlignVCenter);
  layout->addSpacing(4);
  layout->addWidget(VerticalRule(this));
  layout->addSpacing(4);
  undo_ = Tool(QStringLiteral("undo-2"), QStringLiteral("Undo"),
               QStringLiteral("edit:undo"));
  redo_ = Tool(QStringLiteral("redo-2"), QStringLiteral("Redo"),
               QStringLiteral("edit:redo"));
  layout->addWidget(undo_);
  layout->addWidget(redo_);
  layout->addWidget(Tool(QStringLiteral("layout-grid"),
                         QStringLiteral("Tidy up layout"),
                         QStringLiteral("edit:layout")));
  auto* add = new QPushButton(this);
  add->setProperty("variant", "ghost");
  add->setFocusPolicy(Qt::NoFocus);
  add->setCursor(Qt::PointingHandCursor);
  add->setAccessibleName(QStringLiteral("Add node"));
  auto* add_layout = new QHBoxLayout(add);
  add_layout->setContentsMargins(10, 0, 8, 0);
  add_layout->setSpacing(6);
  auto* plus = new QLabel(add);
  plus->setPixmap(App::Get().Glyph(QStringLiteral("plus"),
                                   App::Get().Token("foreground"), kAddIcon));
  auto* text = MakeLabel(QStringLiteral("Add node"), nullptr, add);
  auto* keys =
      MakeLabel(ShortcutFor(QStringLiteral("edit:add-node")), "kbd", add);
  for (QLabel* label : {plus, text, keys}) {
    label->setAttribute(Qt::WA_TransparentForMouseEvents);
    add_layout->addWidget(label);
  }
  add->setMinimumWidth(add_layout->sizeHint().width());
  connect(add, &QPushButton::clicked, this,
          [this] { emit Command(QStringLiteral("edit:add-node")); });
  layout->addWidget(add);
  layout->addStretch(1);
  left_ = PanelToggle(QStringLiteral("panel-left"), QStringLiteral("node list"),
                      QStringLiteral("view:toggle-left"));
  bottom_ = PanelToggle(QStringLiteral("panel-bottom"),
                        QStringLiteral("bottom panel"),
                        QStringLiteral("view:toggle-bottom"));
  right_ =
      PanelToggle(QStringLiteral("panel-right"), QStringLiteral("inspector"),
                  QStringLiteral("view:toggle-right"));
  layout->addWidget(left_);
  layout->addWidget(bottom_);
  layout->addWidget(right_);
  layout->addSpacing(4);
  layout->addWidget(VerticalRule(this));
  layout->addSpacing(4);
  auto* outputs = MakeButton(QStringLiteral("Outputs"), "ghost",
                             QStringLiteral("folder-open"), this);
  outputs->setFocusPolicy(Qt::NoFocus);
  connect(outputs, &QPushButton::clicked, this, [] {
    try {
      App::Get().session().reveal_outputs();
    } catch (const rust::Error& error) {
      App::Get().ShowError(QString::fromUtf8(error.what()));
    }
  });
  layout->addWidget(outputs);
  layout->addWidget(Tool(QStringLiteral("settings"), QStringLiteral("Settings"),
                         QStringLiteral("view:settings")));
  run_ = MakeButton(QStringLiteral("Run"), "primary", QStringLiteral("play"),
                    this);
  run_->setFocusPolicy(Qt::NoFocus);
  connect(run_, &QPushButton::clicked, this,
          [this] { emit Command(QStringLiteral("run:start")); });
  layout->addWidget(run_);
  running_ = new QWidget(this);
  auto* running_layout = new QHBoxLayout(running_);
  running_layout->setContentsMargins(0, 0, 0, 0);
  running_layout->setSpacing(6);
  running_layout->addWidget(new Spinner(running_));
  running_layout->addWidget(
      MakeLabel(QStringLiteral("Running…"), nullptr, running_));
  auto* stop = MakeButton(QStringLiteral("Stop"), nullptr,
                          QStringLiteral("square"), running_);
  stop->setFocusPolicy(Qt::NoFocus);
  connect(stop, &QPushButton::clicked, this,
          [this] { emit Command(QStringLiteral("run:cancel")); });
  running_layout->addWidget(stop);
  layout->addWidget(running_);
  running_->hide();
}

QToolButton* TopBar::Tool(const QString& icon, const QString& label,
                          const QString& command) {
  auto* button = MakeIconButton(icon, Tip(label, command), this);
  button->setFocusPolicy(Qt::NoFocus);
  connect(button, &QToolButton::clicked, this,
          [this, command] { emit Command(command); });
  return button;
}

QToolButton* TopBar::PanelToggle(const QString& icon, const QString& label,
                                 const QString& command) {
  auto* button = Tool(icon, label, command);
  button->setCheckable(true);
  button->setProperty("label", label);
  button->setProperty("command", command);
  return button;
}

void TopBar::SetPanels(bool left, bool bottom, bool right) {
  for (auto [button, open] :
       {std::pair{left_, left}, std::pair{bottom_, bottom},
        std::pair{right_, right}}) {
    button->setChecked(open);
    const QString label = button->property("label").toString();
    button->setToolTip(
        Tip((open ? QStringLiteral("Hide ") : QStringLiteral("Show ")) + label,
            button->property("command").toString()));
  }
}

QString TopBar::Problems() const {
  QStringList problems;
  if (!isVisible()) problems << QStringLiteral("hidden");
  if (name_->text().isEmpty()) problems << QStringLiteral("no project name");
  if (saved_->text() != QStringLiteral("Saved")) {
    problems << QStringLiteral("badge reads '%1'").arg(saved_->text());
  }
  if (!left_->isChecked() || !bottom_->isChecked() || !right_->isChecked()) {
    problems << QStringLiteral("a panel toggle is off");
  }
  if (!run_->isVisible()) problems << QStringLiteral("no Run button");
  if (saved_->height() > kBadgeMaxHeight) {
    problems << QStringLiteral("the saved badge is %1 px tall")
                    .arg(saved_->height());
  }
  for (QPushButton* button : findChildren<QPushButton*>()) {
    if (button->isVisible() && button->layout() != nullptr &&
        button->width() < button->layout()->sizeHint().width()) {
      problems
          << QStringLiteral("'%1' is cut off").arg(button->accessibleName());
    }
  }
  return problems.join(QStringLiteral("; "));
}

void TopBar::Sync() {
  App& app = App::Get();
  const bridge::GraphView graph = app.session().graph();
  name_->setText(name_->fontMetrics().elidedText(
      ToQ(graph.name), Qt::ElideRight, kNameMaxWidth));
  name_->setToolTip(ToQ(graph.name));
  saved_->setText(graph.dirty ? QStringLiteral("Unsaved")
                              : QStringLiteral("Saved"));
  undo_->setEnabled(graph.can_undo);
  redo_->setEnabled(graph.can_redo);
  const bool running = app.session().bar().running;
  run_->setVisible(!running);
  running_->setVisible(running);
}

}
