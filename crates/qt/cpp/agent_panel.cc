#include "agent_panel.h"

#include <QDateTime>
#include <QHBoxLayout>
#include <QJsonDocument>
#include <QJsonObject>
#include <QKeyEvent>
#include <QMenu>
#include <QMouseEvent>
#include <QPushButton>
#include <QScrollBar>
#include <QShowEvent>
#include <QStyle>
#include <QTextBlockFormat>
#include <QTextBrowser>
#include <QTextCursor>
#include <QTextTable>
#include <algorithm>
#include <cmath>
#include <functional>
#include <utility>

#include "app.h"
#include "event_hub.h"
#include "opsis-qt/src/bridge.rs.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr int kBarHeight = 40;
constexpr int kIconSlot = 20;
constexpr int kToolIcon = 13;
constexpr int kRowHeight = 20;
constexpr int kDetailIndent = 26;
constexpr int kDetailLines = 30;
constexpr int kAskLines = 14;
constexpr int kPromptLines = 6;
constexpr int kThreadHeight = 30;
constexpr int kSendSize = 26;
constexpr int kBubbleShare = 85;
constexpr int kBubblePadding = 24;
constexpr int kTickMs = 1000;
constexpr int kListIndent = 18;
const char* const kSignIn = "sign-in";

class ClickRow : public QWidget {
 public:
  explicit ClickRow(std::function<void()> clicked, QWidget* parent = nullptr)
      : QWidget(parent), clicked_(std::move(clicked)) {
    if (clicked_) setCursor(Qt::PointingHandCursor);
  }

 protected:
  void mouseReleaseEvent(QMouseEvent* event) override {
    if (clicked_ && event->button() == Qt::LeftButton &&
        rect().contains(event->position().toPoint())) {
      clicked_();
      return;
    }
    QWidget::mouseReleaseEvent(event);
  }

 private:
  std::function<void()> clicked_;
};

class BubbleRow : public QWidget {
 public:
  explicit BubbleRow(QLabel* bubble, QWidget* parent = nullptr)
      : QWidget(parent), bubble_(bubble) {
    auto* line = new QHBoxLayout(this);
    line->setContentsMargins(0, 0, 0, 0);
    line->addStretch(1);
    line->addWidget(bubble);
  }

 protected:
  void resizeEvent(QResizeEvent* event) override {
    QWidget::resizeEvent(event);
    const QMargins margins = bubble_->contentsMargins();
    const int padding = margins.left() + margins.right() + kBubblePadding;
    int widest = 0;
    for (const QString& line : bubble_->text().split(QLatin1Char('\n'))) {
      widest = std::max(widest, bubble_->fontMetrics().horizontalAdvance(line));
    }
    bubble_->setFixedWidth(
        std::min(width() * kBubbleShare / 100, widest + padding));
  }

 private:
  QLabel* bubble_;
};

QHBoxLayout* ButtonsOf(QFrame* note) {
  auto* body = note->layout()->itemAt(1)->layout();
  return static_cast<QHBoxLayout*>(body->itemAt(body->count() - 1)->layout());
}

class MarkdownView : public QTextBrowser {
 public:
  explicit MarkdownView(const QString& html, QWidget* parent = nullptr)
      : QTextBrowser(parent) {
    setObjectName(QStringLiteral("AgentSaid"));
    setFrameShape(QFrame::NoFrame);
    setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
    setVerticalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
    setOpenExternalLinks(true);
    setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Fixed);
    document()->setDocumentMargin(0);
    document()->setIndentWidth(kListIndent);
    document()->setDefaultStyleSheet(
        QStringLiteral("p { margin-top: 0px; margin-bottom: 10px; }"
                       "ul, ol { margin-top: 0px; margin-bottom: 8px; "
                       "margin-left: 0px; -qt-list-indent: 1; }"
                       "li { margin-bottom: 4px; }"
                       "blockquote { margin-left: 10px; }"
                       "a { color: %1; }")
            .arg(App::Get().Token(QStringLiteral("foreground")).name()));
    setHtml(html);
    QTextCursor end(document());
    end.movePosition(QTextCursor::End);
    if (QTextTable* table = end.currentTable()) {
      QTextTableFormat format = table->format();
      format.setBottomMargin(0);
      table->setFormat(format);
    } else {
      QTextBlockFormat format = end.blockFormat();
      format.setBottomMargin(0);
      end.setBlockFormat(format);
    }
  }

 protected:
  void resizeEvent(QResizeEvent* event) override {
    QTextBrowser::resizeEvent(event);
    document()->setTextWidth(viewport()->width());
    const int height = static_cast<int>(std::ceil(document()->size().height()));
    if (height != this->height()) setFixedHeight(height);
  }
};

void Repolish(QWidget* widget) {
  widget->style()->unpolish(widget);
  widget->style()->polish(widget);
}

QLabel* IconLabel(const QString& icon, const QString& token, int size,
                  QWidget* parent) {
  auto* label = new QLabel(parent);
  label->setPixmap(App::Get().Glyph(icon, App::Get().Token(token), size));
  label->setFixedSize(kIconSlot, kRowHeight);
  label->setAlignment(Qt::AlignCenter);
  label->setAttribute(Qt::WA_TransparentForMouseEvents);
  return label;
}

