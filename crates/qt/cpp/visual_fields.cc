#include "visual_fields.h"

#include <QDoubleValidator>
#include <QFontMetrics>
#include <QGridLayout>
#include <QHBoxLayout>
#include <QJsonDocument>
#include <QJsonObject>
#include <QKeyEvent>
#include <QLabel>
#include <QLocale>
#include <QMouseEvent>
#include <QPainter>
#include <QPainterPath>
#include <QPushButton>
#include <QVBoxLayout>
#include <algorithm>
#include <cmath>
#include <utility>
#include <vector>

#include "app.h"
#include "color_picker.h"
#include "form_hold.h"
#include "opsis-qt/src/bridge.rs.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr int kTokenLabelWidth = 112;
constexpr int kLayerLabelWidth = 56;
constexpr double kMsPerSecond = 1000.0;
constexpr double kNudge = 0.01;
constexpr double kBigNudge = 0.1;
constexpr double kThousandths = 1000.0;
constexpr qreal kDot = 6.0;
constexpr int kSpotWidth = 280;

double Rounded(double value) {
  return std::round(std::clamp(value, 0.0, 1.0) * kThousandths) / kThousandths;
}

QString Seconds(double ms) { return QString::number(ms / kMsPerSecond); }

QString FormatList(std::vector<double> parts) {
  while (parts.size() > 1 && parts.back() == 0.0) parts.pop_back();
  if (std::all_of(parts.begin(), parts.end(),
                  [](double part) { return part == 0.0; })) {
    return {};
  }
  QStringList texts;
  for (double part : parts) texts.append(QString::number(part));
  return texts.join(QLatin1Char(','));
}

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

QWidget* MakeOffsetsField(const bridge::FieldView& field,
                          const std::function<void(const QString&)>& commit) {
  auto* box = new QWidget();
  auto* grid = new QGridLayout(box);
  grid->setContentsMargins(0, 0, 0, 0);
  grid->setHorizontalSpacing(8);
  grid->setVerticalSpacing(8);
  std::vector<double> parts;
  const QString text = ToQ(field.text);
  if (!text.trimmed().isEmpty()) {
    for (const QString& part : text.split(QLatin1Char(','))) {
      parts.push_back(QLocale::c().toDouble(part.trimmed()));
    }
  }
  const auto layers = std::max<std::size_t>(
      {static_cast<std::size_t>(std::max(1.0, field.number)), parts.size(), 1});
  parts.resize(layers, 0.0);
  for (std::size_t i = 0; i < layers; ++i) {
    auto* cell = new QWidget(box);
    auto* row = new QHBoxLayout(cell);
    row->setContentsMargins(0, 0, 0, 0);
    row->setSpacing(6);
    const QString name = QStringLiteral("Layer %1").arg(i + 1);
    auto* label = MakeLabel(name, "muted", cell);
    label->setFixedWidth(kLayerLabelWidth);
    row->addWidget(label);
    auto* line = new QLineEdit(Seconds(parts.at(i)), cell);
    line->setAccessibleName(name);
    line->setAlignment(Qt::AlignRight);
    auto* validator = new QDoubleValidator(0, 3600, 3, line);
    validator->setNotation(QDoubleValidator::StandardNotation);
    validator->setLocale(QLocale::c());
    line->setValidator(validator);
    row->addWidget(line, 1);
    row->addWidget(MakeLabel(QStringLiteral("s"), "muted", cell));
    QObject::connect(
        line, &QLineEdit::editingFinished, box, [line, parts, i, commit] {
          if (!line->isModified()) return;
          std::vector<double> next = parts;
          next.at(i) = std::max(
              0.0,
              std::round(QLocale::c().toDouble(line->text()) * kMsPerSecond));
          commit(FormatList(next));
        });
    grid->addWidget(cell, static_cast<int>(i / 2), static_cast<int>(i % 2));
  }
  return box;
}

SpotFrame::SpotFrame(QString label, QString x_key, QString y_key, double x,
                     double y, double aspect, const QString& picture,
                     CommitPatch commit, QWidget* parent)
    : QWidget(parent),
      label_(std::move(label)),
      x_key_(std::move(x_key)),
      y_key_(std::move(y_key)),
      x_(x),
      y_(y),
      aspect_(aspect > 0 ? aspect : 16.0 / 9.0),
      commit_(std::move(commit)) {
  if (!picture.isEmpty()) picture_.load(picture);
  setFocusPolicy(Qt::StrongFocus);
  setCursor(Qt::CrossCursor);
  setAccessibleName(label_);
  QSizePolicy policy(QSizePolicy::Preferred, QSizePolicy::Preferred);
  policy.setHeightForWidth(true);
  setSizePolicy(policy);
}

