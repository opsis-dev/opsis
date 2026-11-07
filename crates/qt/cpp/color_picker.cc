#include "color_picker.h"

#include <QGuiApplication>
#include <QHBoxLayout>
#include <QKeyEvent>
#include <QLinearGradient>
#include <QMouseEvent>
#include <QPainter>
#include <QPainterPath>
#include <QScreen>
#include <QVBoxLayout>
#include <algorithm>
#include <cmath>
#include <utility>

#include "app.h"
#include "form_hold.h"
#include "motion.h"
#include "opsis-qt/src/bridge.rs.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr int kPickerWidth = 256;
constexpr int kChecker = 4;
constexpr qreal kThumb = 7.0;
constexpr int kSwatchSize = 24;
constexpr double kFineStep = 0.01;
constexpr double kCoarseStep = 0.1;
constexpr double kFullHue = 360.0;
constexpr int kHueStops = 6;

QBrush Checker() {
  QPixmap tile(kChecker * 2, kChecker * 2);
  tile.fill(App::Get().Token("background"));
  QPainter painter(&tile);
  const QColor muted = App::Get().Token("muted");
  painter.fillRect(0, 0, kChecker, kChecker, muted);
  painter.fillRect(kChecker, kChecker, kChecker, kChecker, muted);
  painter.end();
  return {tile};
}

void PaintThumb(QPainter* painter, const QPointF& center) {
  painter->setBrush(Qt::NoBrush);
  painter->setPen(QPen(QColor(0, 0, 0, 100), 1.0));
  painter->drawEllipse(center, kThumb + 1, kThumb + 1);
  painter->setPen(QPen(Qt::white, 2.0));
  painter->drawEllipse(center, kThumb - 1, kThumb - 1);
}

}

std::optional<Hsva> ParseColor(const QString& text) {
  const bridge::ColorView view = App::Get().session().parse_color(ToRust(text));
  if (!view.valid) return std::nullopt;
  return Hsva{view.h, view.s, view.v, view.a};
}

QString FormatColor(const Hsva& color, const QString& format) {
  return ToQ(App::Get().session().format_color(color.h, color.s, color.v,
                                               color.a, ToRust(format)));
}

QString FormatOf(const QString& text) {
  return ToQ(App::Get().session().parse_color(ToRust(text)).format);
}

QColor ToQColor(const Hsva& color) {
  return QColor::fromHsvF(
      static_cast<float>(std::fmod(color.h, kFullHue) / kFullHue),
      static_cast<float>(std::clamp(color.s, 0.0, 1.0)),
      static_cast<float>(std::clamp(color.v, 0.0, 1.0)),
      static_cast<float>(std::clamp(color.a, 0.0, 1.0)));
}

void PaintColorChip(QPainter* painter, const QRectF& rect, const QString& value,
                    qreal radius) {
  QPainterPath path;
  path.addRoundedRect(rect, radius, radius);
  painter->save();
  painter->setRenderHint(QPainter::Antialiasing);
  painter->setClipPath(path);
  painter->fillRect(rect, Checker());
  if (const auto color = ParseColor(value)) {
    painter->fillRect(rect, ToQColor(*color));
  }
  painter->setClipping(false);
  painter->setBrush(Qt::NoBrush);
  painter->setPen(QPen(App::Get().Token("input"), 1.0));
  painter->drawPath(path);
  painter->restore();
}

SaturationArea::SaturationArea(QWidget* parent) : QWidget(parent) {
  setAccessibleName(QStringLiteral("Saturation and brightness"));
  setFocusPolicy(Qt::StrongFocus);
  setCursor(Qt::CrossCursor);
  setMinimumHeight(sizeHint().height());
}

void SaturationArea::SetColor(const Hsva& color) {
  color_ = color;
  setToolTip(QStringLiteral("%1% saturation, %2% brightness")
                 .arg(std::lround(color.s * 100))
                 .arg(std::lround(color.v * 100)));
  update();
}

