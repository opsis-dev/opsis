#include "mixer_controls.h"

#include <QCheckBox>
#include <QGridLayout>
#include <QHBoxLayout>
#include <QJsonDocument>
#include <QKeyEvent>
#include <QLabel>
#include <QLinearGradient>
#include <QLocale>
#include <QMouseEvent>
#include <QPainter>
#include <QPainterPath>
#include <QPolygonF>
#include <QPushButton>
#include <QScreen>
#include <QVBoxLayout>
#include <QWheelEvent>
#include <algorithm>
#include <array>
#include <cmath>
#include <memory>
#include <utility>

#include "app.h"
#include "form_hold.h"
#include "motion.h"
#include "opsis-qt/src/bridge.rs.h"
#include "playback_clock.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr double kStartDeg = -135.0;
constexpr double kSweepDeg = 270.0;
constexpr double kDragPx = 150.0;
constexpr double kFineDragPx = 600.0;
constexpr double kNudge = 0.02;
constexpr double kBigNudge = 0.1;
constexpr double kRingInset = 3.0;
constexpr double kPointerInset = 4.0;
constexpr double kDragStart = 3.0;
constexpr double kFaderStep = 0.5;
constexpr double kFaderBigStep = 6.0;
constexpr double kMeterMin = -60.0;
constexpr double kMeterMax = 6.0;
constexpr double kMaxReductionDb = 24.0;
constexpr double kClipDb = -0.5;
constexpr double kCurveRange = 18.0;
constexpr int kSmallFont = 10;
constexpr int kEqPopoverWidth = 420;
constexpr int kCompPopoverWidth = 320;
constexpr int kBandKnob = 30;

QColor Token(const char* name) { return App::Get().Token(QLatin1String(name)); }

double Normalized(double value, double min, double max) {
  return max > min ? std::clamp((value - min) / (max - min), 0.0, 1.0) : 0.0;
}

QPointF Polar(QPointF centre, double radius, double degrees) {
  const double radians = (degrees - 90.0) * M_PI / 180.0;
  return centre +
         QPointF(radius * std::cos(radians), radius * std::sin(radians));
}

QLabel* SmallLabel(const QString& text, const char* role, QWidget* parent) {
  auto* label = MakeLabel(text, role, parent);
  QFont font = label->font();
  font.setPixelSize(kSmallFont);
  UseFont(label, font);
  label->setAlignment(Qt::AlignCenter);
  return label;
}

QWidget* SwitchHeader(const QString& title, const QString& label, bool on,
                      const std::function<void(bool)>& changed,
                      QWidget* parent) {
  auto* row = new QWidget(parent);
  auto* layout = new QHBoxLayout(row);
  layout->setContentsMargins(0, 0, 0, 0);
  auto* text = MakeLabel(title, nullptr, row);
  QFont font = text->font();
  font.setWeight(QFont::Medium);
  text->setFont(font);
  layout->addWidget(text, 1);
  auto* toggle = new ToggleSwitch(row);
  toggle->setAccessibleName(label);
  toggle->SetCheckedQuietly(on);
  QObject::connect(toggle, &ToggleSwitch::toggled, row, changed);
  layout->addWidget(toggle);
  return row;
}

}

QString MixText(const QString& kind, double value) {
  if (kind == QStringLiteral("ratio")) {
    return QStringLiteral("%1:1").arg(QString::number(value, 'f', 1));
  }
  if (kind == QStringLiteral("fixed1")) return QString::number(value, 'f', 1);
  if (kind == QStringLiteral("fixed2")) return QString::number(value, 'f', 2);
  if (kind == QStringLiteral("ignore")) {
    return value > 0 ? MixText(QStringLiteral("ms"), value)
                     : QStringLiteral("Off");
  }
  return ToQ(App::Get().session().mix_text(ToRust(kind), value));
}

KnobSpec SpecFrom(const QJsonObject& range, const QString& label, double step,
                  const QString& format, const QString& unit, bool log) {
  KnobSpec spec;
  spec.label = label;
  spec.min = range.value(QStringLiteral("min")).toDouble();
  spec.max = range.value(QStringLiteral("max")).toDouble(1);
  spec.reset = range.value(QStringLiteral("default")).toDouble(spec.min);
  spec.step = step;
  spec.log = log;
  spec.format = format;
  spec.unit = unit;
  return spec;
}

