#include "visual_fields.h"

#include <QFontMetrics>
#include <QHBoxLayout>
#include <QJsonDocument>
#include <QJsonObject>
#include <QLabel>
#include <QPushButton>
#include <QVBoxLayout>

#include "app.h"
#include "color_picker.h"
#include "opsis-qt/src/bridge.rs.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr int kTokenLabelWidth = 112;

QString Compact(const QJsonObject& object) {
  return QString::fromUtf8(
      QJsonDocument(object).toJson(QJsonDocument::Compact));
}

QJsonObject ObjectOf(const rust::String& json) {
  return QJsonDocument::fromJson(ToQ(json).toUtf8()).object();
}

}

QWidget* MakeThemeField(const bridge::FieldView& field,
                        const QStringList& swatches, bool open,
                        const std::function<void(bool)>& on_open,
                        const CommitValue& commit) {
  auto* box = new QWidget();
  auto* column = new QVBoxLayout(box);
  column->setContentsMargins(0, 0, 0, 0);
  column->setSpacing(8);
  const QJsonObject theme = ObjectOf(field.value_json);
  const QJsonObject colors = theme.value(QStringLiteral("colors")).toObject();
  const QString mode = ToQ(field.text);
  const QJsonObject preset = ObjectOf(field.extra_json).value(mode).toObject();
  const auto send = [commit, mode](const QString& next_mode,
                                   const QJsonObject& next_colors) {
    commit(Compact(QJsonObject{
        {QStringLiteral("mode"), next_mode.isEmpty() ? mode : next_mode},
        {QStringLiteral("colors"), next_colors}}));
  };
  auto* modes = new Segmented(box);
  modes->SetChoices({{QStringLiteral("light"), QStringLiteral("Light")},
                     {QStringLiteral("dark"), QStringLiteral("Dark")}});
  modes->SetOutline();
  modes->SetValue(mode);
  QObject::connect(modes, &Segmented::Changed, box,
                   [send, colors](const QString& next) { send(next, colors); });
  column->addWidget(modes);
  if (field.choices.empty()) return box;

  int changed = 0;
  for (auto it = colors.begin(); it != colors.end(); ++it) {
    if (!it.value().toString().trimmed().isEmpty()) ++changed;
  }
  auto* header = new QHBoxLayout();
  header->setSpacing(4);
  auto* toggle = MakeButton(
      changed > 0 ? QStringLiteral("Colours (%1 changed)").arg(changed)
                  : QStringLiteral("Colours"),
      "ghost",
      open ? QStringLiteral("chevron-down") : QStringLiteral("chevron-right"),
      box);
  header->addWidget(toggle);
  header->addStretch(1);
  if (changed > 0) {
    auto* reset = MakeButton(QStringLiteral("Reset"), "ghost",
                             QStringLiteral("rotate-ccw"), box);
    QObject::connect(reset, &QPushButton::clicked, box,
                     [send] { send(QString(), QJsonObject()); });
    header->addWidget(reset);
  }
  column->addLayout(header);

  auto* body = new QWidget(box);
  auto* rows = new QVBoxLayout(body);
  rows->setContentsMargins(0, 0, 0, 0);
  rows->setSpacing(8);
  QStringList choices = swatches;
  for (auto it = preset.begin(); it != preset.end(); ++it) {
    if (!choices.contains(it.value().toString())) {
      choices.append(it.value().toString());
    }
  }
  for (const auto& choice : field.choices) {
    const QString token = ToQ(choice.value);
    const QString label = ToQ(choice.label);
    auto* row = new QHBoxLayout();
    row->setSpacing(8);
    auto* name = MakeLabel(label, "muted", body);
    name->setFixedWidth(kTokenLabelWidth);
    name->setText(name->fontMetrics().elidedText(label, Qt::ElideRight,
                                                 kTokenLabelWidth));
    name->setToolTip(label);
    row->addWidget(name);
    auto* input = new ColorInput(label, colors.value(token).toString(),
                                 QStringLiteral("Theme"), choices, body);
    input->SetFallback(preset.value(token).toString());
    QObject::connect(input, &ColorInput::Changed, box,
                     [send, colors, token](const QString& color) {
                       QJsonObject next = colors;
                       if (color.trimmed().isEmpty()) {
                         next.remove(token);
                       } else {
                         next.insert(token, color);
                       }
                       send(QString(), next);
                     });
    row->addWidget(input, 1);
    rows->addLayout(row);
  }
  body->setVisible(open);
  column->addWidget(body);
  QObject::connect(toggle, &QPushButton::clicked, box, [body, toggle, on_open] {
    const bool shown = !body->isVisible();
    body->setVisible(shown);
    toggle->setIcon(App::Get().Icon(shown ? QStringLiteral("chevron-down")
                                          : QStringLiteral("chevron-right"),
                                    App::Get().Token("foreground")));
    on_open(shown);
  });
  return box;
}

}