void SaturationArea::paintEvent(QPaintEvent*) {
  QPainter painter(this);
  painter.setRenderHint(QPainter::Antialiasing);
  const QRectF area = QRectF(rect()).adjusted(0.5, 0.5, -0.5, -0.5);
  QPainterPath path;
  path.addRoundedRect(area, 6, 6);
  painter.setClipPath(path);
  QLinearGradient across(area.topLeft(), area.topRight());
  across.setColorAt(0, Qt::white);
  across.setColorAt(1, ToQColor(Hsva{color_.h, 1.0, 1.0, 1.0}));
  painter.fillRect(area, across);
  QLinearGradient down(area.topLeft(), area.bottomLeft());
  down.setColorAt(0, QColor(0, 0, 0, 0));
  down.setColorAt(1, Qt::black);
  painter.fillRect(area, down);
  painter.setClipping(false);
  PaintThumb(&painter, QPointF(area.left() + color_.s * area.width(),
                               area.top() + (1.0 - color_.v) * area.height()));
}

void SaturationArea::PickAt(const QPointF& point) {
  const double s = std::clamp(point.x() / std::max(1, width()), 0.0, 1.0);
  const double v =
      1.0 - std::clamp(point.y() / std::max(1, height()), 0.0, 1.0);
  emit Picked(s, v);
}

void SaturationArea::mousePressEvent(QMouseEvent* event) {
  setFocus();
  PickAt(event->position());
}

void SaturationArea::mouseMoveEvent(QMouseEvent* event) {
  if ((event->buttons() & Qt::LeftButton) != 0) PickAt(event->position());
}

void SaturationArea::keyPressEvent(QKeyEvent* event) {
  const double step =
      (event->modifiers() & Qt::ShiftModifier) != 0 ? kCoarseStep : kFineStep;
  double s = color_.s;
  double v = color_.v;
  switch (event->key()) {
    case Qt::Key_Left:
      s -= step;
      break;
    case Qt::Key_Right:
      s += step;
      break;
    case Qt::Key_Up:
      v += step;
      break;
    case Qt::Key_Down:
      v -= step;
      break;
    default:
      QWidget::keyPressEvent(event);
      return;
  }
  emit Picked(std::clamp(s, 0.0, 1.0), std::clamp(v, 0.0, 1.0));
}

ChannelSlider::ChannelSlider(const QString& label, double maximum, double step,
                             bool alpha, QWidget* parent)
    : QWidget(parent), maximum_(maximum), step_(step), alpha_(alpha) {
  setAccessibleName(label);
  setToolTip(label);
  setFocusPolicy(Qt::StrongFocus);
  setCursor(Qt::PointingHandCursor);
  setFixedHeight(sizeHint().height());
}

void ChannelSlider::SetColor(const Hsva& color) {
  color_ = color;
  value_ = alpha_ ? color.a : color.h;
  update();
}

void ChannelSlider::paintEvent(QPaintEvent*) {
  QPainter painter(this);
  painter.setRenderHint(QPainter::Antialiasing);
  const qreal inset = kThumb + 1;
  const QRectF track(inset, (height() - 12) / 2.0, width() - inset * 2, 12);
  QPainterPath path;
  path.addRoundedRect(track.adjusted(-inset, 0, inset, 0), 6, 6);
  painter.save();
  painter.setClipPath(path);
  QLinearGradient gradient(track.topLeft(), track.topRight());
  if (alpha_) {
    painter.fillRect(path.boundingRect(), Checker());
    QColor opaque = ToQColor(color_);
    opaque.setAlphaF(1.0F);
    QColor clear = opaque;
    clear.setAlphaF(0.0F);
    gradient.setColorAt(0, clear);
    gradient.setColorAt(1, opaque);
  } else {
    for (int i = 0; i <= kHueStops; ++i) {
      const double at = static_cast<double>(i) / kHueStops;
      gradient.setColorAt(at,
                          ToQColor(Hsva{at * kFullHue * 0.999, 1.0, 1.0, 1.0}));
    }
  }
  gradient.setSpread(QGradient::PadSpread);
  painter.fillRect(path.boundingRect(), gradient);
  painter.restore();
  PaintThumb(&painter, QPointF(track.left() + value_ / maximum_ * track.width(),
                               track.center().y()));
}