QString Clipped(const QString& text, int lines) {
  const QStringList rows = text.split(QLatin1Char('\n'));
  if (rows.size() <= lines) return text;
  return rows.mid(0, lines).join(QLatin1Char('\n')) +
         QStringLiteral("\n… %1 more lines").arg(rows.size() - lines);
}

QLabel* Mono(const QString& text, const char* name, int lines,
             QWidget* parent) {
  auto* label = new QLabel(Clipped(text, lines), parent);
  label->setObjectName(QString::fromLatin1(name));
  label->setTextFormat(Qt::PlainText);
  label->setWordWrap(true);
  label->setTextInteractionFlags(Qt::TextSelectableByMouse);
  return label;
}

QString ToolKey(const bridge::AgentToolView& tool) {
  return QStringList{ToQ(tool.id),
                     ToQ(tool.label),
                     ToQ(tool.icon),
                     tool.failed ? QStringLiteral("f") : QString(),
                     tool.running ? QStringLiteral("r") : QString(),
                     tool.expanded ? QStringLiteral("e") : QString(),
                     ToQ(tool.output)}
      .join(QChar(0x1f));
}

QString EntryKey(const bridge::AgentEntryView& entry) {
  QStringList parts{ToQ(entry.kind),
                    ToQ(entry.id),
                    ToQ(entry.text),
                    ToQ(entry.icon),
                    entry.failed ? QStringLiteral("f") : QString(),
                    entry.running ? QStringLiteral("r") : QString(),
                    entry.expanded ? QStringLiteral("e") : QString()};
  for (const auto& tool : entry.tools) parts << ToolKey(tool);
  return parts.join(QChar(0x1e));
}

QString Elapsed(qint64 since) {
  const qint64 seconds =
      std::max<qint64>(0, (QDateTime::currentMSecsSinceEpoch() - since) / 1000);
  if (seconds < 60) return QStringLiteral("%1s").arg(seconds);
  return QStringLiteral("%1m %2s")
      .arg(seconds / 60)
      .arg(seconds % 60, 2, 10, QLatin1Char('0'));
}

QPushButton* InlineButton(const QString& text, bool muted, QWidget* parent) {
  auto* button = MakeButton(text, "inline", QString(), parent);
  button->setProperty("tone", muted ? "muted" : "default");
  button->setCursor(Qt::PointingHandCursor);
  return button;
}

}

ElidedLabel::ElidedLabel(QWidget* parent) : QLabel(parent) {
  setSizePolicy(QSizePolicy::Ignored, QSizePolicy::Preferred);
}

void ElidedLabel::SetFull(const QString& text) {
  full_ = text;
  setToolTip(text);
  Elide();
}

QSize ElidedLabel::minimumSizeHint() const {
  return {0, fontMetrics().height()};
}

void ElidedLabel::resizeEvent(QResizeEvent* event) {
  QLabel::resizeEvent(event);
  Elide();
}

void ElidedLabel::Elide() {
  setText(width() <= 0
              ? full_
              : fontMetrics().elidedText(full_, Qt::ElideRight, width()));
}

AgentPrompt::AgentPrompt(QWidget* parent) : QPlainTextEdit(parent) {
  setObjectName(QStringLiteral("AgentPrompt"));
  setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
  setTabChangesFocus(true);
}

void AgentPrompt::keyPressEvent(QKeyEvent* event) {
  if ((event->key() == Qt::Key_Return || event->key() == Qt::Key_Enter) &&
      (event->modifiers() & Qt::ControlModifier)) {
    emit SendRequested();
    return;
  }
  QPlainTextEdit::keyPressEvent(event);
}

void AgentPrompt::focusInEvent(QFocusEvent* event) {
  QPlainTextEdit::focusInEvent(event);
  emit FocusChanged(true);
}

void AgentPrompt::focusOutEvent(QFocusEvent* event) {
  QPlainTextEdit::focusOutEvent(event);
  emit FocusChanged(false);
}

AgentPanel::AgentPanel(QWidget* parent) : QWidget(parent) {
  setObjectName(QStringLiteral("AgentPanel"));
  auto* layout = new QVBoxLayout(this);
  layout->setContentsMargins(0, 0, 0, 0);
  layout->setSpacing(0);
  pages_ = new QStackedWidget(this);
  pages_->addWidget(BuildList());
  pages_->addWidget(BuildThread());
  layout->addWidget(pages_);
  ticker_.setInterval(kTickMs);
  connect(&ticker_, &QTimer::timeout, this, &AgentPanel::SyncWorking);
  for (auto signal : {&EventHub::AssistantChanged, &EventHub::SettingsChanged,
                      &EventHub::ProjectChanged}) {
    connect(EventHub::Instance(), signal, this, &AgentPanel::Sync);
  }
  Sync();
}

