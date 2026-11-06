#include "visual_fields.h"

#include <QDoubleValidator>
#include <QFontMetrics>
#include <QGridLayout>
#include <QHBoxLayout>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QKeyEvent>
#include <QLabel>
#include <QLocale>
#include <QMouseEvent>
#include <QPainter>
#include <QPainterPath>
#include <QPushButton>
#include <QSlider>
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
constexpr double kWideAspect = 16.0 / 9.0;
constexpr qreal kRemoveSize = 16.0;
constexpr int kFrameStepMs = 100;

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

PictureFrame::PictureFrame(double aspect, const QString& picture,
                           QWidget* parent)
    : QWidget(parent), aspect_(aspect > 0 ? aspect : kWideAspect) {
  if (!picture.isEmpty()) picture_.load(picture);
  setFocusPolicy(Qt::StrongFocus);
  setCursor(Qt::CrossCursor);
  QSizePolicy policy(QSizePolicy::Preferred, QSizePolicy::Preferred);
  policy.setHeightForWidth(true);
  setSizePolicy(policy);
}

int PictureFrame::heightForWidth(int width) const {
  return static_cast<int>(std::lround(width / aspect_));
}

QSize PictureFrame::sizeHint() const {
  return {kSpotWidth, heightForWidth(kSpotWidth)};
}

QRectF PictureFrame::PaintPicture(QPainter* painter) {
  painter->setRenderHint(QPainter::Antialiasing);
  painter->setRenderHint(QPainter::SmoothPixmapTransform);
  const QRectF area = QRectF(rect()).adjusted(0.5, 0.5, -0.5, -0.5);
  QPainterPath path;
  path.addRoundedRect(area, 6, 6);
  painter->fillPath(path, App::Get().Token("muted"));
  if (!picture_.isNull()) {
    painter->save();
    painter->setClipPath(path);
    const QSizeF scaled =
        QSizeF(picture_.size())
            .scaled(area.size(), Qt::KeepAspectRatioByExpanding);
    painter->drawPixmap(
        QRectF(area.center() - QPointF(scaled.width(), scaled.height()) / 2,
               scaled),
        picture_, QRectF(picture_.rect()));
    painter->restore();
  }
  painter->setPen(QPen(
      hasFocus() ? App::Get().Token("ring") : App::Get().Token("border"), 1.0));
  painter->setBrush(Qt::NoBrush);
  painter->drawPath(path);
  return area;
}

SpotFrame::SpotFrame(QString label, QString x_key, QString y_key, double x,
                     double y, double aspect, const QString& picture,
                     CommitPatch commit, QWidget* parent)
    : PictureFrame(aspect, picture, parent),
      label_(std::move(label)),
      x_key_(std::move(x_key)),
      y_key_(std::move(y_key)),
      x_(x),
      y_(y),
      commit_(std::move(commit)) {
  setAccessibleName(label_);
}