KnobDial::KnobDial(KnobSpec spec, double value, int size, OnValue on_value,
                   QWidget* parent)
    : QWidget(parent),
      spec_(std::move(spec)),
      value_(value),
      on_value_(std::move(on_value)) {
  setFixedSize(size, size);
  setFocusPolicy(Qt::StrongFocus);
  setCursor(Qt::SizeVerCursor);
  const QString spoken =
      spec_.label + QStringLiteral(": ") + MixText(spec_.format, value_) +
      (spec_.unit.isEmpty() ? QString() : QStringLiteral(" ") + spec_.unit);
  setAccessibleName(spec_.label);
  setToolTip(spoken + QStringLiteral(
                          ". Drag up or down, double-click to reset, click the "
                          "number to type."));
}

double KnobDial::Fraction(double value) const {
  return App::Get().session().knob_fraction(spec_.min, spec_.max, spec_.log,
                                            value);
}

void KnobDial::SetValue(double value) {
  value_ = value;
  update();
}

void KnobDial::Commit(double value) {
  value_ = App::Get().session().knob_value(spec_.min, spec_.max, spec_.step,
                                           spec_.log, Fraction(value));
  update();
  if (on_value_) on_value_(value_);
}

void KnobDial::Nudge(double by) {
  if (!isEnabled()) return;
  value_ = App::Get().session().knob_value(spec_.min, spec_.max, spec_.step,
                                           spec_.log, Fraction(value_) + by);
  update();
  if (on_value_) on_value_(value_);
}

void KnobDial::paintEvent(QPaintEvent*) {
  QPainter painter(this);
  painter.setRenderHint(QPainter::Antialiasing);
  const QPointF centre(width() / 2.0, height() / 2.0);
  const double radius = width() / 2.0 - kRingInset;
  const QRectF ring(centre.x() - radius, centre.y() - radius, radius * 2,
                    radius * 2);
  const auto arc = [&](double from, double to, const QColor& colour) {
    QPainterPath path;
    path.arcMoveTo(ring, 90 - from);
    path.arcTo(ring, 90 - from, -(to - from));
    painter.setPen(QPen(colour, 3, Qt::SolidLine, Qt::RoundCap));
    painter.setBrush(Qt::NoBrush);
    painter.drawPath(path);
  };
  arc(kStartDeg, kStartDeg + kSweepDeg, Token("muted"));
  const double fraction = Fraction(value_);
  const double angle = kStartDeg + fraction * kSweepDeg;
  const double centre_fraction =
      spec_.min < 0 && spec_.max > 0 && !spec_.log ? Fraction(0) : 0.0;
  const double centre_angle = kStartDeg + centre_fraction * kSweepDeg;
  if (std::abs(angle - centre_angle) > 0.5) {
    arc(std::min(angle, centre_angle), std::max(angle, centre_angle),
        Token("primary"));
  }
  painter.setPen(QPen(Token("foreground"), 2, Qt::SolidLine, Qt::RoundCap));
  painter.drawLine(centre, Polar(centre, radius - kPointerInset, angle));
}

void KnobDial::mousePressEvent(QMouseEvent* event) {
  setFocus();
  dragging_ = true;
  started_ = false;
  press_y_ = event->position().y();
  press_fraction_ = Fraction(value_);
}

void KnobDial::mouseMoveEvent(QMouseEvent* event) {
  if (!dragging_) return;
  const double dy = press_y_ - event->position().y();
  if (!started_) {
    if (std::abs(dy) < kDragStart) return;
    started_ = true;
    FormHold::Instance()->Begin();
  }
  const double span =
      (event->modifiers() & Qt::ShiftModifier) != 0 ? kFineDragPx : kDragPx;
  value_ = App::Get().session().knob_value(
      spec_.min, spec_.max, spec_.step, spec_.log, press_fraction_ + dy / span);
  update();
  if (on_value_) on_value_(value_);
}

void KnobDial::mouseReleaseEvent(QMouseEvent*) {
  dragging_ = false;
  if (started_) FormHold::Instance()->End();
  started_ = false;
}

void KnobDial::mouseDoubleClickEvent(QMouseEvent*) { Commit(spec_.reset); }

void KnobDial::wheelEvent(QWheelEvent* event) {
  Nudge(event->angleDelta().y() > 0 ? kNudge : -kNudge);
}