QWidget* AgentPanel::BuildList() {
  auto* page = new QWidget(this);
  page->setObjectName(QStringLiteral("AgentThreads"));
  auto* layout = new QVBoxLayout(page);
  layout->setContentsMargins(0, 0, 0, 0);
  layout->setSpacing(0);
  auto* bar = new QWidget(page);
  bar->setObjectName(QStringLiteral("AgentBar"));
  bar->setFixedHeight(kBarHeight);
  auto* line = new QHBoxLayout(bar);
  line->setContentsMargins(8, 0, 6, 0);
  line->setSpacing(4);
  search_ = new QLineEdit(bar);
  search_->setObjectName(QStringLiteral("AgentSearch"));
  search_->setProperty("role", "search");
  search_->setPlaceholderText(QStringLiteral("Search threads"));
  search_->addAction(
      App::Get().Icon(QStringLiteral("search"),
                      App::Get().Token(QStringLiteral("muted-foreground")), 14),
      QLineEdit::LeadingPosition);
  line->addWidget(search_, 1);
  auto* add = MakeIconButton(QStringLiteral("plus"),
                             QStringLiteral("New thread"), bar, 16);
  add->setObjectName(QStringLiteral("AgentNewThread"));
  line->addWidget(add);
  layout->addWidget(bar);
  list_off_ = new QWidget(page);
  auto* off = new QVBoxLayout(list_off_);
  off->setContentsMargins(8, 8, 8, 0);
  layout->addWidget(list_off_);
  auto* scroll = new QScrollArea(page);
  scroll->setWidgetResizable(true);
  scroll->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
  auto* content = new QWidget(scroll);
  auto* body = new QVBoxLayout(content);
  body->setContentsMargins(6, 4, 6, 6);
  body->setSpacing(0);
  threads_ = new QVBoxLayout();
  threads_->setSpacing(0);
  body->addLayout(threads_);
  threads_empty_ = MakeLabel(QString(), "muted", content);
  threads_empty_->setObjectName(QStringLiteral("AgentThreadsEmpty"));
  threads_empty_->setWordWrap(true);
  threads_empty_->setAlignment(Qt::AlignCenter);
  threads_empty_->setContentsMargins(16, 24, 16, 0);
  body->addWidget(threads_empty_);
  body->addStretch(1);
  scroll->setWidget(content);
  layout->addWidget(scroll, 1);
  connect(add, &QToolButton::clicked, this, &AgentPanel::NewThread);
  connect(search_, &QLineEdit::textChanged, this, &AgentPanel::FilterThreads);
  return page;
}

QWidget* AgentPanel::BuildThread() {
  auto* page = new QWidget(this);
  page->setObjectName(QStringLiteral("AgentThreadPage"));
  auto* layout = new QVBoxLayout(page);
  layout->setContentsMargins(0, 0, 0, 0);
  layout->setSpacing(0);
  auto* bar = new QWidget(page);
  bar->setObjectName(QStringLiteral("AgentBar"));
  bar->setFixedHeight(kBarHeight);
  auto* line = new QHBoxLayout(bar);
  line->setContentsMargins(4, 0, 6, 0);
  line->setSpacing(4);
  auto* back = MakeIconButton(QStringLiteral("chevron-left"),
                              QStringLiteral("All threads"), bar, 16);
  back->setObjectName(QStringLiteral("AgentBack"));
  line->addWidget(back);
  title_ = new ElidedLabel(bar);
  title_->setObjectName(QStringLiteral("AgentTitle"));
  line->addWidget(title_, 1);
  menu_ = MakeIconButton(QStringLiteral("ellipsis"),
                         QStringLiteral("Thread actions"), bar, 16);
  menu_->setObjectName(QStringLiteral("AgentMenu"));
  menu_->setPopupMode(QToolButton::InstantPopup);
  auto* actions = new QMenu(menu_);
  actions->addAction(QStringLiteral("End this thread"), this,
                     [] { App::Get().session().agent_stop(); });
  menu_->setMenu(actions);
  line->addWidget(menu_);
  layout->addWidget(bar);
  scroll_ = new QScrollArea(page);
  scroll_->setObjectName(QStringLiteral("AgentConversation"));
  scroll_->setWidgetResizable(true);
  scroll_->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
  auto* content = new QWidget(scroll_);
  auto* body = new QVBoxLayout(content);
  body->setContentsMargins(12, 12, 12, 8);
  body->setSpacing(10);
  entries_ = new QVBoxLayout();
  entries_->setSpacing(10);
  body->addLayout(entries_);
  asks_ = new QWidget(content);
  asks_layout_ = new QVBoxLayout(asks_);
  asks_layout_->setContentsMargins(0, 0, 0, 0);
  asks_layout_->setSpacing(10);
  body->addWidget(asks_);
  body->addStretch(1);
  scroll_->setWidget(content);
  layout->addWidget(scroll_, 1);
  empty_ = new QWidget(page);
  empty_->setObjectName(QStringLiteral("AgentEmpty"));
  auto* empty = new QVBoxLayout(empty_);
  empty->setContentsMargins(28, 0, 28, 0);
  empty->setSpacing(6);
  empty->addStretch(1);
  auto* lead = MakeLabel(QStringLiteral("Ask for a change to this project"),
                         nullptr, empty_);
  lead->setObjectName(QStringLiteral("AgentNoteTitle"));
  lead->setAlignment(Qt::AlignCenter);
  lead->setWordWrap(true);
  empty->addWidget(lead);
  auto* explained = MakeLabel(
      QStringLiteral("The agent reads the project's files, changes it with "
                     "Lua and asks before it changes anything."),
      "muted", empty_);
  explained->setAlignment(Qt::AlignCenter);
  explained->setWordWrap(true);
  empty->addWidget(explained);
  empty->addStretch(1);
  layout->addWidget(empty_, 1);
  working_ = MakeLabel(QString(), "muted", page);
  working_->setObjectName(QStringLiteral("AgentWorking"));
  working_->setContentsMargins(14, 2, 12, 6);
  layout->addWidget(working_);
  notes_ = new QWidget(page);
  notes_layout_ = new QVBoxLayout(notes_);
  notes_layout_->setContentsMargins(8, 0, 8, 8);
  notes_layout_->setSpacing(8);
  layout->addWidget(notes_);
  composer_ = new QFrame(page);
  composer_->setObjectName(QStringLiteral("AgentComposer"));
  auto* frame = new QVBoxLayout(composer_);
  frame->setContentsMargins(10, 8, 6, 6);
  frame->setSpacing(4);
  prompt_ = new AgentPrompt(composer_);
  frame->addWidget(prompt_);
  auto* tools = new QHBoxLayout();
  tools->setSpacing(6);
  mode_ = new QToolButton(composer_);
  mode_->setObjectName(QStringLiteral("AgentMode"));
  mode_->setPopupMode(QToolButton::InstantPopup);
  mode_->setToolButtonStyle(Qt::ToolButtonTextBesideIcon);
  mode_->setLayoutDirection(Qt::RightToLeft);
  mode_->setIcon(App::Get().Icon(QStringLiteral("chevron-down"),
                                 App::Get().Token(QStringLiteral("foreground")),
                                 12));
  mode_->setIconSize(QSize(12, 12));
  mode_->setCursor(Qt::PointingHandCursor);
  mode_->setMenu(new QMenu(mode_));
  tools->addWidget(mode_);
  tools->addStretch(1);
  auto* hint = MakeLabel(QStringLiteral("Ctrl+Enter"), "muted", composer_);
  hint->setObjectName(QStringLiteral("AgentHint"));
  tools->addWidget(hint);
  send_ = new QToolButton(composer_);
  send_->setObjectName(QStringLiteral("AgentSend"));
  send_->setFixedSize(kSendSize, kSendSize);
  send_->setIconSize(QSize(14, 14));
  send_->setCursor(Qt::PointingHandCursor);
  tools->addWidget(send_);
  frame->addLayout(tools);
  auto* holder = new QWidget(page);
  auto* holding = new QVBoxLayout(holder);
  holding->setContentsMargins(8, 0, 8, 8);
  holding->addWidget(composer_);
  layout->addWidget(holder);
  connect(back, &QToolButton::clicked, this,
          [] { App::Get().session().agent_back(); });
  connect(send_, &QToolButton::clicked, this, [this] {
    if (busy_) {
      App::Get().session().agent_cancel();
    } else {
      Send();
    }
  });
  connect(prompt_, &AgentPrompt::SendRequested, this, &AgentPanel::Send);
  connect(prompt_, &QPlainTextEdit::textChanged, this, [this] {
    FitPrompt();
    send_->setEnabled(
        busy_ || (CanSend() && !prompt_->toPlainText().trimmed().isEmpty()));
  });
  connect(prompt_, &AgentPrompt::FocusChanged, this, [this](bool focused) {
    composer_->setProperty("focus", focused);
    Repolish(composer_);
  });
  FitPrompt();
  return page;
}