void SpotFrame::paintEvent(QPaintEvent*) {
  QPainter painter(this);
  const QRectF area = PaintPicture(&painter);
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

namespace {

bridge::BoundsView ToBounds(const Area& area) {
  return bridge::BoundsView{area.x, area.y, area.width, area.height};
}

Area FromBounds(const bridge::BoundsView& bounds) {
  return Area{bounds.x, bounds.y, bounds.width, bounds.height};
}

}

BoxesFrame::BoxesFrame(std::vector<Area> areas, int most, double aspect,
                       const QString& picture, QWidget* parent)
    : PictureFrame(aspect, picture, parent),
      areas_(std::move(areas)),
      most_(most) {
  setMouseTracking(true);
}

QRectF BoxesFrame::ToWidget(const Area& area) const {
  const QRectF frame = QRectF(rect()).adjusted(0.5, 0.5, -0.5, -0.5);
  return {frame.left() + area.x * frame.width(),
          frame.top() + area.y * frame.height(), area.width * frame.width(),
          area.height * frame.height()};
}

QRectF BoxesFrame::RemoveButton(const Area& area) const {
  const QRectF box = ToWidget(area);
  return {box.right() - kRemoveSize / 2, box.top() - kRemoveSize / 2,
          kRemoveSize, kRemoveSize};
}

QPointF BoxesFrame::FractionAt(const QPointF& point) const {
  return {std::clamp(point.x() / std::max(1, width()), 0.0, 1.0),
          std::clamp(point.y() / std::max(1, height()), 0.0, 1.0)};
}

void BoxesFrame::paintEvent(QPaintEvent*) {
  QPainter painter(this);
  PaintPicture(&painter);
  QColor fill = App::Get().Token("primary");
  fill.setAlphaF(0.25F);
  const QColor edge = App::Get().Token("primary");
  for (std::size_t i = 0; i < areas_.size(); ++i) {
    const Area& area = moving_ == static_cast<int>(i) ? draft_ : areas_.at(i);
    painter.setPen(QPen(edge, 2.0));
    painter.setBrush(fill);
    painter.drawRect(ToWidget(area));
  }
  if (drawing_) {
    painter.setPen(QPen(edge, 2.0, Qt::DashLine));
    painter.setBrush(fill);
    painter.drawRect(ToWidget(draft_));
  }
  if (moving_ >= 0 || drawing_) return;
  for (const Area& area : areas_) {
    const QRectF button = RemoveButton(area);
    painter.setPen(QPen(App::Get().Token("border"), 1.0));
    painter.setBrush(App::Get().Token("background"));
    painter.drawEllipse(button);
    painter.drawPixmap(button.adjusted(3, 3, -3, -3).toRect(),
                       App::Get().Glyph(QStringLiteral("x"),
                                        App::Get().Token("foreground"), 10));
  }
}

void BoxesFrame::mousePressEvent(QMouseEvent* event) {
  setFocus();
  const QPointF point = event->position();
  for (int i = static_cast<int>(areas_.size()) - 1; i >= 0; --i) {
    if (RemoveButton(areas_.at(static_cast<std::size_t>(i))).contains(point)) {
      std::vector<Area> next = areas_;
      next.erase(next.begin() + i);
      emit Changed(next);
      return;
    }
  }
  start_ = FractionAt(point);
  for (int i = static_cast<int>(areas_.size()) - 1; i >= 0; --i) {
    if (ToWidget(areas_.at(static_cast<std::size_t>(i))).contains(point)) {
      moving_ = i;
      origin_ = areas_.at(static_cast<std::size_t>(i));
      draft_ = origin_;
      return;
    }
  }
  if (static_cast<int>(areas_.size()) >= most_) return;
  drawing_ = true;
  draft_ = FromBounds(App::Get().session().box_from(start_.x(), start_.y(),
                                                    start_.x(), start_.y()));
  update();
}

void BoxesFrame::mouseMoveEvent(QMouseEvent* event) {
  const QPointF at = FractionAt(event->position());
  if (moving_ >= 0) {
    draft_ = FromBounds(App::Get().session().move_box(
        ToBounds(origin_), at.x() - start_.x(), at.y() - start_.y()));
    update();
  } else if (drawing_) {
    draft_ = FromBounds(
        App::Get().session().box_from(start_.x(), start_.y(), at.x(), at.y()));
    update();
  } else {
    bool over = false;
    for (const Area& area : areas_) {
      over = over || ToWidget(area).contains(event->position());
    }
    setCursor(over ? Qt::SizeAllCursor : Qt::CrossCursor);
  }
}

void BoxesFrame::mouseReleaseEvent(QMouseEvent*) {
  std::vector<Area> next = areas_;
  if (moving_ >= 0) {
    next.at(static_cast<std::size_t>(moving_)) = draft_;
  } else if (drawing_ && App::Get().session().box_kept(ToBounds(draft_))) {
    next.push_back(draft_);
  } else {
    drawing_ = false;
    update();
    return;
  }
  moving_ = -1;
  drawing_ = false;
  update();
  emit Changed(next);
}

QWidget* MakeBoxesField(const bridge::FieldView& field,
                        const CommitValue& commit, const CommitPatch& patch) {
  auto* box = new QWidget();
  auto* column = new QVBoxLayout(box);
  column->setContentsMargins(0, 0, 0, 0);
  column->setSpacing(8);
  const QJsonObject extra = ObjectOf(field.extra_json);
  const QString reference = extra.value(QStringLiteral("picture")).toString();
  const QString picture =
      reference.isEmpty()
          ? QString()
          : ToQ(App::Get().session().file_path(ToRust(reference)));
  std::vector<Area> areas;
  for (const QJsonValue& value :
       QJsonDocument::fromJson(ToQ(field.value_json).toUtf8()).array()) {
    const QJsonObject item = value.toObject();
    areas.push_back(Area{item.value(QStringLiteral("x")).toDouble(),
                         item.value(QStringLiteral("y")).toDouble(),
                         item.value(QStringLiteral("width")).toDouble(),
                         item.value(QStringLiteral("height")).toDouble()});
  }
  const int most = static_cast<int>(std::min<std::uint32_t>(
      field.max_items, static_cast<std::uint32_t>(INT32_MAX)));
  auto* frame = new BoxesFrame(areas, most,
                               extra.value(QStringLiteral("aspect")).toDouble(),
                               picture, box);
  frame->setAccessibleName(ToQ(field.label));
  QObject::connect(
      frame, &BoxesFrame::Changed, box,
      [commit](const std::vector<Area>& next) {
        QJsonArray array;
        for (const Area& area : next) {
          array.append(QJsonObject{{QStringLiteral("x"), area.x},
                                   {QStringLiteral("y"), area.y},
                                   {QStringLiteral("width"), area.width},
                                   {QStringLiteral("height"), area.height}});
        }
        commit(QString::fromUtf8(
            QJsonDocument(array).toJson(QJsonDocument::Compact)));
      });
  column->addWidget(frame);
  const QString time_key = extra.value(QStringLiteral("timeKey")).toString();
  const double duration = extra.value(QStringLiteral("durationMs")).toDouble();
  if (!time_key.isEmpty() && duration > 0) {
    auto* row = new QHBoxLayout();
    row->setSpacing(12);
    auto* slider = new QSlider(Qt::Horizontal, box);
    slider->setAccessibleName(QStringLiteral("Frame to draw on"));
    slider->setRange(0, static_cast<int>(duration));
    slider->setSingleStep(kFrameStepMs);
    slider->setPageStep(kFrameStepMs * 10);
    const double time =
        std::min(extra.value(QStringLiteral("time")).toDouble(), duration);
    slider->setValue(static_cast<int>(time));
    auto* shown = MakeLabel(Seconds(time) + QStringLiteral(" s"), "muted", box);
    row->addWidget(slider, 1);
    row->addWidget(shown);
    QObject::connect(slider, &QSlider::sliderPressed, box,
                     [] { FormHold::Instance()->Begin(); });
    QObject::connect(slider, &QSlider::sliderReleased, box,
                     [] { FormHold::Instance()->End(); });
    QObject::connect(slider, &QSlider::valueChanged, box,
                     [patch, time_key, shown](int value) {
                       const int stepped = value / kFrameStepMs * kFrameStepMs;
                       shown->setText(Seconds(stepped) + QStringLiteral(" s"));
                       patch(QJsonObject{{time_key, stepped}});
                     });
    column->addLayout(row);
  }
  const auto count = static_cast<int>(areas.size());
  column->addWidget(MakeLabel(
      count >= most
          ? QStringLiteral("Up to %1 areas. Remove one to add another.")
                .arg(most)
          : QStringLiteral("%1 of %2 areas.").arg(count).arg(most),
      "muted", box));
  return box;
}

}
