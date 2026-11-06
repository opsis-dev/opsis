#include "keyframe_fields.h"

#include <QDoubleValidator>
#include <QFrame>
#include <QGridLayout>
#include <QHBoxLayout>
#include <QJsonArray>
#include <QJsonDocument>
#include <QLabel>
#include <QLineEdit>
#include <QLocale>
#include <QPushButton>
#include <QSlider>
#include <QVBoxLayout>
#include <cmath>

#include "app.h"
#include "form_hold.h"
#include "opsis-qt/src/bridge.rs.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr int kNumberWidth = 56;
constexpr int kDialLabelWidth = 72;
constexpr int kDialValueWidth = 56;
constexpr int kTimePlaces = 2;

QJsonObject ObjectOf(const rust::String& json) {
  return QJsonDocument::fromJson(ToQ(json).toUtf8()).object();
}

QString Trimmed(double value, int places) {
  QString text = QString::number(value, 'f', places);
  if (text.contains(QLatin1Char('.'))) {
    while (text.endsWith(QLatin1Char('0'))) text.chop(1);
    if (text.endsWith(QLatin1Char('.'))) text.chop(1);
  }
  return text == QStringLiteral("-0") ? QStringLiteral("0") : text;
}

QLineEdit* NumberBox(const QString& name, const QString& text,
                     QWidget* parent) {
  auto* line = new QLineEdit(text, parent);
  line->setAccessibleName(name);
  line->setAlignment(Qt::AlignRight);
  line->setFixedWidth(kNumberWidth);
  auto* validator = new QDoubleValidator(line);
  validator->setNotation(QDoubleValidator::StandardNotation);
  validator->setLocale(QLocale::c());
  line->setValidator(validator);
  return line;
}

void OnTyped(QLineEdit* line, QObject* context,
             const std::function<void(double)>& done) {
  QObject::connect(line, &QLineEdit::editingFinished, context, [line, done] {
    if (!line->isModified()) return;
    line->setModified(false);
    bool ok = false;
    const double value = QLocale::c().toDouble(line->text(), &ok);
    if (ok) done(value);
  });
}

Combo* EaseBox(const QJsonArray& eases, const QString& ease, QWidget* parent) {
  auto* combo = new Combo(parent);
  combo->setAccessibleName(QStringLiteral("Easing"));
  for (const QJsonValue& entry : eases) {
    const QJsonArray pair = entry.toArray();
    combo->addItem(pair.at(1).toString(), pair.at(0).toString());
  }
  const int index =
      combo->findData(ease.isEmpty() ? QStringLiteral("linear") : ease);
  combo->setCurrentIndex(std::max(0, index));
  return combo;
}

QHBoxLayout* KeyRow(const QJsonObject& key, double fps, const QJsonArray& eases,
                    const QString& remove_label, QWidget* parent,
                    const KeyframeEdit& set, const KeyframeEdit& remove,
                    QLineEdit* extra = nullptr) {
  auto* row = new QHBoxLayout();
  row->setSpacing(4);
  auto* at = NumberBox(
      QStringLiteral("At"),
      Trimmed(key.value(QStringLiteral("frame")).toDouble() / fps, kTimePlaces),
      parent);
  row->addWidget(at);
  row->addWidget(MakeLabel(QStringLiteral("s"), "muted", parent));
  OnTyped(at, parent, [set, fps](double seconds) {
    set(QJsonObject{{QStringLiteral("frame"), std::max(0.0, seconds * fps)}});
  });
  if (extra != nullptr) row->addWidget(extra);
  auto* ease =
      EaseBox(eases, key.value(QStringLiteral("ease")).toString(), parent);
  QObject::connect(ease, &QComboBox::activated, parent, [ease, set](int) {
    set(QJsonObject{{QStringLiteral("ease"), ease->currentData().toString()}});
  });
  row->addWidget(ease, 1);
  auto* drop = MakeIconButton(QStringLiteral("x"), remove_label, parent, 14);
  QObject::connect(drop, &QToolButton::clicked, parent,
                   [remove] { remove(QJsonObject()); });
  row->addWidget(drop);
  return row;
}

}