void KnobDial::keyPressEvent(QKeyEvent* event) {
  const double step =
      (event->modifiers() & Qt::ShiftModifier) != 0 ? kBigNudge : kNudge;
  switch (event->key()) {
    case Qt::Key_Up:
    case Qt::Key_Right:
      Nudge(step);
      break;
    case Qt::Key_Down:
    case Qt::Key_Left:
      Nudge(-step);
      break;
    default:
      QWidget::keyPressEvent(event);
  }
}

QWidget* MakeKnob(const KnobSpec& spec, double value, int size,
                  const OnValue& on_value, bool disabled, QWidget* parent) {
  auto* box = new QWidget(parent);
  auto* column = new QVBoxLayout(box);
  column->setContentsMargins(0, 0, 0, 0);
  column->setSpacing(2);
  auto* reading = new QPushButton(MixText(spec.format, value), box);
  reading->setObjectName(QStringLiteral("KnobValue"));
  reading->setFlat(true);
  reading->setToolTip(QStringLiteral("Click to type a value"));
  auto* typed = new QLineEdit(box);
  typed->setObjectName(QStringLiteral("KnobInput"));
  typed->setAccessibleName(spec.label + QStringLiteral(" value"));
  typed->setAlignment(Qt::AlignCenter);
  typed->hide();
  auto* dial = new KnobDial(
      spec, value, size,
      [reading, spec, on_value](double next) {
        reading->setText(MixText(spec.format, next));
        on_value(next);
      },
      box);
  column->addWidget(dial, 0, Qt::AlignHCenter);
  column->addWidget(SmallLabel(spec.label, "muted", box));
  column->addWidget(reading, 0, Qt::AlignHCenter);
  column->addWidget(typed, 0, Qt::AlignHCenter);
  QObject::connect(reading, &QPushButton::clicked, box, [reading, typed, dial] {
    typed->setText(QString::number(dial->value()));
    reading->hide();
    typed->show();
    typed->setFocus();
    typed->selectAll();
  });
  QObject::connect(typed, &QLineEdit::editingFinished, box,
                   [reading, typed, dial, spec, on_value] {
                     bool ok = false;
                     const double entered =
                         QLocale::c().toDouble(typed->text().trimmed(), &ok);
                     typed->hide();
                     reading->show();
                     if (!ok || !typed->isModified()) return;
                     const double snapped = App::Get().session().knob_value(
                         spec.min, spec.max, spec.step, spec.log,
                         App::Get().session().knob_fraction(spec.min, spec.max,
                                                            spec.log, entered));
                     dial->SetValue(snapped);
                     reading->setText(MixText(spec.format, snapped));
                     on_value(snapped);
                   });
  box->setEnabled(!disabled);
  if (disabled) box->setProperty("dim", true);
  return box;
}

Fader::Fader(QString label, double value, double min, double max,
             OnValue on_value, QWidget* parent)
    : QWidget(parent),
      label_(std::move(label)),
      value_(value),
      min_(min),
      max_(max),
      on_value_(std::move(on_value)) {
  setFocusPolicy(Qt::StrongFocus);
  setCursor(Qt::SizeVerCursor);
  setAccessibleName(label_);
  setToolTip(QStringLiteral("%1: %2 dB. Double-click for 0 dB.")
                 .arg(label_, MixText(QStringLiteral("db"), value_)));
  setMinimumHeight(64);
  setFixedWidth(16);
}

double Fader::Position(double value) const {
  return App::Get().session().knob_fraction(min_, max_, false, value);
}

double Fader::ThumbY() const {
  const double top = 4;
  const double bottom = height() - 4;
  return bottom - Position(value_) * (bottom - top);
}

double Fader::ValueAt(double y) const {
  const double top = 4;
  const double bottom = height() - 4;
  const double fraction = 1 - (y - top) / std::max(1.0, bottom - top);
  return App::Get().session().knob_value(min_, max_, kFaderStep, false,
                                         fraction);
}

void Fader::Set(double value) {
  value_ = value;
  setToolTip(QStringLiteral("%1: %2 dB. Double-click for 0 dB.")
                 .arg(label_, MixText(QStringLiteral("db"), value_)));
  update();
  if (on_value_) on_value_(value_);
}

