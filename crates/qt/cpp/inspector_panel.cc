#include "inspector_panel.h"

#include <QApplication>
#include <QButtonGroup>
#include <QComboBox>
#include <QDateTimeEdit>
#include <QDoubleValidator>
#include <QFrame>
#include <QHBoxLayout>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonValue>
#include <QKeyEvent>
#include <QKeySequence>
#include <QLineEdit>
#include <QLocale>
#include <QPlainTextEdit>
#include <QPushButton>
#include <QScrollBar>
#include <QTextEdit>
#include <QTimeZone>
#include <algorithm>
#include <cmath>
#include <functional>
#include <memory>

#include "app.h"
#include "color_picker.h"
#include "event_hub.h"
#include "list_fields.h"
#include "opsis-qt/src/bridge.rs.h"
#include "project_panel.h"
#include "var_text.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr int kToggleGap = 6;
constexpr int kMinutePrecision = 16;
constexpr int kJsonHeight = 96;

constexpr int kActionGap = 8;

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

class KeyChordButton : public QPushButton {
 public:
  explicit KeyChordButton(const QString& chord) {
    setProperty("variant", "keys");
    setFocusPolicy(Qt::StrongFocus);
    Show(chord);
  }
  void SetOnChord(std::function<void(const QString&)> on_chord) {
    on_chord_ = std::move(on_chord);
  }

 protected:
  void keyPressEvent(QKeyEvent* event) override {
    if (event->key() == Qt::Key_Tab || event->key() == Qt::Key_Backtab) {
      QPushButton::keyPressEvent(event);
      return;
    }
    const QString chord = ChordOf(event);
    if (chord.isEmpty()) return;
    Show(chord);
    if (on_chord_) on_chord_(chord);
  }
  void focusInEvent(QFocusEvent* event) override {
    QPushButton::focusInEvent(event);
    setText(text() + QStringLiteral("   Listening…"));
  }
  void focusOutEvent(QFocusEvent* event) override {
    QPushButton::focusOutEvent(event);
    Show(chord_);
  }

 private:
  static QString ChordOf(const QKeyEvent* event) {
    const int key = event->key();
    if (key == Qt::Key_Control || key == Qt::Key_Shift || key == Qt::Key_Alt ||
        key == Qt::Key_Meta) {
      return {};
    }
    QStringList parts;
    const Qt::KeyboardModifiers mods = event->modifiers();
    if (mods.testFlag(Qt::ControlModifier)) parts << QStringLiteral("Control");
    if (mods.testFlag(Qt::MetaModifier)) parts << QStringLiteral("Meta");
    if (mods.testFlag(Qt::AltModifier)) parts << QStringLiteral("Alt");
    if (mods.testFlag(Qt::ShiftModifier)) parts << QStringLiteral("Shift");
    parts << KeyName(key, event->text());
    return parts.join(QLatin1Char('+'));
  }
  static QString KeyName(int key, const QString& typed) {
    switch (key) {
      case Qt::Key_Space:
        return QStringLiteral("Space");
      case Qt::Key_Return:
      case Qt::Key_Enter:
        return QStringLiteral("Enter");
      case Qt::Key_Escape:
        return QStringLiteral("Escape");
      case Qt::Key_Backspace:
        return QStringLiteral("Backspace");
      case Qt::Key_Delete:
        return QStringLiteral("Delete");
      case Qt::Key_Up:
        return QStringLiteral("ArrowUp");
      case Qt::Key_Down:
        return QStringLiteral("ArrowDown");
      case Qt::Key_Left:
        return QStringLiteral("ArrowLeft");
      case Qt::Key_Right:
        return QStringLiteral("ArrowRight");
      case Qt::Key_Home:
        return QStringLiteral("Home");
      case Qt::Key_End:
        return QStringLiteral("End");
      case Qt::Key_PageUp:
        return QStringLiteral("PageUp");
      case Qt::Key_PageDown:
        return QStringLiteral("PageDown");
      default:
        break;
    }
    if (key >= Qt::Key_F1 && key <= Qt::Key_F24) {
      return QStringLiteral("F%1").arg(key - Qt::Key_F1 + 1);
    }
    const QString name = QKeySequence(key).toString();
    if (name.size() == 1) return name.toUpper();
    return typed.isEmpty() ? name : typed.toUpper();
  }
  void Show(const QString& chord) {
    chord_ = chord;
    setText(chord.isEmpty() ? QStringLiteral("Press a key")
                            : QString(chord).replace(QLatin1Char('+'),
                                                     QStringLiteral("  +  ")));
  }