int SpotFrame::heightForWidth(int width) const {
  return static_cast<int>(std::lround(width / aspect_));
}

QSize SpotFrame::sizeHint() const {
  return {kSpotWidth, heightForWidth(kSpotWidth)};
}

void SpotFrame::paintEvent(QPaintEvent*) {
  QPainter painter(this);
  painter.setRenderHint(QPainter::Antialiasing);
  painter.setRenderHint(QPainter::SmoothPixmapTransform);
  const QRectF area = QRectF(rect()).adjusted(0.5, 0.5, -0.5, -0.5);
  QPainterPath path;
  path.addRoundedRect(area, 6, 6);
  painter.fillPath(path, App::Get().Token("muted"));
  if (!picture_.isNull()) {
    painter.save();
    painter.setClipPath(path);
    const QSizeF scaled =
        QSizeF(picture_.size())
            .scaled(area.size(), Qt::KeepAspectRatioByExpanding);
    painter.drawPixmap(
        QRectF(area.center() - QPointF(scaled.width(), scaled.height()) / 2,
               scaled),
        picture_, QRectF(picture_.rect()));
    painter.restore();
  }
  painter.setPen(QPen(
      hasFocus() ? App::Get().Token("ring") : App::Get().Token("border"), 1.0));
  painter.drawPath(path);
  const QPointF dot(area.left() + x_ * area.width(),
                    area.top() + y_ * area.height());
  painter.setPen(QPen(App::Get().Token("background"), 2.0));
  painter.setBrush(App::Get().Token("primary"));
  painter.drawEllipse(dot, kDot, kDot);
}

void SpotFrame::Place(double x, double y) {
  x_ = Rounded(x);
  y_ = Rounded(y);
  setToolTip(QStringLiteral("%1: %2%, %3%")
                 .arg(label_)
                 .arg(std::lround(x_ * 100))
                 .arg(std::lround(y_ * 100)));
  update();
  commit_(QJsonObject{{x_key_, x_}, {y_key_, y_}});
}

void SpotFrame::PlaceAt(const QPointF& point) {
  Place(point.x() / std::max(1, width()), point.y() / std::max(1, height()));
}

void SpotFrame::mousePressEvent(QMouseEvent* event) {
  setFocus();
  dragging_ = true;
  FormHold::Instance()->Begin();
  PlaceAt(event->position());
}

void SpotFrame::mouseMoveEvent(QMouseEvent* event) {
  if (dragging_) PlaceAt(event->position());
}

void SpotFrame::mouseReleaseEvent(QMouseEvent*) {
  if (!dragging_) return;
  dragging_ = false;
  FormHold::Instance()->End();
}

void SpotFrame::keyPressEvent(QKeyEvent* event) {
  const double step =
      (event->modifiers() & Qt::ShiftModifier) != 0 ? kBigNudge : kNudge;
  switch (event->key()) {
    case Qt::Key_Left:
      Place(x_ - step, y_);
      break;
    case Qt::Key_Right:
      Place(x_ + step, y_);
      break;
    case Qt::Key_Up:
      Place(x_, y_ - step);
      break;
    case Qt::Key_Down:
      Place(x_, y_ + step);
      break;
    default:
      QWidget::keyPressEvent(event);
  }
}

QWidget* MakeSpotField(const bridge::FieldView& field,
                       const CommitPatch& commit) {
  const QJsonObject extra = ObjectOf(field.extra_json);
  const QString reference = extra.value(QStringLiteral("picture")).toString();
  const QString picture =
      reference.isEmpty()
          ? QString()
          : ToQ(App::Get().session().file_path(ToRust(reference)));
  return new SpotFrame(
      ToQ(field.label), ToQ(field.key),
      extra.value(QStringLiteral("yKey")).toString(), field.number,
      extra.value(QStringLiteral("y")).toDouble(0.5),
      extra.value(QStringLiteral("aspect")).toDouble(), picture, commit);
}

}
