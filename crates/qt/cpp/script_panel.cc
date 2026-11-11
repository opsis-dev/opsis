#include "script_panel.h"

#include <QCompleter>
#include <QHBoxLayout>
#include <QInputDialog>
#include <QJsonDocument>
#include <QJsonObject>
#include <QKeyEvent>
#include <QMouseEvent>
#include <QScrollBar>
#include <QTextBlock>
#include <QVBoxLayout>
#include <algorithm>
#include <utility>

#include "app.h"
#include "event_hub.h"
#include "opsis-qt/src/bridge.rs.h"

namespace opsis {

namespace {

constexpr int kSaveDelay = 600;
constexpr int kHighlightDelay = 30;
constexpr double kTurnRatio = 1.6;
constexpr int kOutputShare = 2;
constexpr int kEditorShare = 3;
constexpr float kLineMarkAlpha = 0.18F;

QString LookSheet(const bridge::ScriptLookView& look) {
  return QStringLiteral(
             "QPlainTextEdit { font-family: \"JetBrains Mono\"; font-size: "
             "%1px; background: %2; color: %3; selection-background-color: "
             "%4; selection-color: %3; }")
      .arg(look.font_size)
      .arg(ToQ(look.background), ToQ(look.text), ToQ(look.selection));
}

QTextEdit::ExtraSelection LineMark(QPlainTextEdit* editor, int line,
                                   QColor tone) {
  QTextEdit::ExtraSelection mark;
  tone.setAlphaF(kLineMarkAlpha);
  mark.format.setBackground(tone);
  mark.format.setProperty(QTextFormat::FullWidthSelection, true);
  mark.cursor =
      QTextCursor(editor->document()->findBlockByLineNumber(line - 1));
  return mark;
}

QTextEdit::ExtraSelection BracketMark(QPlainTextEdit* editor, int at,
                                      const QColor& tone) {
  QTextEdit::ExtraSelection mark;
  mark.format.setBackground(tone);
  mark.cursor = QTextCursor(editor->document());
  mark.cursor.setPosition(at);
  mark.cursor.setPosition(at + 1, QTextCursor::KeepAnchor);
  return mark;
}

QString Outcome(const bridge::ScriptView& view) {
  const QString outcome = ToQ(view.outcome);
  if (view.running) return {};
  if (outcome == QStringLiteral("ok")) return QStringLiteral("Finished");
  if (outcome == QStringLiteral("stopped")) {
    return view.rolled_back ? QStringLiteral(
                                  "Stopped. Its changes to the project were "
                                  "undone.")
                            : QStringLiteral("Stopped");
  }
  if (outcome == QStringLiteral("failed")) {
    return view.rolled_back ? QStringLiteral(
                                  "Failed. Its changes to the project were "
                                  "undone.")
                            : QStringLiteral("Failed");
  }
  return {};
}

}

ScriptHighlighter::ScriptHighlighter(QTextDocument* document)
    : QSyntaxHighlighter(document) {}

void ScriptHighlighter::SetRanges(std::vector<Range> ranges) {
  ranges_ = std::move(ranges);
  rehighlight();
}

void ScriptHighlighter::highlightBlock(const QString& text) {
  const int start = currentBlock().position();
  const int end = start + static_cast<int>(text.size());
  auto first = std::lower_bound(ranges_.begin(), ranges_.end(), start,
                                [](const Range& range, int at) {
                                  return range.start + range.length <= at;
                                });
  for (auto it = first; it != ranges_.end() && it->start < end; ++it) {
    const int from = std::max(it->start, start);
    const int to = std::min(it->start + it->length, end);
    if (to > from) setFormat(from - start, to - from, it->format);
  }
}

ScriptEditor::ScriptEditor(QWidget* parent) : QPlainTextEdit(parent) {
  setObjectName(QStringLiteral("ScriptEditor"));
  setAccessibleName(QStringLiteral("Script"));
  setFrameShape(QFrame::NoFrame);
  setLineWrapMode(QPlainTextEdit::NoWrap);
  setTabChangesFocus(false);
  connect(this, &QPlainTextEdit::cursorPositionChanged, this,
          &ScriptEditor::ShowBracket);
}

bool ScriptEditor::event(QEvent* event) {
  if (event->type() == QEvent::ShortcutOverride) {
    auto* key = static_cast<QKeyEvent*>(event);
    const bool command = (key->modifiers() & Qt::ControlModifier) != 0;
    const bool enter =
        key->key() == Qt::Key_Return || key->key() == Qt::Key_Enter;
    if ((command && (enter || key->key() == Qt::Key_Slash)) ||
        key->key() == Qt::Key_Tab || key->key() == Qt::Key_Backtab) {
      event->accept();
      return true;
    }
  }
  return QPlainTextEdit::event(event);
}

void ScriptEditor::keyPressEvent(QKeyEvent* event) {
  const Qt::KeyboardModifiers modifiers = event->modifiers();
  const bool command = (modifiers & Qt::ControlModifier) != 0;
  const bool enter =
      event->key() == Qt::Key_Return || event->key() == Qt::Key_Enter;
  if (command && enter) {
    emit RunRequested();
    return;
  }
  if (command && event->key() == Qt::Key_Slash) {
    Handle(QStringLiteral("comment"));
    return;
  }
  const bool plain =
      (modifiers & ~(Qt::ShiftModifier | Qt::KeypadModifier)) == Qt::NoModifier;
  if (event->key() == Qt::Key_Tab) {
    Handle(QStringLiteral("tab"));
    return;
  }
  if (event->key() == Qt::Key_Backtab) {
    Handle(QStringLiteral("backtab"));
    return;
  }
  if (plain && enter && Handle(QStringLiteral("enter"))) return;
  if (plain && event->key() == Qt::Key_Backspace &&
      Handle(QStringLiteral("backspace"))) {
    return;
  }
  const QString text = event->text();
  if (plain && text.size() == 1 && text.at(0).isPrint() && Handle(text)) {
    return;
  }
  QPlainTextEdit::keyPressEvent(event);
}

bool ScriptEditor::Handle(const QString& key) {
  const QTextCursor current = textCursor();
  const bridge::TextChangeView change = App::Get().session().script_key(
      ToRust(toPlainText()), static_cast<std::uint32_t>(current.anchor()),
      static_cast<std::uint32_t>(current.position()), ToRust(key));
  if (!change.handled) return false;
  QTextCursor edit(document());
  edit.beginEditBlock();
  edit.setPosition(static_cast<int>(change.start));
  edit.setPosition(static_cast<int>(change.end), QTextCursor::KeepAnchor);
  const QString inserted = ToQ(change.text);
  if (edit.hasSelection() || !inserted.isEmpty()) edit.insertText(inserted);
  edit.endEditBlock();
  QTextCursor after(document());
  after.setPosition(static_cast<int>(change.anchor));
  after.setPosition(static_cast<int>(change.cursor), QTextCursor::KeepAnchor);
  setTextCursor(after);
  ensureCursorVisible();
  return true;
}

void ScriptEditor::MarkLine(int line) {
  marked_ = line;
  const QTextBlock block = document()->findBlockByLineNumber(line - 1);
  if (block.isValid()) {
    QTextCursor at(block);
    setTextCursor(at);
    centerCursor();
  }
  ShowBracket();
}

void ScriptEditor::SetMarks(const QColor& bracket, const QColor& error) {
  bracket_ = bracket;
  error_ = error;
  ShowBracket();
}

void ScriptEditor::ClearMark() {
  if (marked_ == 0) return;
  marked_ = 0;
  ShowBracket();
}

void ScriptEditor::ShowBracket() {
  QList<QTextEdit::ExtraSelection> marks;
  if (marked_ > 0) marks.append(LineMark(this, marked_, error_));
  const bridge::BracketView bracket = App::Get().session().script_bracket(
      ToRust(toPlainText()),
      static_cast<std::uint32_t>(textCursor().position()));
  if (bracket.found) {
    marks.append(BracketMark(this, static_cast<int>(bracket.open), bracket_));
    marks.append(BracketMark(this, static_cast<int>(bracket.close), bracket_));
  }
  setExtraSelections(marks);
}

ScriptPanel::ScriptPanel(QWidget* parent)
    : QWidget(parent),
      files_(new Combo(this)),
      new_(MakeButton(QStringLiteral("New"), "ghost", QStringLiteral("plus"),
                      this)),
      rename_(MakeButton(QStringLiteral("Rename"), "ghost",
                         QStringLiteral("pencil"), this)),
      run_(MakeButton(QStringLiteral("Run"), "outline", QStringLiteral("play"),
                      this)),
      stop_(MakeButton(QStringLiteral("Stop"), "outline",
                       QStringLiteral("square"), this)),
      badge_(MakeLabel(QStringLiteral("Running"), "badge", this)),
      status_(MakeLabel(QString(), "muted", this)),
      theme_(new Combo(this)),
      size_(new QSpinBox(this)),
      split_(new QSplitter(Qt::Horizontal, this)),
      editor_(new ScriptEditor(split_)),
      highlighter_(new ScriptHighlighter(editor_->document())),
      output_(new QPlainTextEdit(split_)) {
  setObjectName(QStringLiteral("Panel"));
  setAttribute(Qt::WA_StyledBackground);
  auto* layout = new QVBoxLayout(this);
  layout->setContentsMargins(0, 0, 0, 0);
  layout->setSpacing(0);
  auto* bar = new QWidget(this);
  bar->setObjectName(QStringLiteral("TimelineToolbar"));
  bar->setAttribute(Qt::WA_StyledBackground);
  auto* row = new QHBoxLayout(bar);
  row->setContentsMargins(8, 4, 8, 4);
  row->setSpacing(6);
  files_->setObjectName(QStringLiteral("ScriptFiles"));
  files_->setAccessibleName(QStringLiteral("Scripts"));
  files_->setMinimumContentsLength(16);
  files_->setPlaceholderText(QStringLiteral("No scripts yet"));
  row->addWidget(files_);
  new_->setObjectName(QStringLiteral("ScriptNew"));
  new_->setToolTip(
      QStringLiteral("Make a script in the project's scripts "
                     "folder"));
  row->addWidget(new_);
  rename_->setObjectName(QStringLiteral("ScriptRename"));
  rename_->setToolTip(QStringLiteral("Rename the open script"));
  row->addWidget(rename_);
  run_->setObjectName(QStringLiteral("ScriptRun"));
  run_->setToolTip(QStringLiteral("Run the script (Ctrl+Enter)"));
  row->addWidget(run_);
  stop_->setObjectName(QStringLiteral("ScriptStop"));
  row->addWidget(stop_);
  badge_->setObjectName(QStringLiteral("ScriptBadge"));
  badge_->setToolTip(
      QStringLiteral("Changes made while a script runs, yours included, are "
                     "undone with it if it fails or is stopped."));
  row->addWidget(badge_);
  status_->setObjectName(QStringLiteral("ScriptStatus"));
  row->addWidget(status_, 1);
  theme_->setObjectName(QStringLiteral("ScriptTheme"));
  theme_->setAccessibleName(QStringLiteral("Code theme"));
  theme_->setToolTip(QStringLiteral("Code theme"));
  theme_->setEditable(true);
  theme_->setInsertPolicy(QComboBox::NoInsert);
  theme_->completer()->setCompletionMode(QCompleter::PopupCompletion);
  theme_->completer()->setFilterMode(Qt::MatchContains);
  theme_->setSizePolicy(QSizePolicy::Fixed, QSizePolicy::Fixed);
  for (const auto& choice : App::Get().session().script_themes()) {
    theme_->addItem(ToQ(choice.label), ToQ(choice.value));
  }
  row->addWidget(theme_);
  size_->setObjectName(QStringLiteral("ScriptFontSize"));
  size_->setAccessibleName(QStringLiteral("Code size"));
  size_->setToolTip(QStringLiteral("Code size"));
  size_->setSuffix(QStringLiteral(" px"));
  row->addWidget(size_);
  layout->addWidget(bar);
  output_->setObjectName(QStringLiteral("ScriptOutput"));
  output_->setAccessibleName(QStringLiteral("Script output"));
  output_->setReadOnly(true);
  output_->setFrameShape(QFrame::NoFrame);
  output_->viewport()->installEventFilter(this);
  split_->setObjectName(QStringLiteral("ScriptSplit"));
  split_->addWidget(editor_);
  split_->addWidget(output_);
  split_->setStretchFactor(0, kEditorShare);
  split_->setStretchFactor(1, kOutputShare);
  layout->addWidget(split_, 1);

  save_.setSingleShot(true);
  save_.setInterval(kSaveDelay);
  connect(&save_, &QTimer::timeout, this, &ScriptPanel::Save);
  highlight_.setSingleShot(true);
  highlight_.setInterval(kHighlightDelay);
  connect(&highlight_, &QTimer::timeout, this, &ScriptPanel::Highlight);
  connect(editor_, &QPlainTextEdit::textChanged, this, [this] {
    if (loading_) return;
    editor_->ClearMark();
    save_.start();
    highlight_.start();
  });
  connect(editor_, &ScriptEditor::RunRequested, this, &ScriptPanel::Run);
  connect(run_, &QPushButton::clicked, this, &ScriptPanel::Run);
  connect(stop_, &QPushButton::clicked, this,
          [] { App::Get().session().script_stop(); });
  connect(new_, &QPushButton::clicked, this, [this] {
    Save();
    try {
      App::Get().session().script_new();
    } catch (const rust::Error& error) {
      App::Get().ShowError(QString::fromUtf8(error.what()));
    }
  });
  connect(rename_, &QPushButton::clicked, this, &ScriptPanel::Rename);
  connect(files_, &QComboBox::activated, this, [this](int index) {
    Save();
    App::Get().session().script_open(ToRust(files_->itemText(index)));
  });
  for (auto signal : {&EventHub::ScriptChanged, &EventHub::ProjectChanged}) {
    connect(EventHub::Instance(), signal, this, &ScriptPanel::Rebuild);
  }
  connect(theme_, &QComboBox::activated, this, [this](int index) {
    const QString name = theme_->itemData(index).toString();
    if (name.isEmpty()) return;
    QJsonObject script;
    script.insert(QStringLiteral("theme"), name);
    QJsonObject patch;
    patch.insert(QStringLiteral("script"), script);
    App::Get().session().patch_settings(ToRust(QString::fromUtf8(
        QJsonDocument(patch).toJson(QJsonDocument::Compact))));
  });
  connect(size_, &QSpinBox::valueChanged, this, [](int size) {
    App::Get().session().patch_settings(
        ToRust(QStringLiteral(R"({"script":{"fontSize":%1}})").arg(size)));
  });
  connect(EventHub::Instance(), &EventHub::SettingsChanged, this,
          &ScriptPanel::ApplyLook);
  connect(EventHub::Instance(), &EventHub::FlushDrafts, this,
          &ScriptPanel::Save);
  ApplyLook();
  Rebuild();
}

ScriptPanel::~ScriptPanel() = default;

void ScriptPanel::resizeEvent(QResizeEvent* event) {
  QWidget::resizeEvent(event);
  const Qt::Orientation turn =
      width() >= kTurnRatio * height() ? Qt::Horizontal : Qt::Vertical;
  if (split_->orientation() != turn) split_->setOrientation(turn);
}

bool ScriptPanel::eventFilter(QObject* watched, QEvent* event) {
  if (watched == output_->viewport() &&
      event->type() == QEvent::MouseButtonRelease) {
    const auto* mouse = static_cast<QMouseEvent*>(event);
    const int block =
        output_->cursorForPosition(mouse->position().toPoint()).blockNumber();
    if (block >= 0 && block < static_cast<int>(output_lines_.size()) &&
        output_lines_.at(block) > 0) {
      GoToLine(output_lines_.at(block));
    }
  }
  return QWidget::eventFilter(watched, event);
}

void ScriptPanel::GoToLine(int line) {
  editor_->MarkLine(line);
  editor_->setFocus();
}

void ScriptPanel::Load(const QString& file) {
  QString text;
  if (!file.isEmpty()) {
    try {
      text = ToQ(App::Get().session().script_read(ToRust(file)));
    } catch (const rust::Error& error) {
      App::Get().ShowError(QString::fromUtf8(error.what()));
    }
  }
  loading_ = true;
  editor_->setPlainText(text);
  loading_ = false;
  open_ = file;
  editor_->setEnabled(!file.isEmpty());
  editor_->setPlaceholderText(
      file.isEmpty()
          ? QStringLiteral("Press New to make a script in the project's "
                           "scripts folder.")
          : QString());
  Highlight();
}

void ScriptPanel::Rename() {
  if (open_.isEmpty()) return;
  Save();
  const QString current = open_.section(QLatin1Char('/'), -1);
  bool chosen = false;
  const QString name = QInputDialog::getText(
      this, QStringLiteral("Rename script"), QStringLiteral("Name"),
      QLineEdit::Normal, current, &chosen);
  if (!chosen || name.trimmed().isEmpty() || name.trimmed() == current) return;
  try {
    App::Get().session().script_rename(ToRust(dir_), ToRust(open_),
                                       ToRust(name.trimmed()));
  } catch (const rust::Error& error) {
    App::Get().ShowError(QString::fromUtf8(error.what()));
  }
}

void ScriptPanel::Save() {
  save_.stop();
  if (open_.isEmpty()) return;
  try {
    App::Get().session().script_save(ToRust(dir_), ToRust(open_),
                                     ToRust(editor_->toPlainText()));
  } catch (const rust::Error& error) {
    App::Get().ShowError(QString::fromUtf8(error.what()));
  }
}

void ScriptPanel::Run() {
  if (open_.isEmpty()) return;
  save_.stop();
  try {
    App::Get().session().script_run(ToRust(dir_), ToRust(open_),
                                    ToRust(editor_->toPlainText()));
  } catch (const rust::Error& error) {
    App::Get().ShowError(QString::fromUtf8(error.what()));
  }
}

void ScriptPanel::Highlight() {
  std::vector<ScriptHighlighter::Range> ranges;
  for (const auto& range :
       App::Get().session().script_highlight(ToRust(editor_->toPlainText()))) {
    QTextCharFormat format;
    if (!range.color.empty()) {
      format.setForeground(QColor::fromString(ToQ(range.color)));
    }
    if (range.bold) format.setFontWeight(QFont::Bold);
    if (range.italic) format.setFontItalic(true);
    if (range.error) {
      format.setUnderlineStyle(QTextCharFormat::WaveUnderline);
      format.setUnderlineColor(QColor::fromString(error_colour_));
    }
    ranges.push_back({static_cast<int>(range.start),
                      static_cast<int>(range.length), format});
  }
  highlighter_->SetRanges(std::move(ranges));
}

void ScriptPanel::ApplyLook() {
  const bridge::ScriptLookView look = App::Get().session().script_look();
  const QString sheet = LookSheet(look);
  editor_->setStyleSheet(sheet);
  output_->setStyleSheet(sheet);
  error_colour_ = ToQ(look.error);
  warning_colour_ = ToQ(look.warning);
  editor_->SetMarks(QColor::fromString(ToQ(look.bracket)),
                    QColor::fromString(error_colour_));
  theme_->blockSignals(true);
  theme_->setCurrentIndex(theme_->findData(ToQ(look.theme)));
  theme_->blockSignals(false);
  size_->blockSignals(true);
  size_->setRange(static_cast<int>(look.font_min),
                  static_cast<int>(look.font_max));
  size_->setValue(static_cast<int>(look.font_size));
  size_->blockSignals(false);
  Highlight();
  last_line_ = 0;
  if (last_ != nullptr) ShowOutput(*last_);
}

void ScriptPanel::ShowOutput(const bridge::ScriptView& view) {
  const std::uint64_t first = view.output.empty() ? 0 : view.output.front().id;
  const std::uint64_t newest = view.output.empty() ? 0 : view.output.back().id;
  const bool append = first <= last_line_ + 1 && newest >= last_line_ &&
                      last_line_ > 0 && first > 0;
  QScrollBar* bar = output_->verticalScrollBar();
  const bool at_end = bar->value() >= bar->maximum() - 4;
  if (!append) {
    output_->clear();
    output_lines_.clear();
  }
  const QString warn = warning_colour_;
  const QString error = error_colour_;
  for (const auto& line : view.output) {
    if (append && line.id <= last_line_) continue;
    const QString kind = ToQ(line.kind);
    const QString colour = kind == QStringLiteral("error")  ? error
                           : kind == QStringLiteral("warn") ? warn
                                                            : QString();
    const QString text = ToQ(line.text).toHtmlEscaped();
    output_->appendHtml(KeepSpaces(
        colour.isEmpty() ? text
                         : QStringLiteral("<span style=\"color:%1\">%2</span>")
                               .arg(colour, text)));
    const bool here = ToQ(line.file) == open_;
    output_lines_.push_back(here ? static_cast<int>(line.line) : 0);
  }
  if (view.output.empty() && !view.running && view.outcome.empty()) {
    output_->setPlaceholderText(
        QStringLiteral("What the script prints shows here."));
  }
  last_line_ = newest;
  if (at_end || !append) bar->setValue(bar->maximum());
}

void ScriptPanel::Rebuild() {
  const bridge::ScriptView view = App::Get().session().script_view();
  const bool fresh_outcome =
      last_ == nullptr || last_->outcome != view.outcome ||
      last_->failure != view.failure || last_->running != view.running;
  files_->blockSignals(true);
  files_->clear();
  for (const auto& file : view.files) files_->addItem(ToQ(file));
  files_->setCurrentText(ToQ(view.open));
  files_->setEnabled(!view.files.empty());
  files_->blockSignals(false);
  const QString open = ToQ(view.open);
  const QString dir = ToQ(view.dir);
  if (open != open_ || dir != dir_) {
    const bool still_there =
        std::any_of(view.files.begin(), view.files.end(),
                    [this](const auto& file) { return ToQ(file) == open_; });
    if (save_.isActive() && dir == dir_ && still_there) {
      Save();
    } else {
      save_.stop();
    }
    dir_ = dir;
    Load(open);
  } else if (!open_.isEmpty()) {
    const bridge::TextChangeView reload = App::Get().session().script_reload(
        ToRust(open_), ToRust(editor_->toPlainText()));
    if (reload.handled) {
      const int at = editor_->textCursor().position();
      loading_ = true;
      editor_->setPlainText(ToQ(reload.text));
      loading_ = false;
      QTextCursor cursor = editor_->textCursor();
      cursor.setPosition(std::min(
          at, static_cast<int>(editor_->document()->characterCount() - 1)));
      editor_->setTextCursor(cursor);
      Highlight();
    }
  }
  new_->setEnabled(view.has_project);
  rename_->setEnabled(view.has_project && !open_.isEmpty());
  run_->setEnabled(view.has_project && !view.running && !open_.isEmpty());
  stop_->setEnabled(view.running);
  badge_->setVisible(view.running);
  status_->setText(Outcome(view));
  ShowOutput(view);
  if (fresh_outcome && !view.running &&
      ToQ(view.outcome) == QStringLiteral("failed") &&
      ToQ(view.failure_file) == open_ && view.failure_line > 0) {
    editor_->MarkLine(static_cast<int>(view.failure_line));
  }
  last_ = std::make_unique<bridge::ScriptView>(view);
}

}