void AgentPanel::showEvent(QShowEvent* event) {
  QWidget::showEvent(event);
  Sync();
}

void AgentPanel::FitPrompt() {
  const int lines = std::clamp(
      static_cast<int>(prompt_->document()->size().height()), 2, kPromptLines);
  prompt_->setFixedHeight(lines * prompt_->fontMetrics().lineSpacing() + 4);
}

void AgentPanel::NewThread() {
  try {
    App::Get().session().agent_new_thread();
  } catch (const rust::Error& error) {
    App::Get().ShowError(QString::fromUtf8(error.what()));
  }
}

void AgentPanel::Send() {
  const QString text = prompt_->toPlainText().trimmed();
  if (text.isEmpty()) return;
  if (live_) {
    if (App::Get().session().agent_prompt(ToRust(text))) prompt_->clear();
    return;
  }
  if (thread_.isEmpty()) return;
  try {
    App::Get().session().agent_continue(ToRust(thread_), ToRust(text));
    prompt_->clear();
  } catch (const rust::Error& error) {
    App::Get().ShowError(QString::fromUtf8(error.what()));
  }
}

void AgentPanel::FilterThreads() {
  shown_threads_.clear();
  Sync();
}

void AgentPanel::Sync() {
  const bridge::AgentPanelView view = App::Get().session().agent_view();
  const bool thread = !view.thread.empty();
  pages_->setCurrentIndex(thread ? 1 : 0);
  if (thread) {
    SyncThread(view);
  } else {
    SyncList(view);
  }
}