  QString chord_;
  std::function<void(const QString&)> on_chord_;
};

class TextArea : public QPlainTextEdit {
 public:
  using QPlainTextEdit::QPlainTextEdit;
  std::function<void()> on_commit;

 protected:
  void focusOutEvent(QFocusEvent* event) override {
    QPlainTextEdit::focusOutEvent(event);
    if (on_commit) on_commit();
    document()->setModified(false);
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
  scroll_->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
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
  if (ColorPicker::IsOpen()) return true;
  QWidget* focus = window() != nullptr ? window()->focusWidget() : nullptr;
  if (focus == nullptr || content_ == nullptr ||
      !content_->isAncestorOf(focus)) {
    return false;
  }
  if (auto* line = qobject_cast<QLineEdit*>(focus)) return line->isModified();
  if (auto* surface = qobject_cast<VarSurface*>(focus))
    return surface->HasDraft();
  if (auto* area = qobject_cast<QPlainTextEdit*>(focus)) {
    return area->document()->isModified();
  }
  return false;
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
        const bool raw = field.raw_choices;
        connect(segmented, &Segmented::Changed, this,
                [this, key, raw](const QString& value) {
                  Commit(key, raw ? value : Json(QJsonValue(value)));
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
      const bool raw = field.raw_choices;
      connect(combo, &QComboBox::activated, this,
              [this, key, combo, raw](int index) {
                const QString value = combo->itemData(index).toString();
                Commit(key, raw ? value : Json(QJsonValue(value)));
              });
      return combo;
    }
    case bridge::FieldKind::Select: {
      auto* combo = new Combo();
      const QString none = ToQ(field.none_label);
      if (!none.isEmpty() || field.text.empty()) {
        combo->addItem(none.isEmpty() ? QStringLiteral("Choose…") : none,
                       QString());
      }
      for (const auto& choice : field.choices) {
        combo->addItem(ToQ(choice.label), ToQ(choice.value));
      }
      combo->setCurrentIndex(std::max(0, combo->findData(ToQ(field.text))));
      connect(combo, &QComboBox::activated, this,
              [this, key, combo](int index) {
                const QString value = combo->itemData(index).toString();
                Commit(key, value.isEmpty() ? QStringLiteral("null")
                                            : Json(QJsonValue(value)));
              });
      return combo;
    }
    case bridge::FieldKind::EnumList: {
      auto* holder = new QWidget();
      auto* flow = new FlowLayout(holder, kToggleGap);
      const QJsonArray chosen =
          QJsonDocument::fromJson(ToQ(field.value_json).toUtf8()).array();
      auto buttons =
          std::make_shared<std::vector<std::pair<QString, QPushButton*>>>();
      for (const auto& choice : field.choices) {
        auto* button = new QPushButton(ToQ(choice.label), holder);
        button->setProperty("variant", "toggle-outline");
        button->setCheckable(true);
        button->setChecked(chosen.contains(QJsonValue(ToQ(choice.value))));
        flow->addWidget(button);
        buttons->emplace_back(ToQ(choice.value), button);
        connect(button, &QPushButton::clicked, this, [this, key, buttons] {
          QJsonArray picked;
          for (const auto& [value, toggle] : *buttons) {
            if (toggle->isChecked()) picked.append(value);
          }
          Commit(key, QString::fromUtf8(QJsonDocument(picked).toJson(
                          QJsonDocument::Compact)));
        });
      }
      return holder;
    }
    case bridge::FieldKind::PresetNumber: {
      auto* holder = new QWidget();
      auto* flow = new FlowLayout(holder, kToggleGap);
      auto* group = new QButtonGroup(holder);
      int nearest = -1;
      double best = 0;
      int index = 0;
      for (const auto& choice : field.choices) {
        auto* button = new QPushButton(ToQ(choice.label), holder);
        button->setProperty("variant", "toggle-outline");
        button->setCheckable(true);
        group->addButton(button, index);
        flow->addWidget(button);
        const double distance =
            std::abs(ToQ(choice.value).toDouble() - field.number);
        if (nearest < 0 || distance < best) {
          nearest = index;
          best = distance;
        }
        const QString value = ToQ(choice.value);
        connect(button, &QPushButton::clicked, this,
                [this, key, value] { Commit(key, value); });
        ++index;
      }
      if (nearest >= 0) group->button(nearest)->setChecked(true);
      return holder;
    }
    case bridge::FieldKind::DateTime: {
      auto* edit = new QDateTimeEdit();
      edit->setDisplayFormat(QStringLiteral("yyyy-MM-dd HH:mm"));
      edit->setCalendarPopup(true);
      const QDateTime at =
          QDateTime::fromString(ToQ(field.text).left(kMinutePrecision),
                                QStringLiteral("yyyy-MM-ddTHH:mm"));
      edit->setDateTime(at.isValid() ? at : QDateTime::currentDateTime());
      connect(edit, &QDateTimeEdit::editingFinished, this, [this, key, edit] {
        Commit(key, Json(QJsonValue(edit->dateTime().toString(
                        QStringLiteral("yyyy-MM-ddTHH:mm:00")))));
      });
      return edit;
    }
    case bridge::FieldKind::Timezone: {
      auto* combo = new Combo();
      combo->addItem(QStringLiteral("This computer's time zone"), QString());
      for (const QByteArray& zone : QTimeZone::availableTimeZoneIds()) {
        combo->addItem(QString::fromUtf8(zone), QString::fromUtf8(zone));
      }
      combo->setCurrentIndex(std::max(0, combo->findData(ToQ(field.text))));
      connect(combo, &QComboBox::activated, this,
              [this, key, combo](int index) {
                const QString zone = combo->itemData(index).toString();
                Commit(key, zone.isEmpty() ? QStringLiteral("null")
                                           : Json(QJsonValue(zone)));
              });
      return combo;
    }
    case bridge::FieldKind::Keys: {
      auto* keys = new KeyChordButton(ToQ(field.text));
      keys->SetOnChord([this, key](const QString& chord) {
        Commit(key, Json(QJsonValue(chord)));
      });
      return keys;
    }
    case bridge::FieldKind::StringList:
      return MakeStringList(
          field, [this, key](const QString& json) { Commit(key, json); });
    case bridge::FieldKind::NumberList:
      return MakeNumberList(
          field, [this, key](const QString& json) { Commit(key, json); });
    case bridge::FieldKind::ColorList:
      return MakeColorList(
          field, swatches_,
          [this, key](const QString& json) { Commit(key, json); },
          [this] { Rebuild(); });
    case bridge::FieldKind::Color: {
      auto* input = new ColorInput(ToQ(field.label), ToQ(field.text),
                                   ToQ(field.zero_label), swatches_);
      connect(input, &ColorInput::Changed, this,
              [this, key](const QString& value) {
                Commit(key, value.isEmpty() ? QStringLiteral("\"\"")
                                            : Json(QJsonValue(value)));
              });
      connect(input, &ColorInput::PickerClosed, this, &InspectorPanel::Rebuild);
      return input;
    }
    case bridge::FieldKind::ObjectList:
      return MakeObjectList(
          field, [this, key](const QString& json) { Commit(key, json); },
          &closed_items_, &bodies_);
    case bridge::FieldKind::Json: {
      auto* area = new TextArea();
      area->setPlainText(ToQ(field.value_json));
      area->setFixedHeight(kJsonHeight);
      QFont mono(QStringLiteral("JetBrains Mono"));
      mono.setPixelSize(12);
      area->setFont(mono);
      const QString initial = ToQ(field.value_json);
      area->on_commit = [this, key, area, initial] {
        const QString text = area->toPlainText().trimmed();
        if (text == initial) return;
        QJsonParseError error{};
        QJsonDocument::fromJson(QByteArray("[") + text.toUtf8() + "]", &error);
        if (error.error != QJsonParseError::NoError) {
          App::Get().ShowError(QStringLiteral("That isn't valid JSON."));
          return;
        }
        Commit(key, text);
      };
      return area;
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
      if (field.clearable) {
        auto* clear = MakeButton(QStringLiteral("Clear"), "ghost", QString());
        connect(clear, &QPushButton::clicked, this,
                [this, key] { Commit(key, QStringLiteral("null")); });
        row->addWidget(clear);
      }
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
  bool first = true;
  for (const auto& problem : problems) {
    auto* row = new QHBoxLayout();
    row->setSpacing(8);
    auto* icon = new QLabel(alert);
    if (first) {
      icon->setPixmap(App::Get().Glyph(
          QStringLiteral("circle-alert"),
          App::Get().Token(errors ? "destructive" : "foreground"), kAlertIcon));
    }
    icon->setFixedWidth(kAlertIcon);
    row->addWidget(icon, 0, Qt::AlignTop);
    auto* line = MakeLabel(ToQ(problem), errors ? "error" : "muted", alert);
    line->setWordWrap(true);
    row->addWidget(line, 1);
    box->addLayout(row);
    first = false;
  }
  layout->addWidget(alert);
}

void InspectorPanel::AddActions(QVBoxLayout* layout,
                                const bridge::InspectorView& view,
                                QWidget* content) {
  const QString node = node_;
  auto* actions = new QWidget(content);
  auto* row = new FlowLayout(actions, kActionGap);
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
  layout->addWidget(actions);
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

QWidget* InspectorPanel::WithPresets(const bridge::FieldView& field,
                                     QWidget* row, QWidget* content) {
  auto* box = new QWidget(content);
  auto* column = new QVBoxLayout(box);
  column->setContentsMargins(0, 0, 0, 0);
  column->setSpacing(6);
  if (!field.presets_label.empty()) {
    column->addWidget(MakeLabel(ToQ(field.presets_label), "small", box));
  }
  auto* holder = new QWidget(box);
  auto* flow = new FlowLayout(holder, kToggleGap);
  const QString node = node_;
  for (const auto& preset : field.presets) {
    auto* button =
        MakeButton(ToQ(preset.label), "toggle-outline", QString(), holder);
    const QString values = ToQ(preset.values_json);
    connect(button, &QPushButton::clicked, this, [node, values] {
      try {
        App::Get().session().set_params(ToRust(node), ToRust(values));
      } catch (const rust::Error& error) {
        App::Get().ShowError(QString::fromUtf8(error.what()));
      }
    });
    flow->addWidget(button);
  }
  column->addWidget(holder);
  column->addWidget(row);
  return box;
}

void InspectorPanel::AddTabs(QVBoxLayout* layout, const QList<QWidget*>& basic,
                             const QList<QWidget*>& advanced,
                             QWidget* content) {
  auto* bar = new QFrame(content);
  bar->setObjectName(QStringLiteral("FormTabs"));
  auto* tabs = new QHBoxLayout(bar);
  tabs->setContentsMargins(3, 3, 3, 3);
  tabs->setSpacing(3);
  auto* basic_tab = MakeButton(QStringLiteral("Basic"), "tab", QString(), bar);
  auto* advanced_tab =
      MakeButton(QStringLiteral("Advanced  %1").arg(advanced.size()), "tab",
                 QString(), bar);
  for (QPushButton* tab : {basic_tab, advanced_tab}) {
    tab->setCheckable(true);
    tab->setAutoExclusive(true);
    tabs->addWidget(tab, 1);
  }
  layout->addWidget(bar);
  auto* basic_page = new QWidget(content);
  auto* basic_rows = new QVBoxLayout(basic_page);
  basic_rows->setContentsMargins(0, 0, 0, 0);
  basic_rows->setSpacing(14);
  if (basic.isEmpty()) {
    auto* empty =
        MakeLabel(QStringLiteral("Every setting of this node is in Advanced."),
                  "muted", basic_page);
    empty->setWordWrap(true);
    basic_rows->addWidget(empty);
  }
  for (QWidget* row : basic) basic_rows->addWidget(row);
  auto* advanced_page = new QWidget(content);
  auto* advanced_rows = new QVBoxLayout(advanced_page);
  advanced_rows->setContentsMargins(0, 0, 0, 0);
  advanced_rows->setSpacing(14);
  for (QWidget* row : advanced) advanced_rows->addWidget(row);
  layout->addWidget(basic_page);
  layout->addWidget(advanced_page);
  const auto show = [this, basic_page, advanced_page](bool on_advanced) {
    advanced_open_ = on_advanced;
    basic_page->setVisible(!on_advanced);
    advanced_page->setVisible(on_advanced);
  };
  (advanced_open_ ? advanced_tab : basic_tab)->setChecked(true);
  show(advanced_open_);
  connect(basic_tab, &QPushButton::clicked, this, [show] { show(false); });
  connect(advanced_tab, &QPushButton::clicked, this, [show] { show(true); });
}

void InspectorPanel::Rebuild() {
  App& app = App::Get();
  const bridge::InspectorView view = app.session().inspector();
  if (ToQ(view.id) != node_) advanced_open_ = false;
  node_ = ToQ(view.id);
  swatches_.clear();
  for (const auto& swatch : view.swatches) swatches_.append(ToQ(swatch));
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
    QList<QWidget*> basic;
    QList<QWidget*> advanced;
    bodies_.clear();
    for (const auto& field : view.fields) {
      QWidget* control = MakeControl(field);
      QWidget* row =
          field.kind == bridge::FieldKind::Switch
              ? MakeSwitchField(ToQ(field.label),
                                static_cast<ToggleSwitch*>(control),
                                ToQ(field.help), content)
              : MakeField(ToQ(field.label), control, ToQ(field.help), content);
      if (!field.presets.empty()) row = WithPresets(field, row, content);
      if (!field.group.empty()) {
        if (QVBoxLayout* body = bodies_.value(ToQ(field.group))) {
          body->addWidget(row);
        } else {
          row->deleteLater();
        }
        continue;
      }
      (field.advanced ? advanced : basic).append(row);
    }
    if (advanced.isEmpty()) {
      for (QWidget* row : basic) layout->addWidget(row);
    } else {
      AddTabs(layout, basic, advanced, content);
    }
    layout->addWidget(MakeRule(content));
    AddActions(layout, view, content);
    layout->addStretch(1);
  }
  const int scroll = scroll_->verticalScrollBar() != nullptr
                         ? scroll_->verticalScrollBar()->value()
                         : 0;
  for (QLineEdit* line : content->findChildren<QLineEdit*>()) {
    connect(line, &QLineEdit::editingFinished, line,
            [line] { line->setModified(false); });
  }
  scroll_->setWidget(content);
  content_ = content;
  scroll_->verticalScrollBar()->setValue(scroll);
}

}
