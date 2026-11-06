#include "inspector_panel.h"

#include <QApplication>
#include <QComboBox>
#include <QDoubleValidator>
#include <QFrame>
#include <QHBoxLayout>
#include <QJsonDocument>
#include <QJsonValue>
#include <QLineEdit>
#include <QLocale>
#include <QPlainTextEdit>
#include <QPushButton>
#include <QScrollBar>
#include <QTextEdit>
#include <algorithm>
#include <cmath>
#include <functional>

#include "app.h"
#include "event_hub.h"
#include "opsis-qt/src/bridge.rs.h"
#include "project_panel.h"
#include "var_text.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr int kHeaderBadge = 40;
constexpr int kAlertIcon = 16;

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
  const auto refresh_project = [this] {
    if (node_.isEmpty() && !Editing()) Rebuild();
  };
  connect(EventHub::Instance(), &EventHub::CompiledChanged, this,
          refresh_project);
  connect(EventHub::Instance(), &EventHub::RunChanged, this, refresh_project);
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
          qobject_cast<QPlainTextEdit*>(focus) != nullptr ||
          qobject_cast<QTextEdit*>(focus) != nullptr);
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
      auto* combo = new Combo();
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
    case bridge::FieldKind::Textarea:
    case bridge::FieldKind::Text: {
      auto* text = new VarText(ToQ(field.text),
                               field.kind == bridge::FieldKind::Textarea,
                               field.pauses, ToQ(field.zero_label));
      connect(text, &VarText::Committed, this,
              [this, key](const QString& value) {
                Commit(key, Json(QJsonValue(value)));
              });
      return text;
    }
    case bridge::FieldKind::Code: {
      auto* area = new TextArea();
      area->setPlainText(ToQ(field.text));
      area->setFixedHeight(96);
      QFont mono(QStringLiteral("JetBrains Mono"));
      mono.setPixelSize(12);
      area->setFont(mono);
      const QString initial = ToQ(field.text);
      area->on_commit = [this, key, area, initial] {
        if (area->toPlainText() != initial) {
          Commit(key, Json(QJsonValue(area->toPlainText())));
        }
      };
      return area;
    }
    default: {
      auto* text = MakeLabel(ToQ(field.text), "muted");
      text->setWordWrap(true);
      return text;
    }
  }
}

void InspectorPanel::AddHeader(QVBoxLayout* layout,
                               const bridge::InspectorView& view,
                               QWidget* content) {
  auto* header = new QHBoxLayout();
  header->setSpacing(12);
  header->addWidget(
      MakeBadge(ToQ(view.glyph), QColor::fromString(ToQ(view.color)),
                kHeaderBadge, content),
      0, Qt::AlignTop);
  auto* names = new QVBoxLayout();
  names->setSpacing(4);
  auto* name = MakeLabel(ToQ(view.label), "heading2", content);
  names->addWidget(name);
  auto* badges = new QHBoxLayout();
  badges->setSpacing(4);
  auto* category = MakeLabel(ToQ(view.category), "badge-secondary", content);
  badges->addWidget(category);
  if (view.renamed)
    badges->addWidget(MakeLabel(ToQ(view.title), "badge", content));
  badges->addStretch(1);
  names->addLayout(badges);
  header->addLayout(names, 1);
  layout->addLayout(header);
  if (!view.description.empty()) {
    auto* about = MakeLabel(ToQ(view.description), "muted", content);
    about->setWordWrap(true);
    layout->addWidget(about);
  }
  AddProblems(layout, view.errors, true, content);
  AddProblems(layout, view.warnings, false, content);
  const QString node = node_;
  auto* label =
      new QLineEdit(view.renamed ? ToQ(view.label) : QString(), content);
  label->setPlaceholderText(ToQ(view.title));
  const QString initial = label->text();
  connect(label, &QLineEdit::editingFinished, this, [node, label, initial] {
    if (label->text() == initial) return;
    try {
      App::Get().session().rename_node(ToRust(node), ToRust(label->text()));
    } catch (const rust::Error& error) {
      App::Get().ShowError(QString::fromUtf8(error.what()));
    }
  });
  layout->addWidget(MakeField(
      QStringLiteral("Name"), label,
      ToQ(view.category_id) == QStringLiteral("Browser")
          ? QStringLiteral("Also used by the camera, chapters and captions.")
          : QString(),
      content));
  auto* skip = new ToggleSwitch(content);
  skip->SetCheckedQuietly(view.disabled);
  connect(skip, &ToggleSwitch::toggled, this, [node](bool on) {
    try {
      App::Get().session().set_skipped(ToRust(node), on);
    } catch (const rust::Error& error) {
      App::Get().ShowError(QString::fromUtf8(error.what()));
    }
  });
  layout->addWidget(MakeSwitchField(QStringLiteral("Skip this node"), skip,
                                    QString(), content));
}