void ChannelSlider::MoveTo(double value) {
  value_ = std::clamp(std::round(value / step_) * step_, 0.0, maximum_);
  update();
  emit Moved(value_);
}

void ChannelSlider::PickAt(qreal x) {
  const qreal inset = kThumb + 1;
  const qreal span = std::max<qreal>(1.0, width() - inset * 2);
  MoveTo(std::clamp((x - inset) / span, 0.0, 1.0) * maximum_);
}

void ChannelSlider::mousePressEvent(QMouseEvent* event) {
  setFocus();
  PickAt(event->position().x());
}

void ChannelSlider::mouseMoveEvent(QMouseEvent* event) {
  if ((event->buttons() & Qt::LeftButton) != 0) PickAt(event->position().x());
}

void ChannelSlider::keyPressEvent(QKeyEvent* event) {
  const double step =
      (event->modifiers() & Qt::ShiftModifier) != 0 ? step_ * 10 : step_;
  switch (event->key()) {
    case Qt::Key_Left:
    case Qt::Key_Down:
      MoveTo(value_ - step);
      break;
    case Qt::Key_Right:
    case Qt::Key_Up:
      MoveTo(value_ + step);
      break;
    case Qt::Key_Home:
      MoveTo(0);
      break;
    case Qt::Key_End:
      MoveTo(maximum_);
      break;
    default:
      QWidget::keyPressEvent(event);
  }
}

QPointer<ColorPicker> ColorPicker::open_;

ColorPicker* ColorPicker::Open(QWidget* anchor, const QString& value,
                               const QString& fallback,
                               const QStringList& swatches,
                               OnChange on_change) {
  if (open_ != nullptr) open_->close();
  auto* picker = new ColorPicker(anchor->window(), value, fallback, swatches,
                                 std::move(on_change));
  open_ = picker;
  FormHold::Instance()->Begin();
  FitHeightToWidth(picker);
  QPoint at = anchor->mapToGlobal(QPoint(0, anchor->height() + 4));
  if (const QScreen* screen = anchor->screen()) {
    const QRect room = screen->availableGeometry();
    at.setX(std::clamp(at.x(), room.left(), room.right() - picker->width()));
    if (at.y() + picker->height() > room.bottom()) {
      at.setY(anchor->mapToGlobal(QPoint(0, 0)).y() - picker->height() - 4);
    }
  }
  picker->move(at);
  FadeIn(picker, kMotionMs, kRisePx);
  return picker;
}

bool ColorPicker::IsOpen() { return open_ != nullptr && open_->isVisible(); }

