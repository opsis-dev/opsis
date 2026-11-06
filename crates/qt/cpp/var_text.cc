#include "var_text.h"

#include <QAbstractTextDocumentLayout>
#include <QFocusEvent>
#include <QFontMetricsF>
#include <QHBoxLayout>
#include <QHelpEvent>
#include <QKeyEvent>
#include <QMenu>
#include <QMimeData>
#include <QPainter>
#include <QPushButton>
#include <QScrollBar>
#include <QTextBlock>
#include <QTextCursor>
#include <QTextDocument>
#include <QTextFragment>
#include <QToolButton>
#include <QToolTip>
#include <QVBoxLayout>
#include <QWidgetAction>

#include "app.h"
#include "opsis-qt/src/bridge.rs.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr int kChipType = QTextFormat::UserObject + 1;
constexpr int kChipName = QTextFormat::UserProperty + 1;
constexpr int kChipPause = QTextFormat::UserProperty + 2;
constexpr int kChipMissing = QTextFormat::UserProperty + 3;
constexpr double kChipPadX = 6.0;
constexpr double kChipMarginX = 2.0;
constexpr double kChipRadius = 5.0;
constexpr int kChipFontPx = 12;
constexpr int kChipFillAlpha = 38;
constexpr int kSingleMinHeight = 32;
constexpr int kMultiMinHeight = 76;
constexpr int kFramePad = 4;
constexpr int kValueShown = 24;

const QChar kObject = QChar::ObjectReplacementCharacter;

QFont ChipFont() {
  QFont font(QStringLiteral("Inter"));
  font.setPixelSize(kChipFontPx);
  font.setWeight(QFont::Medium);
  return font;
}

QString ChipText(const QTextFormat& format) {
  return format.boolProperty(kChipPause) ? QStringLiteral("pause")
                                         : format.stringProperty(kChipName);
}

std::vector<bridge::VariableView> Variables() {
  std::vector<bridge::VariableView> list;
  for (const auto& variable : App::Get().session().variables()) {
    list.push_back(variable);
  }
  return list;
}

QTextCharFormat ChipFormat(const bridge::TextToken& token,
                           const std::vector<bridge::VariableView>& vars) {
  QTextCharFormat format;
  format.setObjectType(kChipType);
  if (token.kind == bridge::TokenKind::Pause) {
    format.setProperty(kChipPause, true);
    format.setToolTip(QStringLiteral("Holds still for a moment while typing"));
    return format;
  }
  const QString name = ToQ(token.text);
  format.setProperty(kChipName, name);
  const auto found = std::find_if(
      vars.begin(), vars.end(),
      [&name](const auto& variable) { return ToQ(variable.name) == name; });
  if (found == vars.end()) {
    format.setProperty(kChipMissing, true);
    format.setToolTip(QStringLiteral("This variable doesn't exist"));
  } else {
    const QString value = ToQ(found->value);
    format.setToolTip(value.isEmpty() ? QStringLiteral("(empty)") : value);
  }
  return format;
}

}

ChipObject::ChipObject(QObject* parent) : QObject(parent) {}

QSizeF ChipObject::intrinsicSize(QTextDocument* document, int position,
                                 const QTextFormat& format) {
  Q_UNUSED(position)
  const QFontMetricsF chip(ChipFont());
  const QFontMetricsF line(document->defaultFont());
  return {
      chip.horizontalAdvance(ChipText(format)) + 2 * (kChipPadX + kChipMarginX),
      line.height()};
}

void ChipObject::drawObject(QPainter* painter, const QRectF& rect,
                            QTextDocument* document, int position,
                            const QTextFormat& format) {
  Q_UNUSED(document)
  Q_UNUSED(position)
  App& app = App::Get();
  QColor ink = format.boolProperty(kChipMissing) ? app.Token("destructive")
                                                 : app.Token("primary");
  QColor fill = ink;
  fill.setAlpha(kChipFillAlpha);
  const QRectF body = rect.adjusted(kChipMarginX, 1, -kChipMarginX, -1);
  painter->save();
  painter->setRenderHint(QPainter::Antialiasing);
  painter->setPen(Qt::NoPen);
  painter->setBrush(fill);
  painter->drawRoundedRect(body, kChipRadius, kChipRadius);
  painter->setFont(ChipFont());
  painter->setPen(ink);
  painter->drawText(body, Qt::AlignCenter, ChipText(format));
  painter->restore();
}