void AgentPanel::SyncList(const bridge::AgentPanelView& view) {
  const QString filter = search_->text().trimmed();
  QStringList keys{filter, view.assistant_on ? QStringLiteral("on") : QString(),
                   view.has_project ? QStringLiteral("p") : QString()};
  for (const auto& entry : view.threads) {
    keys << ToQ(entry.id) + ToQ(entry.title) + ToQ(entry.when) +
                ToQ(entry.group) + ToQ(entry.state);
  }
  const QString key = keys.join(QChar(0x1e));
  if (key == shown_threads_) return;
  shown_threads_ = key;
  while (QLayoutItem* item = list_off_->layout()->takeAt(0)) {
    delete item->widget();
    delete item;
  }
  if (!view.assistant_on) {
    QFrame* note =
        Note(QStringLiteral("plug"), QStringLiteral("muted-foreground"),
             QStringLiteral("The assistant is off"),
             QStringLiteral("Agents reach the project only while "
                            "it's on."));
    auto* on = MakeButton(QStringLiteral("Turn on the assistant"), "primary",
                          QString(), note);
    on->setObjectName(QStringLiteral("AgentTurnOn"));
    connect(on, &QPushButton::clicked, this, [] {
      App::Get().session().patch_settings(
          ToRust(QStringLiteral("{\"assistant\":{\"enabled\":true}}")));
    });
    ButtonsOf(note)->insertWidget(0, on);
    list_off_->layout()->addWidget(note);
  }
  list_off_->setVisible(!view.assistant_on);
  while (QLayoutItem* item = threads_->takeAt(0)) {
    delete item->widget();
    delete item;
  }
  QString group;
  int shown = 0;
  for (const auto& entry : view.threads) {
    const QString title = ToQ(entry.title);
    if (!filter.isEmpty() && !title.contains(filter, Qt::CaseInsensitive)) {
      continue;
    }
    if (ToQ(entry.group) != group) {
      group = ToQ(entry.group);
      auto* heading = MakeLabel(group, nullptr, this);
      heading->setObjectName(QStringLiteral("AgentGroup"));
      threads_->addWidget(heading);
    }
    auto* row = new QPushButton(this);
    row->setObjectName(QStringLiteral("AgentThread"));
    row->setProperty("variant", "row");
    row->setProperty("thread", ToQ(entry.id));
    row->setAccessibleName(title);
    row->setFixedHeight(kThreadHeight);
    row->setCursor(Qt::PointingHandCursor);
    auto* line = new QHBoxLayout(row);
    line->setContentsMargins(8, 0, 8, 0);
    line->setSpacing(8);
    auto* dot = new QLabel(row);
    dot->setObjectName(QStringLiteral("AgentDot"));
    dot->setProperty("state", ToQ(entry.state));
    dot->setFixedSize(6, 6);
    dot->setVisible(!entry.state.empty());
    auto* slot = new QWidget(row);
    slot->setFixedWidth(10);
    auto* slotting = new QHBoxLayout(slot);
    slotting->setContentsMargins(0, 0, 0, 0);
    slotting->addWidget(dot, 0, Qt::AlignCenter);
    line->addWidget(slot);
    auto* name = new ElidedLabel(row);
    name->SetFull(title);
    line->addWidget(name, 1);
    auto* when = MakeLabel(ToQ(entry.when), "muted", row);
    when->setObjectName(QStringLiteral("AgentWhen"));
    line->addWidget(when);
    for (QWidget* child :
         {slot, static_cast<QWidget*>(dot), static_cast<QWidget*>(name),
          static_cast<QWidget*>(when)}) {
      child->setAttribute(Qt::WA_TransparentForMouseEvents);
    }
    const QString id = ToQ(entry.id);
    connect(row, &QPushButton::clicked, this,
            [id] { App::Get().session().agent_open(ToRust(id)); });
    threads_->addWidget(row);
    ++shown;
  }
  threads_empty_->setText(
      !view.has_project ? QStringLiteral("Open a project to work on it with "
                                         "an agent.")
      : !filter.isEmpty()
          ? QStringLiteral("No thread matches.")
          : QStringLiteral("No threads yet. Start one with +."));
  threads_empty_->setVisible(shown == 0);
}

void AgentPanel::SyncThread(const bridge::AgentPanelView& view) {
  const QString thread = ToQ(view.thread);
  if (thread != thread_) {
    thread_ = thread;
    for (QWidget* widget : entry_widgets_) delete widget;
    entry_widgets_.clear();
    shown_entries_.clear();
    shown_asks_.clear();
  }
  live_ = view.live;
  status_ = ToQ(view.status);
  busy_ = status_ == QStringLiteral("working");
  working_since_ = view.working_since;
  title_->SetFull(view.title.empty() ? QStringLiteral("New thread")
                                     : ToQ(view.title));
  menu_->setVisible(live_);
  SyncEntries(view);
  SyncAsks(view);
  const bool blank = view.entries.empty() && shown_asks_.isEmpty();
  scroll_->setVisible(!blank);
  empty_->setVisible(blank);
  SyncNotes(view);
  SyncComposer(view);
  SyncWorking();
  if (busy_ || status_ == QStringLiteral("starting")) {
    if (!ticker_.isActive()) ticker_.start();
  } else {
    ticker_.stop();
  }
}

void AgentPanel::SyncEntries(const bridge::AgentPanelView& view) {
  std::vector<QString> keys;
  keys.reserve(view.entries.size());
  for (const auto& entry : view.entries) keys.push_back(EntryKey(entry));
  size_t same = 0;
  while (same < keys.size() && same < shown_entries_.size() &&
         keys.at(same) == shown_entries_.at(same)) {
    ++same;
  }
  if (same == keys.size() && same == shown_entries_.size()) return;
  QScrollBar* bar = scroll_->verticalScrollBar();
  const bool follow = bar->value() >= bar->maximum() - 4;
  for (size_t index = same; index < entry_widgets_.size(); ++index) {
    delete entry_widgets_.at(index);
  }
  entry_widgets_.resize(same);
  for (size_t index = same; index < keys.size(); ++index) {
    QWidget* widget = EntryWidget(view.entries.at(index));
    entries_->addWidget(widget);
    entry_widgets_.push_back(widget);
  }
  shown_entries_ = std::move(keys);
  if (follow) {
    QTimer::singleShot(0, bar, [bar] { bar->setValue(bar->maximum()); });
  }
}