ColorPicker::ColorPicker(QWidget* parent, const QString& value,
                         const QString& fallback, const QStringList& swatches,
                         OnChange on_change)
    : QFrame(parent, Qt::Popup),
      on_change_(std::move(on_change)),
      area_(new SaturationArea(this)),
      hue_(
          new ChannelSlider(QStringLiteral("Hue"), kFullHue, 1.0, false, this)),
      alpha_(new ChannelSlider(QStringLiteral("Opacity"), 1.0, kFineStep, true,
                               this)),
      text_(new QLineEdit(this)),
      formats_(new Segmented(this)) {
  setObjectName(QStringLiteral("Popover"));
  setAttribute(Qt::WA_DeleteOnClose);
  setAttribute(Qt::WA_StyledBackground);
  setFixedWidth(kPickerWidth);
  auto* layout = new QVBoxLayout(this);
  layout->setContentsMargins(12, 12, 12, 12);
  layout->setSpacing(12);
  layout->addWidget(area_);
  layout->addWidget(hue_);
  layout->addWidget(alpha_);
  auto* row = new QHBoxLayout();
  row->setSpacing(8);
  if (QGuiApplication::platformName() != QStringLiteral("wayland")) {
    dropper_ =
        MakeIconButton(QStringLiteral("pipette"),
                       QStringLiteral("Pick a colour from the screen"), this);
    dropper_->setProperty("variant", "outline");
    connect(dropper_, &QToolButton::clicked, this, [this] {
      sampling_ = true;
      setCursor(Qt::CrossCursor);
      grabMouse(Qt::CrossCursor);
    });
    row->addWidget(dropper_);
  }
  text_->setObjectName(QStringLiteral("Mono"));
  text_->setAccessibleName(QStringLiteral("Colour value"));
  row->addWidget(text_, 1);
  layout->addLayout(row);
  formats_->SetChoices({{QStringLiteral("hex"), QStringLiteral("Hex")},
                        {QStringLiteral("rgb"), QStringLiteral("RGB")},
                        {QStringLiteral("hsl"), QStringLiteral("HSL")},
                        {QStringLiteral("oklch"), QStringLiteral("OKLCH")}});
  formats_->SetOutline();
  layout->addWidget(formats_);
  if (!swatches.isEmpty()) {
    auto* list = new QWidget(this);
    list->setAccessibleName(QStringLiteral("Colours in this project"));
    auto* flow = new FlowLayout(list, 6);
    flow->setContentsMargins(0, 0, 0, 0);
    for (const QString& swatch : swatches) {
      auto* chip = new ColorChip(swatch, kSwatchSize, list);
      chip->setToolTip(swatch);
      chip->setAccessibleName(QStringLiteral("Use %1").arg(swatch));
      connect(chip, &QToolButton::clicked, this, [this, swatch] {
        if (const auto picked = ParseColor(swatch)) {
          Pick(*picked, FormatOf(swatch));
        }
      });
      flow->addWidget(chip);
    }
    layout->addWidget(list);
  }
  const QString shown = value.isEmpty() ? fallback : value;
  format_ = FormatOf(shown);
  Show(ParseColor(shown).value_or(Hsva{}), value);
  connect(area_, &SaturationArea::Picked, this, [this](double s, double v) {
    Pick(Hsva{color_.h, s, v, color_.a}, format_);
  });
  connect(hue_, &ChannelSlider::Moved, this, [this](double h) {
    Pick(Hsva{h, color_.s, color_.v, color_.a}, format_);
  });
  connect(alpha_, &ChannelSlider::Moved, this, [this](double a) {
    Pick(Hsva{color_.h, color_.s, color_.v, a}, format_);
  });
  connect(text_, &QLineEdit::editingFinished, this, [this] {
    const QString typed = text_->text().trimmed();
    const auto parsed = ParseColor(typed);
    if (!parsed || !text_->isModified()) return;
    text_->setModified(false);
    format_ = FormatOf(typed);
    Show(*parsed, typed);
    if (on_change_) on_change_(typed);
  });
  connect(text_, &QLineEdit::textEdited, this, [this](const QString& typed) {
    text_->setProperty("invalid", !ParseColor(typed).has_value());
    text_->style()->unpolish(text_);
    text_->style()->polish(text_);
  });
  connect(formats_, &Segmented::Changed, this, [this](const QString& format) {
    format_ = format;
    if (ParseColor(text_->text())) Pick(color_, format);
  });
}

void ColorPicker::SetValue(const QString& value) {
  if (const auto parsed = ParseColor(value)) {
    format_ = FormatOf(value);
    Show(*parsed, value);
  }
}

void ColorPicker::Show(const Hsva& color, const QString& text) {
  color_ = color;
  area_->SetColor(color);
  hue_->SetColor(color);
  alpha_->SetColor(color);
  if (!text_->hasFocus() || !text_->isModified()) text_->setText(text);
  formats_->SetValue(format_);
}

void ColorPicker::Pick(const Hsva& color, const QString& format) {
  format_ = format;
  const QString text = FormatColor(color, format);
  Show(color, text);
  if (on_change_) on_change_(text);
}

void ColorPicker::SampleScreen(const QPoint& global) {
  QScreen* screen = QGuiApplication::screenAt(global);
  if (screen == nullptr) return;
  const QPoint local = global - screen->geometry().topLeft();
  const QImage pixel =
      screen->grabWindow(0, local.x(), local.y(), 1, 1).toImage();
  if (pixel.isNull()) return;
  const QColor sampled = pixel.pixelColor(0, 0);
  if (const auto picked = ParseColor(sampled.name())) {
    Pick(Hsva{picked->h, picked->s, picked->v, color_.a}, format_);
  }
}