void Fader::paintEvent(QPaintEvent*) {
  QPainter painter(this);
  painter.setRenderHint(QPainter::Antialiasing);
  const double top = 4;
  const double bottom = height() - 4;
  const double x = width() / 2.0;
  painter.setPen(Qt::NoPen);
  painter.setBrush(Token("muted"));
  painter.drawRoundedRect(QRectF(x - 2, top, 4, bottom - top), 2, 2);
  const auto at = [&](double value) {
    return bottom - Position(value) * (bottom - top);
  };
  painter.fillRect(QRectF(0, at(0), width(), 1),
                   App::Get().Token(QStringLiteral("foreground")).darker(160));
  const QRectF thumb(0, at(value_) - 5, width(), 10);
  painter.setPen(QPen(Token("background"), 1));
  painter.setBrush(Token("primary"));
  painter.drawRoundedRect(thumb, 2, 2);
}

void Fader::mousePressEvent(QMouseEvent* event) {
  setFocus();
  dragging_ = true;
  FormHold::Instance()->Begin();
  Set(ValueAt(event->position().y()));
}

void Fader::mouseMoveEvent(QMouseEvent* event) {
  if (dragging_) Set(ValueAt(event->position().y()));
}

void Fader::mouseReleaseEvent(QMouseEvent*) {
  if (!dragging_) return;
  dragging_ = false;
  FormHold::Instance()->End();
}

void Fader::mouseDoubleClickEvent(QMouseEvent*) { Set(0); }

void Fader::keyPressEvent(QKeyEvent* event) {
  const double step = (event->modifiers() & Qt::ShiftModifier) != 0
                          ? kFaderBigStep
                          : kFaderStep;
  int direction = 0;
  if (event->key() == Qt::Key_Up) direction = 1;
  if (event->key() == Qt::Key_Down) direction = -1;
  if (direction == 0) {
    QWidget::keyPressEvent(event);
    return;
  }
  Set(App::Get().session().knob_value(min_, max_, kFaderStep, false,
                                      Position(value_ + direction * step)));
}

Meter::Meter(QString target, bool reduction, QWidget* parent)
    : QWidget(parent), target_(std::move(target)), reduction_(reduction) {
  setFixedWidth(reduction_ ? 14 : 10);
  setMinimumHeight(64);
}

void Meter::paintEvent(QPaintEvent*) {
  QPainter painter(this);
  painter.setRenderHint(QPainter::Antialiasing);
  const bridge::MeterView level = App::Get().session().meter(
      ToRust(target_), PlaybackClock::Instance()->frame());
  const QRectF bar(0, 0, 10, height());
  painter.setPen(Qt::NoPen);
  painter.setBrush(Token("muted"));
  painter.drawRoundedRect(bar, 2, 2);
  const auto height_of = [&](double db) {
    return Normalized(db, kMeterMin, kMeterMax) * bar.height();
  };
  if (level.has) {
    QLinearGradient gradient(bar.bottomLeft(), bar.topLeft());
    gradient.setColorAt(0, Token("meter-low"));
    gradient.setColorAt(0.6, Token("meter-mid"));
    gradient.setColorAt(1, Token("meter-high"));
    painter.setOpacity(0.4);
    const double peak = height_of(level.peak);
    painter.fillRect(QRectF(0, bar.bottom() - peak, 10, peak), gradient);
    painter.setOpacity(1);
    const double rms = height_of(level.rms);
    painter.fillRect(QRectF(0, bar.bottom() - rms, 10, rms),
                     Token("meter-low"));
    if (level.hold > kMeterMin) {
      painter.fillRect(
          QRectF(0, bar.bottom() - height_of(level.hold), 10, 1),
          level.hold > kClipDb ? Token("meter-high") : Token("foreground"));
    }
    setToolTip(QStringLiteral("Peak %1 dB, average %2 dB")
                   .arg(MixText(QStringLiteral("level"), level.peak),
                        MixText(QStringLiteral("level"), level.rms)));
  } else {
    setToolTip(QStringLiteral("No level yet"));
  }
  painter.fillRect(QRectF(0, bar.bottom() - height_of(0), 10, 1),
                   App::Get().Token(QStringLiteral("foreground")).darker(160));
  if (reduction_ && !std::isnan(level.reduction)) {
    const QRectF lane(12, 0, 2, height());
    painter.fillRect(lane, Token("muted"));
    painter.fillRect(
        QRectF(
            12, 0, 2,
            std::clamp(level.reduction / kMaxReductionDb, 0.0, 1.0) * height()),
        Token("keyframe"));
  }
}