QWidget* MakeEnvelopeField(const bridge::FieldView& field,
                           const KeyframeEdit& edit) {
  auto* box = new QWidget();
  auto* column = new QVBoxLayout(box);
  column->setContentsMargins(0, 0, 0, 0);
  column->setSpacing(10);
  const QJsonObject extra = ObjectOf(field.extra_json);
  const QJsonObject envelope = ObjectOf(field.value_json);
  const QJsonArray eases = extra.value(QStringLiteral("eases")).toArray();
  const double fps =
      std::max(1.0, extra.value(QStringLiteral("fps")).toDouble());
  for (const QJsonValue& entry :
       extra.value(QStringLiteral("tracks")).toArray()) {
    const QJsonObject track = entry.toObject();
    const QString name = track.value(QStringLiteral("key")).toString();
    const QString label = track.value(QStringLiteral("label")).toString();
    const QString unit = track.value(QStringLiteral("unit")).toString();
    const double factor = track.value(QStringLiteral("factor")).toDouble(1);
    auto* header = new QHBoxLayout();
    auto* title = MakeLabel(label, nullptr, box);
    title->setObjectName(QStringLiteral("TrackTitle"));
    header->addWidget(title);
    header->addStretch(1);
    auto* add =
        MakeButton(QStringLiteral("Add"), "ghost", QStringLiteral("plus"), box);
    add->setAccessibleName(
        QStringLiteral("Add %1 keyframe").arg(label.toLower()));
    QObject::connect(add, &QPushButton::clicked, box, [edit, name] {
      edit(QJsonObject{{QStringLiteral("op"), QStringLiteral("appendKey")},
                       {QStringLiteral("track"), name}});
    });
    header->addWidget(add);
    column->addLayout(header);
    const QJsonArray keys = envelope.value(name).toArray();
    for (int i = 0; i < keys.size(); ++i) {
      const QJsonObject key = keys.at(i).toObject();
      const auto set = [edit, name, i](const QJsonObject& patch) {
        QJsonObject change = patch;
        change.insert(QStringLiteral("op"), QStringLiteral("setKey"));
        change.insert(QStringLiteral("track"), name);
        change.insert(QStringLiteral("index"), i);
        edit(change);
      };
      const auto remove = [edit, name, i](const QJsonObject&) {
        edit(QJsonObject{{QStringLiteral("op"), QStringLiteral("removeKey")},
                         {QStringLiteral("track"), name},
                         {QStringLiteral("index"), i}});
      };
      auto* value = NumberBox(
          QStringLiteral("Value"),
          Trimmed(key.value(QStringLiteral("value")).toDouble() * factor, 3),
          box);
      OnTyped(value, box, [set, factor](double shown) {
        set(QJsonObject{{QStringLiteral("value"), shown / factor}});
      });
      auto* row = KeyRow(key, fps, eases, QStringLiteral("Remove keyframe"),
                         box, set, remove, value);
      if (!unit.isEmpty()) row->insertWidget(3, MakeLabel(unit, "muted", box));
      column->addLayout(row);
    }
  }
  return box;
}