QWidget* AgentPanel::EntryWidget(const bridge::AgentEntryView& entry) {
  const QString kind = ToQ(entry.kind);
  const QString text = ToQ(entry.text);
  if (kind == QStringLiteral("user")) {
    auto* bubble = new QLabel(text);
    bubble->setObjectName(QStringLiteral("AgentUser"));
    bubble->setTextFormat(Qt::PlainText);
    bubble->setWordWrap(true);
    bubble->setTextInteractionFlags(Qt::TextSelectableByMouse);
    return new BubbleRow(bubble);
  }
  if (kind == QStringLiteral("agent")) {
    return new MarkdownView(text);
  }
  if (kind == QStringLiteral("tools")) {
    if (entry.tools.size() == 1) return ToolWidget(entry.tools.front());
    auto* box = new QWidget();
    box->setObjectName(QStringLiteral("AgentTools"));
    auto* stack = new QVBoxLayout(box);
    stack->setContentsMargins(0, 0, 0, 0);
    stack->setSpacing(0);
    const QString id = ToQ(entry.id);
    auto* head = new ClickRow(
        [id] { App::Get().session().agent_toggle(ToRust(id)); }, box);
    head->setObjectName(QStringLiteral("AgentToolGroup"));
    auto* line = new QHBoxLayout(head);
    line->setContentsMargins(0, 0, 0, 0);
    line->setSpacing(6);
    line->addWidget(IconLabel(ToQ(entry.icon),
                              entry.failed ? QStringLiteral("destructive")
                                           : QStringLiteral("muted-foreground"),
                              kToolIcon, head));
    auto* label = new ElidedLabel(head);
    label->setObjectName(QStringLiteral("AgentToolLabel"));
    label->setProperty("running", entry.running);
    label->SetFull(text);
    label->setAttribute(Qt::WA_TransparentForMouseEvents);
    line->addWidget(label, 1);
    line->addWidget(IconLabel(entry.expanded ? QStringLiteral("chevron-down")
                                             : QStringLiteral("chevron-right"),
                              QStringLiteral("muted-foreground"), 12, head));
    stack->addWidget(head);
    if (entry.expanded) {
      for (const auto& tool : entry.tools) stack->addWidget(ToolWidget(tool));
    }
    return box;
  }
  auto* notice = MakeLabel(text, "muted");
  notice->setObjectName(QStringLiteral("AgentNotice"));
  notice->setWordWrap(true);
  return notice;
}

QWidget* AgentPanel::ToolWidget(const bridge::AgentToolView& tool) {
  auto* box = new QWidget();
  box->setObjectName(QStringLiteral("AgentTool"));
  auto* stack = new QVBoxLayout(box);
  stack->setContentsMargins(0, 0, 0, 0);
  stack->setSpacing(0);
  const QString id = ToQ(tool.id);
  const bool opens = !tool.output.empty();
  auto* row = new ClickRow(opens ? std::function<void()>([id] {
                             App::Get().session().agent_toggle(ToRust(id));
                           })
                                 : std::function<void()>(),
                           box);
  row->setMinimumHeight(kRowHeight);
  auto* line = new QHBoxLayout(row);
  line->setContentsMargins(0, 0, 0, 0);
  line->setSpacing(6);
  line->addWidget(IconLabel(ToQ(tool.icon),
                            tool.failed ? QStringLiteral("destructive")
                                        : QStringLiteral("muted-foreground"),
                            kToolIcon, row));
  auto* label = new ElidedLabel(row);
  label->setObjectName(QStringLiteral("AgentToolLabel"));
  label->setProperty("running", tool.running);
  label->SetFull(ToQ(tool.label));
  label->setAttribute(Qt::WA_TransparentForMouseEvents);
  line->addWidget(label, 1);
  if (tool.failed) {
    line->addWidget(
        IconLabel(QStringLiteral("x"), QStringLiteral("destructive"), 12, row));
  }
  stack->addWidget(row);
  if (tool.expanded && opens) {
    QLabel* detail = Mono(ToQ(tool.output), "AgentDetail", kDetailLines, box);
    detail->setContentsMargins(kDetailIndent, 0, 0, 6);
    stack->addWidget(detail);
  }
  return box;
}