EqCurve::EqCurve(QJsonObject eq, double highpass, QSize size, QWidget* parent)
    : QWidget(parent), eq_(std::move(eq)), highpass_(highpass) {
  setFixedSize(size);
}

void EqCurve::SetEq(const QJsonObject& eq) {
  eq_ = eq;
  update();
}

void EqCurve::paintEvent(QPaintEvent*) {
  QPainter painter(this);
  painter.setRenderHint(QPainter::Antialiasing);
  QPainterPath back;
  back.addRoundedRect(QRectF(rect()), 3, 3);
  QColor muted = Token("muted");
  muted.setAlphaF(0.5F);
  painter.fillPath(back, muted);
  const double mid = height() / 2.0;
  painter.setPen(
      QPen(App::Get().Token(QStringLiteral("foreground")).darker(300), 1));
  painter.drawLine(QPointF(0, mid), QPointF(width(), mid));
  const auto steps =
      static_cast<std::uint32_t>(std::max(24, static_cast<int>(width() / 2)));
  const rust::Vec<double> curve = App::Get().session().eq_curve(
      ToRust(
          QString::fromUtf8(QJsonDocument(eq_).toJson(QJsonDocument::Compact))),
      highpass_, steps);
  if (curve.empty()) return;
  QPolygonF line;
  for (std::size_t i = 0; i < curve.size(); ++i) {
    const double x = static_cast<double>(i) /
                     static_cast<double>(curve.size() - 1) * width();
    line << QPointF(x, mid - curve.at(i) / kCurveRange * (mid - 2));
  }
  const bool live = eq_.value(QStringLiteral("on")).toBool() || highpass_ > 0;
  painter.setPen(
      QPen(live ? Token("keyframe") : Token("muted-foreground"), 1.5));
  painter.drawPolyline(line);
}

MixerPopover* MixerPopover::Open(QWidget* anchor, const QString& name,
                                 int width) {
  auto* popover = new MixerPopover(anchor->window(), name, width);
  FormHold::Instance()->Begin();
  FitHeightToWidth(popover);
  QPoint at = anchor->mapToGlobal(QPoint(0, anchor->height() + 4));
  if (const QScreen* screen = anchor->screen()) {
    const QRect room = screen->availableGeometry();
    at.setX(std::clamp(at.x(), room.left(), room.right() - popover->width()));
    if (at.y() + popover->height() > room.bottom()) {
      at.setY(anchor->mapToGlobal(QPoint(0, 0)).y() - popover->height() - 4);
    }
  }
  popover->move(at);
  FadeIn(popover, kMotionMs, kRisePx);
  return popover;
}

MixerPopover::MixerPopover(QWidget* parent, const QString& name, int width)
    : QFrame(parent, Qt::Popup) {
  setObjectName(QStringLiteral("Popover"));
  setAccessibleName(name);
  setAttribute(Qt::WA_DeleteOnClose);
  setAttribute(Qt::WA_StyledBackground);
  setFixedWidth(width);
  auto* layout = new QVBoxLayout(this);
  layout->setContentsMargins(12, 12, 12, 12);
  layout->setSpacing(10);
}

void MixerPopover::hideEvent(QHideEvent* event) {
  QFrame::hideEvent(event);
  if (ended_) return;
  ended_ = true;
  FormHold::Instance()->End();
}

