#include "inspector_panel.h"

#include <QApplication>
#include <QComboBox>
#include <QDoubleValidator>
#include <QHBoxLayout>
#include <QJsonDocument>
#include <QJsonValue>
#include <QLineEdit>
#include <QLocale>
#include <QPlainTextEdit>
#include <QPushButton>
#include <QScrollBar>
#include <algorithm>
#include <cmath>
#include <functional>

#include "app.h"
#include "event_hub.h"
#include "opsis-qt/src/bridge.rs.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr int kSegmentLimit = 3;

QString Json(const QJsonValue& value) {
  return QString::fromUtf8(
      QJsonDocument::fromVariant(QVariantList{value.toVariant()})
          .toJson(QJsonDocument::Compact)
          .mid(1)
          .chopped(1));
}

class TextArea : public QPlainTextEdit {
 public:
  using QPlainTextEdit::QPlainTextEdit;
  std::function<void()> on_commit;

 protected:
  void focusOutEvent(QFocusEvent* event) override {
    QPlainTextEdit::focusOutEvent(event);
    if (on_commit) on_commit();
  }
};

}

InspectorPanel::InspectorPanel(QWidget* parent) : QWidget(parent) {
  setObjectName(QStringLiteral("Panel"));
  setAttribute(Qt::WA_StyledBackground);
  setMinimumWidth(300);
  auto* layout = new QVBoxLayout(this);
  layout->setContentsMargins(0, 0, 0, 0);
  scroll_ = new QScrollArea(this);
  scroll_->setWidgetResizable(true);
  scroll_->setFrameShape(QFrame::NoFrame);
  layout->addWidget(scroll_);
  connect(EventHub::Instance(), &EventHub::SelectionChanged, this,
          &InspectorPanel::Rebuild);
  connect(EventHub::Instance(), &EventHub::DocumentChanged, this, [this] {
    if (!Editing()) Rebuild();
  });
  connect(EventHub::Instance(), &EventHub::ProjectChanged, this,
          &InspectorPanel::Rebuild);
  connect(qApp, &QApplication::focusChanged, this, [this](QWidget*, QWidget*) {
    App::Get().session().report_drafts(Editing());
  });
  connect(EventHub::Instance(), &EventHub::FlushDrafts, this, [] {
    if (QWidget* focus = QApplication::focusWidget()) focus->clearFocus();
    App::Get().session().save_flushed();
  });
  Rebuild();
}

bool InspectorPanel::Editing() const {
  QWidget* focus = QApplication::focusWidget();
  return focus != nullptr && content_ != nullptr &&
         content_->isAncestorOf(focus) &&
         (qobject_cast<QLineEdit*>(focus) != nullptr ||
          qobject_cast<QPlainTextEdit*>(focus) != nullptr);
}

void InspectorPanel::Commit(const QString& key, const QString& json) {
  if (node_.isEmpty()) return;
  try {
    App::Get().session().set_param(ToRust(node_), ToRust(key), ToRust(json));
  } catch (const rust::Error& error) {
    App::Get().ShowError(QString::fromUtf8(error.what()));
  }
}

QWidget* InspectorPanel::MakeControl(const bridge::FieldView& field) {
  const QString key = ToQ(field.key);
  switch (field.kind) {
    case bridge::FieldKind::Switch: {
      auto* toggle = new ToggleSwitch();
      toggle->SetCheckedQuietly(field.checked);
      connect(toggle, &ToggleSwitch::toggled, this, [this, key](bool on) {
        Commit(key, on ? QStringLiteral("true") : QStringLiteral("false"));
      });
      return toggle;
    }
    case bridge::FieldKind::Choice: {
      if (field.choices.size() <= kSegmentLimit) {
        auto* segmented = new Segmented();
        std::vector<std::pair<QString, QString>> choices;
        for (const auto& choice : field.choices) {
          choices.emplace_back(ToQ(choice.value), ToQ(choice.label));
        }
        segmented->SetChoices(choices);
        segmented->SetValue(ToQ(field.text));
        connect(segmented, &Segmented::Changed, this,
                [this, key](const QString& value) {
                  Commit(key, Json(QJsonValue(value)));
                });
        auto* holder = new QWidget();
        auto* row = new QHBoxLayout(holder);
        row->setContentsMargins(0, 0, 0, 0);
        row->addWidget(segmented);
        row->addStretch(1);
        return holder;
      }
      auto* combo = new QComboBox();
      for (const auto& choice : field.choices) {
        combo->addItem(ToQ(choice.label), ToQ(choice.value));
      }
      combo->setCurrentIndex(combo->findData(ToQ(field.text)));
      connect(
          combo, &QComboBox::activated, this, [this, key, combo](int index) {
            Commit(key, Json(QJsonValue(combo->itemData(index).toString())));
          });
      return combo;
    }
    case bridge::FieldKind::Number: {
      auto* holder = new QWidget();
      auto* row = new QHBoxLayout(holder);
      row->setContentsMargins(0, 0, 0, 0);
      row->setSpacing(8);
      auto* line = new QLineEdit();
      const int decimals = static_cast<int>(field.decimals);
      auto* validator =
          new QDoubleValidator(field.minimum, field.maximum, decimals, line);
      validator->setNotation(QDoubleValidator::StandardNotation);
      validator->setLocale(QLocale::c());
      line->setValidator(validator);
      const QString zero = ToQ(field.zero_label);
      const bool zero_shown = !zero.isEmpty() && field.number == 0.0;
      line->setText(zero_shown
                        ? QString()
                        : QLocale::c().toString(field.number, 'f', decimals));
      line->setPlaceholderText(zero);
      row->addWidget(line, 1);
      if (!field.unit.empty())
        row->addWidget(MakeLabel(ToQ(field.unit), "muted"));
      const double scale = field.scale;
      const bool whole = field.decimals == 0 || scale != 1.0;
      const double minimum = field.minimum;
      const double maximum = field.maximum;
      const QString initial = line->text();
      connect(line, &QLineEdit::editingFinished, this,
              [this, key, line, scale, whole, minimum, maximum, initial] {
                if (line->text() == initial) return;
                bool ok = false;
                const double typed =
                    line->text().isEmpty()
                        ? 0.0
                        : QLocale::c().toDouble(line->text(), &ok);
                if (!ok && !line->text().isEmpty()) return;
                const double raw = std::clamp(typed, minimum, maximum) * scale;
                Commit(key, whole ? QString::number(std::llround(raw))
                                  : QString::number(raw));
              });
      return holder;
    }
    case bridge::FieldKind::Textarea: {
      auto* area = new TextArea();
      area->setPlainText(ToQ(field.text));
      area->setFixedHeight(96);
      if (key == QStringLiteral("code") || key == QStringLiteral("script")) {
        area->setFont(QFont(QStringLiteral("JetBrains Mono"), 12));
      }
      const QString initial = ToQ(field.text);
      area->on_commit = [this, key, area, initial] {
        if (area->toPlainText() != initial) {
          Commit(key, Json(QJsonValue(area->toPlainText())));
        }
      };
      return area;
    }
    case bridge::FieldKind::Text: {
      auto* line = new QLineEdit(ToQ(field.text));
      const QString initial = ToQ(field.text);
      connect(line, &QLineEdit::editingFinished, this,
              [this, key, line, initial] {
                if (line->text() != initial) {
                  Commit(key, Json(QJsonValue(line->text())));
                }
              });
      return line;
    }
    default: {
      auto* text = MakeLabel(ToQ(field.text), "muted");
      text->setWordWrap(true);
      return text;
    }
  }
}