VarSurface::VarSurface(bool multiline, bool pauses, QWidget* parent)
    : QTextEdit(parent), multiline_(multiline), pauses_(pauses) {
  setProperty("role", "var");
  setAcceptRichText(false);
  setTabChangesFocus(true);
  setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
  setVerticalScrollBarPolicy(multiline ? Qt::ScrollBarAsNeeded
                                       : Qt::ScrollBarAlwaysOff);
  setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Fixed);
  document()->setDocumentMargin(kFramePad);
  document()->documentLayout()->registerHandler(kChipType,
                                                new ChipObject(this));
  connect(document()->documentLayout(),
          &QAbstractTextDocumentLayout::documentSizeChanged, this,
          &VarSurface::FitHeight);
  FitHeight();
}

void VarSurface::SetValue(const QString& value) {
  const QSignalBlocker blocker(this);
  clear();
  QTextCursor cursor(document());
  const auto vars = Variables();
  for (const auto& token :
       App::Get().session().text_tokens(ToRust(value), pauses_)) {
    if (token.kind == bridge::TokenKind::Text) {
      QString text = ToQ(token.text);
      if (!multiline_) text.replace(QLatin1Char('\n'), QLatin1Char(' '));
      cursor.insertText(text, QTextCharFormat());
    } else {
      cursor.insertText(QString(kObject), ChipFormat(token, vars));
    }
  }
  committed_ = Value();
  FitHeight();
}

QString VarSurface::Value() const {
  rust::Vec<bridge::TextToken> tokens;
  const auto push_text = [&tokens](const QString& text) {
    if (text.isEmpty()) return;
    tokens.push_back(bridge::TextToken{bridge::TokenKind::Text, ToRust(text)});
  };
  bool first_block = true;
  for (QTextBlock block = document()->begin(); block.isValid();
       block = block.next()) {
    if (!first_block) push_text(QStringLiteral("\n"));
    first_block = false;
    for (auto it = block.begin(); !it.atEnd(); ++it) {
      const QTextFragment fragment = it.fragment();
      if (!fragment.isValid()) continue;
      const QTextCharFormat format = fragment.charFormat();
      if (format.objectType() != kChipType) {
        push_text(fragment.text());
        continue;
      }
      for (int i = 0; i < fragment.length(); ++i) {
        if (format.boolProperty(kChipPause)) {
          tokens.push_back(bridge::TextToken{bridge::TokenKind::Pause, {}});
        } else {
          tokens.push_back(
              bridge::TextToken{bridge::TokenKind::Var,
                                ToRust(format.stringProperty(kChipName))});
        }
      }
    }
  }
  return ToQ(App::Get().session().join_tokens(std::move(tokens)));
}

int VarSurface::MissingChips() const {
  int missing = 0;
  for (QTextBlock block = document()->begin(); block.isValid();
       block = block.next()) {
    for (auto it = block.begin(); !it.atEnd(); ++it) {
      const QTextCharFormat format = it.fragment().charFormat();
      if (format.objectType() == kChipType &&
          format.boolProperty(kChipMissing)) {
        missing += it.fragment().length();
      }
    }
  }
  return missing;
}

void VarSurface::InsertVariable(const QString& name) {
  bridge::TextToken token{bridge::TokenKind::Var, ToRust(name)};
  textCursor().insertText(QString(kObject), ChipFormat(token, Variables()));
  setFocus();
}

void VarSurface::InsertPause() {
  bridge::TextToken token{bridge::TokenKind::Pause, {}};
  textCursor().insertText(QString(kObject), ChipFormat(token, {}));
  setFocus();
}

void VarSurface::InsertText(const QString& text) {
  QTextCursor cursor = textCursor();
  const auto vars = Variables();
  QString value = text;
  if (!multiline_) {
    value = value.simplified();
  }
  for (const auto& token :
       App::Get().session().text_tokens(ToRust(value), pauses_)) {
    if (token.kind == bridge::TokenKind::Text) {
      cursor.insertText(ToQ(token.text), QTextCharFormat());
    } else {
      cursor.insertText(QString(kObject), ChipFormat(token, vars));
    }
  }
  setTextCursor(cursor);
}