QWidget* MakeCameraField(const bridge::FieldView& field,
                         const KeyframeEdit& edit) {
  auto* box = new QWidget();
  auto* column = new QVBoxLayout(box);
  column->setContentsMargins(0, 0, 0, 0);
  column->setSpacing(10);
  const QJsonObject extra = ObjectOf(field.extra_json);
  const QJsonArray eases = extra.value(QStringLiteral("eases")).toArray();
  const QJsonArray dials = extra.value(QStringLiteral("dials")).toArray();
  const double fps =
      std::max(1.0, extra.value(QStringLiteral("fps")).toDouble());
  const QJsonValue playhead = extra.value(QStringLiteral("playhead"));
  auto* add = MakeButton(QStringLiteral("Add key at playhead"), "ghost",
                         QStringLiteral("plus"), box);
  add->setEnabled(playhead.isDouble());
  if (!playhead.isDouble()) {
    add->setToolTip(QStringLiteral(
        "Put this node in the film to add keys at the playhead."));
  }
  QObject::connect(add, &QPushButton::clicked, box, [edit, playhead] {
    edit(QJsonObject{{QStringLiteral("op"), QStringLiteral("addCameraKey")},
                     {QStringLiteral("frame"), playhead.toDouble()}});
  });
  auto* header = new QHBoxLayout();
  header->addStretch(1);
  header->addWidget(add);
  column->addLayout(header);
  const QJsonArray keys =
      ObjectOf(field.value_json).value(QStringLiteral("keys")).toArray();
  for (int i = 0; i < keys.size(); ++i) {
    const QJsonObject key = keys.at(i).toObject();
    auto* card = new QFrame(box);
    card->setObjectName(QStringLiteral("MarkItem"));
    card->setAttribute(Qt::WA_StyledBackground);
    auto* rows = new QVBoxLayout(card);
    rows->setContentsMargins(8, 8, 8, 8);
    rows->setSpacing(6);
    const auto set = [edit, i](const QJsonObject& patch) {
      QJsonObject change = patch;
      change.insert(QStringLiteral("op"), QStringLiteral("setCameraKey"));
      change.insert(QStringLiteral("index"), i);
      edit(change);
    };
    const auto remove = [edit, i](const QJsonObject&) {
      edit(
          QJsonObject{{QStringLiteral("op"), QStringLiteral("removeCameraKey")},
                      {QStringLiteral("index"), i}});
    };
    rows->addLayout(KeyRow(key, fps, eases, QStringLiteral("Remove camera key"),
                           card, set, remove));
    auto* grid = new QGridLayout();
    grid->setHorizontalSpacing(8);
    grid->setVerticalSpacing(4);
    int line = 0;
    for (const QJsonValue& entry : dials) {
      const QJsonObject dial = entry.toObject();
      const QString name = dial.value(QStringLiteral("key")).toString();
      const QString label = dial.value(QStringLiteral("label")).toString();
      const QString unit = dial.value(QStringLiteral("unit")).toString();
      const double step =
          std::max(1e-6, dial.value(QStringLiteral("step")).toDouble(1));
      const double minimum = dial.value(QStringLiteral("min")).toDouble();
      const double maximum = dial.value(QStringLiteral("max")).toDouble();
      const double value = key.value(name).toDouble();
      auto* name_label = MakeLabel(label, "muted", card);
      name_label->setFixedWidth(kDialLabelWidth);
      auto* slider = new QSlider(Qt::Horizontal, card);
      slider->setAccessibleName(label);
      slider->setRange(static_cast<int>(std::lround(minimum / step)),
                       static_cast<int>(std::lround(maximum / step)));
      slider->setValue(static_cast<int>(std::lround(value / step)));
      auto* shown = MakeLabel(Trimmed(value, 2) + unit, nullptr, card);
      shown->setFixedWidth(kDialValueWidth);
      shown->setAlignment(Qt::AlignRight | Qt::AlignVCenter);
      QObject::connect(slider, &QSlider::sliderPressed, card,
                       [] { FormHold::Instance()->Begin(); });
      QObject::connect(slider, &QSlider::sliderReleased, card,
                       [] { FormHold::Instance()->End(); });
      QObject::connect(slider, &QSlider::valueChanged, card,
                       [set, name, step, shown, unit](int steps) {
                         const double next = steps * step;
                         shown->setText(Trimmed(next, 2) + unit);
                         set(QJsonObject{{name, next}});
                       });
      grid->addWidget(name_label, line, 0);
      grid->addWidget(slider, line, 1);
      grid->addWidget(shown, line, 2);
      ++line;
    }
    rows->addLayout(grid);
    column->addWidget(card);
  }
  return box;
}

}