QWidget* AgentPanel::AskWidget(const bridge::AgentAskView& ask) {
  auto* box = new QWidget();
  box->setObjectName(QStringLiteral("AgentAsk"));
  auto* stack = new QVBoxLayout(box);
  stack->setContentsMargins(0, 0, 0, 0);
  stack->setSpacing(2);
  auto* row = new QWidget(box);
  auto* line = new QHBoxLayout(row);
  line->setContentsMargins(0, 0, 0, 0);
  line->setSpacing(6);
  line->addWidget(IconLabel(QStringLiteral("shield-alert"),
                            QStringLiteral("warning"), kToolIcon, row));
  auto* title = new ElidedLabel(row);
  title->setObjectName(QStringLiteral("AgentAskTitle"));
  title->SetFull(ToQ(ask.title));
  line->addWidget(title, 1);
  stack->addWidget(row);
  if (!ask.detail.empty()) {
    QLabel* detail = Mono(ToQ(ask.detail), "AgentAskDetail", kAskLines, box);
    detail->setContentsMargins(kDetailIndent, 2, 0, 4);
    stack->addWidget(detail);
  }
  auto* answers = new QWidget(box);
  auto* answering = new QHBoxLayout(answers);
  answering->setContentsMargins(kDetailIndent, 0, 0, 0);
  answering->setSpacing(10);
  QWidget* flow = answers;
  const QString id = ToQ(ask.id);
  const bridge::AgentOptionView* reject = nullptr;
  const bridge::AgentOptionView* allow = nullptr;
  for (const auto& option : ask.options) {
    if (reject == nullptr && ToQ(option.kind) == QStringLiteral("rejectOnce")) {
      reject = &option;
    }
    if (allow == nullptr && ToQ(option.kind) == QStringLiteral("allowOnce")) {
      allow = &option;
    }
  }
  auto answer = [id](const QString& option) {
    App::Get().session().agent_answer(ToRust(id), ToRust(option));
  };
  auto add = [&](const QString& text, bool muted, const QString& value) {
    auto* button = InlineButton(text, muted, flow);
    button->setObjectName(QStringLiteral("AgentAnswer"));
    button->setProperty("option", value);
    connect(button, &QPushButton::clicked, this,
            [answer, value] { answer(value); });
    answering->addWidget(button);
  };
  if (allow != nullptr) {
    add(QStringLiteral("Allow"), false, ToQ(allow->id));
  }
  if (reject != nullptr) {
    add(QStringLiteral("Reject"), true, ToQ(reject->id));
  }
  QMenu* more = nullptr;
  for (const auto& option : ask.options) {
    if (&option == allow || &option == reject) continue;
    const QString value = ToQ(option.id);
    if (allow == nullptr && reject == nullptr) {
      add(ToQ(option.name), false, value);
      continue;
    }
    if (more == nullptr) more = new QMenu(flow);
    more->addAction(ToQ(option.name), this, [answer, value] { answer(value); });
  }
  if (more != nullptr) {
    auto* button = InlineButton(QStringLiteral("More"), true, flow);
    button->setObjectName(QStringLiteral("AgentMore"));
    button->setMenu(more);
    answering->addWidget(button);
  }
  answering->addStretch(1);
  stack->addWidget(answers);
  return box;
}

void AgentPanel::SyncAsks(const bridge::AgentPanelView& view) {
  QStringList keys;
  for (const auto& ask : view.asks) {
    if (!ask.sign_in) keys << ToQ(ask.id) + ToQ(ask.detail);
  }
  const QString key = keys.join(QChar(0x1e));
  if (key == shown_asks_) return;
  shown_asks_ = key;
  while (QLayoutItem* item = asks_layout_->takeAt(0)) {
    delete item->widget();
    delete item;
  }
  for (const auto& ask : view.asks) {
    if (!ask.sign_in) asks_layout_->addWidget(AskWidget(ask));
  }
  asks_->setVisible(!keys.isEmpty());
  QScrollBar* bar = scroll_->verticalScrollBar();
  QTimer::singleShot(0, bar, [bar] { bar->setValue(bar->maximum()); });
}

QFrame* AgentPanel::Note(const QString& icon, const QString& tone,
                         const QString& title, const QString& text) {
  auto* note = new QFrame();
  note->setObjectName(QStringLiteral("AgentNote"));
  auto* line = new QHBoxLayout(note);
  line->setContentsMargins(10, 9, 10, 10);
  line->setSpacing(8);
  auto* mark = new QLabel(note);
  mark->setPixmap(App::Get().Glyph(icon, App::Get().Token(tone), 16));
  line->addWidget(mark, 0, Qt::AlignTop);
  auto* body = new QVBoxLayout();
  body->setSpacing(2);
  auto* heading = MakeLabel(title, nullptr, note);
  heading->setObjectName(QStringLiteral("AgentNoteTitle"));
  heading->setWordWrap(true);
  body->addWidget(heading);
  auto* said = MakeLabel(text, "muted", note);
  said->setObjectName(QStringLiteral("AgentNoteText"));
  said->setWordWrap(true);
  body->addWidget(said);
  body->addSpacing(5);
  auto* buttons = new QHBoxLayout();
  buttons->setSpacing(6);
  body->addLayout(buttons);
  line->addLayout(body, 1);
  buttons->addStretch(1);
  return note;
}