void InspectorPanel::AddProblems(QVBoxLayout* layout,
                                 const rust::Vec<rust::String>& problems,
                                 bool errors, QWidget* content) {
  if (problems.empty()) return;
  auto* alert = new QFrame(content);
  alert->setProperty("role", "alert");
  alert->setProperty("tone", errors ? "destructive" : "default");
  auto* box = new QVBoxLayout(alert);
  box->setContentsMargins(12, 10, 12, 10);
  box->setSpacing(4);
  const auto count = problems.size();
  const QString title =
      errors
          ? (count == 1 ? QStringLiteral("Something needs attention")
                        : QStringLiteral("%1 things need attention").arg(count))
          : (count == 1 ? QStringLiteral("Something to check")
                        : QStringLiteral("%1 things to check").arg(count));
  auto* head = new QHBoxLayout();
  head->setSpacing(8);
  auto* icon = new QLabel(alert);
  icon->setPixmap(App::Get().Glyph(
      QStringLiteral("circle-alert"),
      App::Get().Token(errors ? "destructive" : "foreground"), kAlertIcon));
  head->addWidget(icon);
  head->addWidget(
      MakeLabel(title, errors ? "error-title" : "title-small", alert), 1);
  box->addLayout(head);
  for (const auto& problem : problems) {
    auto* line = MakeLabel(ToQ(problem), errors ? "error" : "muted", alert);
    line->setWordWrap(true);
    line->setContentsMargins(kAlertIcon + 8, 0, 0, 0);
    box->addWidget(line);
  }
  layout->addWidget(alert);
}

void InspectorPanel::AddActions(QVBoxLayout* layout,
                                const bridge::InspectorView& view,
                                QWidget* content) {
  const QString node = node_;
  auto* row = new QHBoxLayout();
  row->setSpacing(8);
  const auto guard = [](const std::function<void()>& action) {
    try {
      action();
    } catch (const rust::Error& error) {
      App::Get().ShowError(QString::fromUtf8(error.what()));
    }
  };
  if (view.runs) {
    auto* run = MakeButton(QStringLiteral("Run this"), "primary",
                           QStringLiteral("play"), content);
    run->setEnabled(!view.running);
    connect(run, &QPushButton::clicked, this, [node, guard] {
      guard([&] { App::Get().session().run_node(ToRust(node), false); });
    });
    row->addWidget(run);
    auto* again = MakeButton(QStringLiteral("Re-run from here"), nullptr,
                             QStringLiteral("rotate-ccw"), content);
    again->setEnabled(!view.running);
    connect(again, &QPushButton::clicked, this, [node, guard] {
      guard([&] { App::Get().session().run_node(ToRust(node), true); });
    });
    row->addWidget(again);
  }
  if (view.group) {
    auto* ungroup = MakeButton(QStringLiteral("Ungroup"), nullptr,
                               QStringLiteral("ungroup"), content);
    connect(ungroup, &QPushButton::clicked, this, [node, guard] {
      guard([&] { App::Get().session().ungroup(ToRust(node)); });
    });
    row->addWidget(ungroup);
  }
  auto* remove = MakeButton(QStringLiteral("Delete"), "ghost",
                            QStringLiteral("trash-2"), content);
  connect(remove, &QPushButton::clicked, this, [node, guard] {
    guard([&] { App::Get().session().delete_node(ToRust(node)); });
  });
  row->addWidget(remove);
  row->addStretch(1);
  layout->addLayout(row);
  if (view.files.empty()) return;
  layout->addWidget(MakeLabel(QStringLiteral("Files"), "title-small", content));
  for (const auto& file : view.files) {
    auto* link = MakeButton(ToQ(file.label), "link",
                            QStringLiteral("external-link"), content);
    const QString path = ToQ(file.path);
    connect(link, &QPushButton::clicked, this, [path, guard] {
      guard([&] { App::Get().session().reveal_file(ToRust(path)); });
    });
    layout->addWidget(link, 0, Qt::AlignLeft);
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
  if (node_.isEmpty() && app.session().has_project()) {
    layout->setContentsMargins(0, 0, 0, 0);
    layout->addWidget(new ProjectPanel(content));
  } else if (node_.isEmpty()) {
    layout->addStretch(1);
    auto* empty =
        MakeLabel(QStringLiteral("Select a node to change its settings."),
                  "muted", content);
    empty->setAlignment(Qt::AlignCenter);
    empty->setWordWrap(true);
    layout->addWidget(empty);
    layout->addStretch(2);
  } else {
    AddHeader(layout, view, content);
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
    layout->addWidget(MakeRule(content));
    AddActions(layout, view, content);
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