void OpenEqPopover(QWidget* anchor, const QJsonObject& book, QJsonObject eq,
                   double highpass,
                   const std::function<void(const QJsonObject&)>& apply) {
  MixerPopover* popover =
      MixerPopover::Open(anchor, QStringLiteral("Equaliser"), kEqPopoverWidth);
  auto* layout = qobject_cast<QVBoxLayout*>(popover->layout());
  auto state = std::make_shared<QJsonObject>(std::move(eq));
  auto* curve =
      new EqCurve(*state, highpass, QSize(kEqPopoverWidth - 32, 90), popover);
  const auto send = [state, curve, apply] {
    curve->SetEq(*state);
    apply(*state);
  };
  layout->addWidget(SwitchHeader(
      QStringLiteral("Equaliser"), QStringLiteral("Equaliser on"),
      state->value(QStringLiteral("on")).toBool(),
      [state, send](bool on) {
        state->insert(QStringLiteral("on"), on);
        send();
      },
      popover));
  layout->addWidget(curve);
  auto* grid = new QGridLayout();
  grid->setHorizontalSpacing(8);
  const QJsonObject bounds = book.value(QStringLiteral("eqBand")).toObject();
  const QJsonArray defaults = book.value(QStringLiteral("defaultStrip"))
                                  .toObject()
                                  .value(QStringLiteral("eq"))
                                  .toObject()
                                  .value(QStringLiteral("bands"))
                                  .toArray();
  const QJsonArray bands = state->value(QStringLiteral("bands")).toArray();
  const bool on = state->value(QStringLiteral("on")).toBool();
  for (int i = 0; i < bands.size(); ++i) {
    const QJsonObject band = bands.at(i).toObject();
    const QJsonObject fallback = defaults.at(i).toObject();
    auto* column = new QVBoxLayout();
    column->setSpacing(4);
    auto* shape = new Combo(popover);
    shape->setAccessibleName(QStringLiteral("Band shape"));
    for (const auto& [value, label] :
         std::array<std::pair<const char*, const char*>, 3>{
             {{"lowshelf", "Low shelf"},
              {"peak", "Bell"},
              {"highshelf", "High shelf"}}}) {
      shape->addItem(QLatin1String(label), QLatin1String(value));
    }
    shape->setCurrentIndex(std::max(
        0, shape->findData(band.value(QStringLiteral("type")).toString())));
    const auto set_band = [state, send, i](const QString& field,
                                           const QJsonValue& value) {
      QJsonArray next = state->value(QStringLiteral("bands")).toArray();
      QJsonObject changed = next.at(i).toObject();
      changed.insert(field, value);
      next.replace(i, changed);
      state->insert(QStringLiteral("bands"), next);
      send();
    };
    QObject::connect(
        shape, &QComboBox::activated, popover, [shape, set_band](int) {
          set_band(QStringLiteral("type"), shape->currentData().toString());
        });
    column->addWidget(shape);
    KnobSpec freq = SpecFrom(bounds.value(QStringLiteral("freq")).toObject(),
                             QStringLiteral("Freq"), 1, QStringLiteral("hz"),
                             QStringLiteral("Hz"), true);
    freq.reset = fallback.value(QStringLiteral("freq")).toDouble();
    KnobSpec gain = SpecFrom(bounds.value(QStringLiteral("gain")).toObject(),
                             QStringLiteral("Gain"), 0.5, QStringLiteral("db"),
                             QStringLiteral("dB"));
    gain.reset = fallback.value(QStringLiteral("gain")).toDouble();
    KnobSpec width = SpecFrom(bounds.value(QStringLiteral("q")).toObject(),
                              QStringLiteral("Width"), 0.05,
                              QStringLiteral("fixed2"), QString(), true);
    width.reset = fallback.value(QStringLiteral("q")).toDouble();
    for (const auto& [spec, field] :
         std::array<std::pair<KnobSpec, const char*>, 3>{
             {{freq, "freq"}, {gain, "gain"}, {width, "q"}}}) {
      const QString key = QLatin1String(field);
      column->addWidget(MakeKnob(
          spec, band.value(key).toDouble(), kBandKnob,
          [set_band, key](double v) { set_band(key, v); }, !on, popover));
    }
    grid->addLayout(column, 0, i);
  }
  layout->addLayout(grid);
  FitHeightToWidth(popover);
}

void OpenCompPopover(QWidget* anchor, const QJsonObject& book, QJsonObject comp,
                     const std::function<void(const QJsonObject&)>& apply) {
  MixerPopover* popover = MixerPopover::Open(
      anchor, QStringLiteral("Compressor"), kCompPopoverWidth);
  auto* layout = qobject_cast<QVBoxLayout*>(popover->layout());
  auto state = std::make_shared<QJsonObject>(std::move(comp));
  layout->addWidget(SwitchHeader(
      QStringLiteral("Compressor"), QStringLiteral("Compressor on"),
      state->value(QStringLiteral("on")).toBool(),
      [state, apply](bool on) {
        state->insert(QStringLiteral("on"), on);
        apply(*state);
      },
      popover));
  const QJsonObject ranges = book.value(QStringLiteral("comp")).toObject();
  const bool on = state->value(QStringLiteral("on")).toBool();
  auto* row = new QHBoxLayout();
  row->setSpacing(4);
  struct Field {
    const char* key;
    const char* label;
    double step;
    const char* format;
    const char* unit;
    bool log;
  };
  for (const Field& field : std::array<Field, 5>{{
           {"threshold", "Threshold", 0.5, "db", "dB", false},
           {"ratio", "Ratio", 0.1, "ratio", "", true},
           {"attack", "Attack", 0.1, "ms", "ms", true},
           {"release", "Release", 1, "ms", "ms", true},
           {"makeup", "Makeup", 0.5, "db", "dB", false},
       }}) {
    const QString key = QLatin1String(field.key);
    row->addWidget(MakeKnob(
        SpecFrom(ranges.value(key).toObject(), QLatin1String(field.label),
                 field.step, QLatin1String(field.format),
                 QLatin1String(field.unit), field.log),
        state->value(key).toDouble(), kBandKnob,
        [state, apply, key](double v) {
          state->insert(key, v);
          apply(*state);
        },
        !on, popover));
  }
  layout->addLayout(row);
  auto* note =
      MakeLabel(QStringLiteral("Attack and release are in milliseconds."),
                "muted", popover);
  note->setWordWrap(true);
  layout->addWidget(note);
  FitHeightToWidth(popover);
}