void InspectorPanel::Rebuild() {
  App& app = App::Get();
  const bridge::InspectorView view = app.session().inspector();
  node_ = ToQ(view.id);
  auto* content = new QWidget();
  auto* layout = new QVBoxLayout(content);
  layout->setContentsMargins(16, 14, 16, 16);
  layout->setSpacing(14);
  if (node_.isEmpty()) {
    layout->addStretch(1);
    auto* empty =
        MakeLabel(QStringLiteral("Select a node to change its settings."),
                  "muted", content);
    empty->setAlignment(Qt::AlignCenter);
    empty->setWordWrap(true);
    layout->addWidget(empty);
    layout->addStretch(2);
  } else {
    auto* header = new QHBoxLayout();
    header->setSpacing(10);
    header->addWidget(MakeBadge(
        ToQ(view.glyph), QColor::fromString(ToQ(view.color)), 28, content));
    auto* name = new QLineEdit(ToQ(view.label), content);
    name->setProperty("role", "flat");
    const QString node = node_;
    const QString initial = ToQ(view.label);
    connect(name, &QLineEdit::editingFinished, this, [node, name, initial] {
      if (name->text() == initial) return;
      try {
        App::Get().session().rename_node(ToRust(node), ToRust(name->text()));
      } catch (const rust::Error& error) {
        App::Get().ShowError(QString::fromUtf8(error.what()));
      }
    });
    header->addWidget(name, 1);
    layout->addLayout(header);
    auto* category =
        MakeLabel(ToQ(view.category) + QStringLiteral(" · ") + ToQ(view.title),
                  "muted", content);
    layout->addWidget(category);
    if (!view.description.empty()) {
      auto* about = MakeLabel(ToQ(view.description), "muted", content);
      about->setWordWrap(true);
      layout->addWidget(about);
    }
    layout->addWidget(MakeRule(content));
    QList<QWidget*> advanced;
    for (const auto& field : view.fields) {
      QWidget* control = MakeControl(field);
      QWidget* row =
          field.kind == bridge::FieldKind::Switch
              ? MakeSwitchField(ToQ(field.label),
                                static_cast<ToggleSwitch*>(control),
                                ToQ(field.help), content)
              : MakeField(ToQ(field.label), control, ToQ(field.help), content);
      if (field.advanced) {
        advanced.append(row);
      } else {
        layout->addWidget(row);
      }
    }
    if (!advanced.isEmpty()) {
      auto* toggle =
          MakeButton(QStringLiteral("Advanced"), "ghost",
                     advanced_open_ ? QStringLiteral("chevron-down")
                                    : QStringLiteral("chevron-right"),
                     content);
      toggle->setStyleSheet(
          QStringLiteral("text-align: left; padding-left: 4px;"));
      layout->addWidget(toggle);
      for (QWidget* row : advanced) {
        row->setVisible(advanced_open_);
        layout->addWidget(row);
      }
      connect(toggle, &QPushButton::clicked, this, [this, advanced, toggle] {
        advanced_open_ = !advanced_open_;
        toggle->setIcon(App::Get().Icon(advanced_open_
                                            ? QStringLiteral("chevron-down")
                                            : QStringLiteral("chevron-right"),
                                        App::Get().Token("foreground")));
        for (QWidget* row : advanced) row->setVisible(advanced_open_);
      });
    }
    layout->addStretch(1);
  }
  const int scroll = scroll_->verticalScrollBar() != nullptr
                         ? scroll_->verticalScrollBar()->value()
                         : 0;
  scroll_->setWidget(content);
  content_ = content;
  scroll_->verticalScrollBar()->setValue(scroll);
}

}