void AgentPanel::SyncNotes(const bridge::AgentPanelView& view) {
  QStringList keys{view.assistant_on ? QStringLiteral("on") : QString(),
                   live_ ? QStringLiteral("live") : QString(), status_,
                   ToQ(view.ended)};
  for (const auto& ask : view.asks) {
    if (!ask.sign_in) continue;
    for (const auto& option : ask.options) keys << ToQ(option.id);
  }
  const QString key = keys.join(QChar(0x1e));
  if (key == shown_notes_) return;
  shown_notes_ = key;
  while (QLayoutItem* item = notes_layout_->takeAt(0)) {
    delete item->widget();
    delete item;
  }
  if (!view.assistant_on) {
    QFrame* note =
        Note(QStringLiteral("plug"), QStringLiteral("muted-foreground"),
             QStringLiteral("The assistant is off"),
             QStringLiteral("Agents reach the project only while "
                            "it's on."));
    auto* on = MakeButton(QStringLiteral("Turn on the assistant"), "primary",
                          QString(), note);
    on->setObjectName(QStringLiteral("AgentTurnOn"));
    connect(on, &QPushButton::clicked, this, [] {
      App::Get().session().patch_settings(
          ToRust(QStringLiteral("{\"assistant\":{\"enabled\":true}}")));
    });
    ButtonsOf(note)->insertWidget(0, on);
    notes_layout_->addWidget(note);
  }
  if (live_ && status_ == QStringLiteral("ended")) {
    QString said = ToQ(view.ended);
    said.remove(QStringLiteral("The agent stopped: "));
    QFrame* note =
        Note(QStringLiteral("triangle-alert"), QStringLiteral("destructive"),
             QStringLiteral("The agent stopped"), said);
    auto* again = MakeButton(QStringLiteral("Start again"), "outline",
                             QStringLiteral("rotate-ccw"), note);
    again->setObjectName(QStringLiteral("AgentStartAgain"));
    connect(again, &QPushButton::clicked, this, &AgentPanel::NewThread);
    auto* log =
        MakeButton(QStringLiteral("Open the log"), "ghost", QString(), note);
    connect(log, &QPushButton::clicked, this, [] {
      try {
        App::Get().session().assistant_open_logs();
      } catch (const rust::Error& error) {
        App::Get().ShowError(QString::fromUtf8(error.what()));
      }
    });
    ButtonsOf(note)->insertWidget(0, again);
    ButtonsOf(note)->insertWidget(1, log);
    notes_layout_->addWidget(note);
  }
  for (const auto& ask : view.asks) {
    if (!ask.sign_in) continue;
    QFrame* note =
        Note(QStringLiteral("key-round"), QStringLiteral("warning"),
             ToQ(ask.title), QStringLiteral("The agent asks you to sign in."));
    int index = 0;
    for (const auto& option : ask.options) {
      auto* button =
          MakeButton(ToQ(option.name), index == 0 ? "primary" : "outline",
                     QString(), note);
      button->setObjectName(QStringLiteral("AgentSignIn"));
      const QString value = ToQ(option.id);
      connect(button, &QPushButton::clicked, this, [value] {
        App::Get().session().agent_answer(ToRust(QString::fromLatin1(kSignIn)),
                                          ToRust(value));
      });
      ButtonsOf(note)->insertWidget(index++, button);
    }
    notes_layout_->addWidget(note);
  }
  notes_->setVisible(notes_layout_->count() > 0);
}

void AgentPanel::SyncComposer(const bridge::AgentPanelView& view) {
  assistant_on_ = view.assistant_on;
  prompt_->setEnabled(assistant_on_);
  prompt_->setPlaceholderText(
      !assistant_on_ ? QStringLiteral("Turn on the assistant first")
      : live_ && status_ == QStringLiteral("starting")
          ? QStringLiteral("Starting the agent…")
          : QStringLiteral("Ask for a change…"));
  QMenu* modes = mode_->menu();
  modes->clear();
  QString current;
  for (const auto& mode : view.modes) {
    const QString value = ToQ(mode.value);
    QAction* action = modes->addAction(ToQ(mode.label), this, [value] {
      App::Get().session().agent_mode(ToRust(value));
    });
    action->setCheckable(true);
    action->setChecked(value == ToQ(view.mode));
    if (value == ToQ(view.mode)) current = ToQ(mode.label);
  }
  mode_->setText(current.isEmpty() && !view.modes.empty()
                     ? ToQ(view.modes.front().label)
                     : current);
  mode_->setVisible(!view.modes.empty());
  send_->setProperty("stop", busy_);
  send_->setToolTip(busy_ ? QStringLiteral("Stop") : QStringLiteral("Send"));
  send_->setIcon(App::Get().Icon(
      busy_ ? QStringLiteral("square") : QStringLiteral("arrow-up"),
      App::Get().Token(busy_ ? QStringLiteral("foreground")
                             : QStringLiteral("primary-foreground")),
      14));
  send_->setEnabled(busy_ ||
                    (CanSend() && !prompt_->toPlainText().trimmed().isEmpty()));
  Repolish(send_);
}

bool AgentPanel::CanSend() const {
  if (!assistant_on_) return false;
  return live_ ? status_ == QStringLiteral("ready") : !thread_.isEmpty();
}

void AgentPanel::SyncWorking() {
  if (busy_ && !shown_asks_.isEmpty()) {
    working_->setText(QStringLiteral("Waiting for your approval"));
  } else if (busy_ && working_since_ > 0) {
    working_->setText(
        QStringLiteral("Working for %1").arg(Elapsed(working_since_)));
  } else if (live_ && status_ == QStringLiteral("starting")) {
    working_->setText(QStringLiteral("Starting the agent…"));
  } else {
    working_->clear();
  }
  working_->setVisible(!working_->text().isEmpty());
}

}