void ColorPicker::mousePressEvent(QMouseEvent* event) {
  if (sampling_) {
    sampling_ = false;
    releaseMouse();
    unsetCursor();
    SampleScreen(event->globalPosition().toPoint());
    return;
  }
  QFrame::mousePressEvent(event);
}

void ColorPicker::hideEvent(QHideEvent* event) {
  QFrame::hideEvent(event);
  if (ended_) return;
  ended_ = true;
  if (sampling_) releaseMouse();
  FormHold::Instance()->End();
}

ColorChip::ColorChip(QString value, int size, QWidget* parent)
    : QToolButton(parent), value_(std::move(value)) {
  setFixedSize(size, size);
  setCursor(Qt::PointingHandCursor);
}

void ColorChip::SetValue(const QString& value) {
  value_ = value;
  update();
}

void ColorChip::paintEvent(QPaintEvent*) {
  QPainter painter(this);
  PaintColorChip(&painter, QRectF(rect()).adjusted(0.5, 0.5, -0.5, -0.5),
                 value_, 4);
  if (hasFocus()) {
    painter.setRenderHint(QPainter::Antialiasing);
    painter.setPen(QPen(App::Get().Token("ring"), 2.0));
    painter.drawRoundedRect(QRectF(rect()).adjusted(1, 1, -1, -1), 4, 4);
  }
}

ColorInput::ColorInput(const QString& label, const QString& value,
                       const QString& empty_label, const QStringList& swatches,
                       QWidget* parent)
    : QWidget(parent),
      chip_(new ColorChip(value, 20, this)),
      line_(new QLineEdit(this)),
      empty_label_(empty_label) {
  setObjectName(QStringLiteral("ColorInput"));
  auto* row = new QHBoxLayout(this);
  row->setContentsMargins(0, 0, 0, 0);
  row->setSpacing(8);
  chip_->setToolTip(QStringLiteral("Pick %1").arg(label.toLower()));
  chip_->setAccessibleName(chip_->toolTip());
  row->addWidget(chip_);
  line_->setObjectName(QStringLiteral("Mono"));
  line_->setText(value);
  line_->setPlaceholderText(empty_label);
  line_->setAccessibleName(label);
  row->addWidget(line_, 1);
  if (!empty_label.isEmpty()) {
    const QString tip =
        QStringLiteral("Use the %1 colour").arg(empty_label.toLower());
    clear_ = MakeIconButton(QStringLiteral("x"), tip, this, 14);
    clear_->setVisible(!value.isEmpty());
    connect(clear_, &QToolButton::clicked, this, [this] {
      Show(QString());
      emit Changed(QString());
    });
    row->addWidget(clear_);
  }
  connect(line_, &QLineEdit::editingFinished, this, [this] {
    if (!line_->isModified()) return;
    line_->setModified(false);
    Show(line_->text().trimmed());
    emit Changed(line_->text().trimmed());
  });
  connect(chip_, &QToolButton::clicked, this, [this, label, swatches] {
    ColorPicker* picker = ColorPicker::Open(
        chip_, line_->text(), fallback_, swatches,
        [self = QPointer<ColorInput>(this)](const QString& picked) {
          if (self == nullptr) return;
          self->Show(picked);
          emit self->Changed(picked);
        });
    picker->setAccessibleName(QStringLiteral("Pick %1").arg(label.toLower()));
  });
}

void ColorInput::SetFallback(const QString& fallback) {
  fallback_ = fallback;
  line_->setToolTip(
      QStringLiteral("%1 %2").arg(empty_label_, fallback).trimmed());
  Show(line_->text());
}

void ColorInput::Show(const QString& value) {
  chip_->SetValue(value.isEmpty() ? fallback_ : value);
  if (line_->text() != value) line_->setText(value);
  if (clear_ != nullptr) clear_->setVisible(!value.isEmpty());
}

}