bool VarSurface::event(QEvent* event) {
  if (event->type() == QEvent::ToolTip) {
    const auto* help = static_cast<QHelpEvent*>(event);
    const QPointF at =
        QPointF(help->pos()) +
        QPointF(horizontalScrollBar()->value(), verticalScrollBar()->value());
    const int position =
        document()->documentLayout()->hitTest(at, Qt::ExactHit);
    if (position >= 0) {
      QTextCursor cursor(document());
      cursor.setPosition(position);
      cursor.movePosition(QTextCursor::NextCharacter, QTextCursor::KeepAnchor);
      const QTextCharFormat format = cursor.charFormat();
      if (format.objectType() == kChipType) {
        QToolTip::showText(help->globalPos(), format.toolTip(), this);
        return true;
      }
    }
    QToolTip::hideText();
    return true;
  }
  return QTextEdit::event(event);
}

void VarSurface::keyPressEvent(QKeyEvent* event) {
  const bool enter =
      event->key() == Qt::Key_Return || event->key() == Qt::Key_Enter;
  if (enter && !multiline_) {
    Commit();
    return;
  }
  QTextEdit::keyPressEvent(event);
}

void VarSurface::focusOutEvent(QFocusEvent* event) {
  QTextEdit::focusOutEvent(event);
  Commit();
}

bool VarSurface::canInsertFromMimeData(const QMimeData* source) const {
  return source->hasText();
}

void VarSurface::insertFromMimeData(const QMimeData* source) {
  if (source->hasText()) InsertText(source->text());
}

void VarSurface::FitHeight() {
  const int content =
      static_cast<int>(document()->documentLayout()->documentSize().height()) +
      2 * frameWidth();
  const int least = multiline_ ? kMultiMinHeight : kSingleMinHeight;
  setFixedHeight(std::max(least, content));
}

void VarSurface::Commit() {
  const QString value = Value();
  if (value == committed_) return;
  committed_ = value;
  emit Committed(value);
}

VarText::VarText(const QString& value, bool multiline, bool pauses,
                 const QString& placeholder, QWidget* parent)
    : QWidget(parent), surface_(new VarSurface(multiline, pauses, this)) {
  surface_->setPlaceholderText(placeholder);
  surface_->SetValue(value);
  connect(surface_, &VarSurface::Committed, this, &VarText::Committed);
  if (!multiline) {
    auto* row = new QHBoxLayout(this);
    row->setContentsMargins(0, 0, 0, 0);
    row->setSpacing(4);
    row->addWidget(surface_, 1);
    auto* insert = MakeIconButton(QStringLiteral("braces"),
                                  QStringLiteral("Insert variable"), this);
    insert->setFocusPolicy(Qt::NoFocus);
    row->addWidget(insert, 0, Qt::AlignTop);
    connect(insert, &QToolButton::clicked, this,
            [this, insert] { ShowVariables(insert); });
    return;
  }
  auto* column = new QVBoxLayout(this);
  column->setContentsMargins(0, 0, 0, 0);
  column->setSpacing(6);
  column->addWidget(surface_);
  auto* footer = new QHBoxLayout();
  footer->setSpacing(4);
  auto* insert = MakeButton(QStringLiteral("Insert variable"), "ghost",
                            QStringLiteral("braces"), this);
  insert->setFocusPolicy(Qt::NoFocus);
  footer->addWidget(insert);
  connect(insert, &QPushButton::clicked, this,
          [this, insert] { ShowVariables(insert); });
  if (pauses) {
    auto* pause = MakeButton(QStringLiteral("Add pause"), "ghost",
                             QStringLiteral("pause"), this);
    pause->setFocusPolicy(Qt::NoFocus);
    footer->addWidget(pause);
    connect(pause, &QPushButton::clicked, surface_, &VarSurface::InsertPause);
  }
  footer->addStretch(1);
  column->addLayout(footer);
}

void VarText::ShowVariables(QWidget* anchor) {
  QMenu menu(this);
  menu.addSection(QStringLiteral("Variables"));
  const auto vars = App::Get().session().variables();
  if (vars.empty()) {
    menu.addAction(QStringLiteral("No variables yet"))->setEnabled(false);
  }
  for (const auto& variable : vars) {
    const QString name = ToQ(variable.name);
    QString value = ToQ(variable.value);
    if (value.size() > kValueShown)
      value = value.left(kValueShown) + QChar(0x2026);
    QAction* action = menu.addAction(
        value.isEmpty() ? name : name + QStringLiteral("    ") + value);
    connect(action, &QAction::triggered, surface_,
            [this, name] { surface_->InsertVariable(name); });
  }
  menu.exec(anchor->mapToGlobal(QPoint(0, anchor->height() + 4)));
}

}