void OpenDuckPopover(QWidget* anchor, const QJsonObject& book,
                     const QString& bus, const QJsonArray& buses,
                     QJsonObject duck,
                     const std::function<void(const QJsonObject&)>& apply) {
  MixerPopover* popover =
      MixerPopover::Open(anchor, QStringLiteral("Ducking"), kCompPopoverWidth);
  auto* layout = qobject_cast<QVBoxLayout*>(popover->layout());
  auto state = std::make_shared<QJsonObject>(std::move(duck));
  layout->addWidget(SwitchHeader(
      QStringLiteral("Lower this bus while others play"),
      QStringLiteral("Ducking on"), state->value(QStringLiteral("on")).toBool(),
      [state, apply](bool on) {
        state->insert(QStringLiteral("on"), on);
        apply(*state);
      },
      popover));
  layout->addWidget(MakeLabel(QStringLiteral("Listen to"), "muted", popover));
  for (const QJsonValue& entry : buses) {
    const QJsonObject other = entry.toObject();
    const QString id = other.value(QStringLiteral("id")).toString();
    if (id == bus) continue;
    auto* check =
        new QCheckBox(other.value(QStringLiteral("name")).toString(), popover);
    check->setChecked(
        state->value(QStringLiteral("keys")).toArray().contains(id));
    QObject::connect(
        check, &QCheckBox::toggled, popover, [state, apply, id](bool on) {
          QJsonArray keys = state->value(QStringLiteral("keys")).toArray();
          QJsonArray next;
          for (const QJsonValue& key : keys) {
            if (key.toString() != id) next.append(key);
          }
          if (on) next.append(id);
          state->insert(QStringLiteral("keys"), next);
          apply(*state);
        });
    layout->addWidget(check);
  }
  const QJsonObject ranges = book.value(QStringLiteral("duck")).toObject();
  const bool on = state->value(QStringLiteral("on")).toBool();
  auto* row = new QHBoxLayout();
  row->setSpacing(4);
  struct Field {
    const char* key;
    const char* label;
    double step;
    const char* format;
    bool log;
  };
  for (const Field& field : std::array<Field, 4>{{
           {"amount", "Amount", 0.5, "fixed1", false},
           {"attack", "Attack", 1, "ms", true},
           {"release", "Release", 1, "ms", true},
           {"ignoreShorterMs", "Ignore under", 50, "ignore", false},
       }}) {
    const QString key = QLatin1String(field.key);
    row->addWidget(MakeKnob(
        SpecFrom(ranges.value(key).toObject(), QLatin1String(field.label),
                 field.step, QLatin1String(field.format),
                 key == QStringLiteral("amount") ? QStringLiteral("dB")
                                                 : QStringLiteral("ms"),
                 field.log),
        state->value(key).toDouble(), kBandKnob,
        [state, apply, key](double v) {
          state->insert(key, v);
          apply(*state);
        },
        !on, popover));
  }
  layout->addLayout(row);
  auto* note = MakeLabel(
      QStringLiteral("Louder sounds lower it more. Attack and release are in "
                     "milliseconds. Sounds shorter than Ignore under don't "
                     "lower it, so quick clicks and pops don't make the music "
                     "pump."),
      "muted", popover);
  note->setWordWrap(true);
  layout->addWidget(note);
  FitHeightToWidth(popover);
}

}
