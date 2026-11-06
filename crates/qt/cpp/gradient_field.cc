#include "gradient_field.h"

#include <QDoubleValidator>
#include <QHBoxLayout>
#include <QJsonDocument>
#include <QKeyEvent>
#include <QLinearGradient>
#include <QLocale>
#include <QMouseEvent>
#include <QPainter>
#include <QPainterPath>
#include <QVBoxLayout>
#include <algorithm>
#include <cmath>

#include "app.h"
#include "color_picker.h"
#include "form_hold.h"
#include "opsis-qt/src/bridge.rs.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr qreal kInset = 8.0;
constexpr qreal kTrackHeight = 24.0;
constexpr qreal kHandle = 16.0;
constexpr qreal kHandleTop = 18.0;
constexpr double kFullPercent = 100.0;
constexpr int kNumberWidth = 64;
constexpr int kBigStep = 10;

QLineEdit* SmallNumber(const QString& label, double maximum, QWidget* parent) {
  auto* line = new QLineEdit(parent);
  line->setAccessibleName(label);
  line->setFixedWidth(kNumberWidth);
  line->setAlignment(Qt::AlignRight);
  auto* validator = new QDoubleValidator(-maximum, maximum, 0, line);
  validator->setLocale(QLocale::c());
  line->setValidator(validator);
  return line;
}

QString Number(double value) { return QString::number(std::lround(value)); }

QString JsonText(const QJsonObject& object) {
  return QString::fromUtf8(
      QJsonDocument(object).toJson(QJsonDocument::Compact));
}

}

StopBar::StopBar(QWidget* parent) : QWidget(parent) {
  setAccessibleName(QStringLiteral("Colour stops"));
  setFocusPolicy(Qt::StrongFocus);
  setMinimumHeight(sizeHint().height());
}

void StopBar::SetStops(std::vector<GradientStop> stops, bool can_add,
                       int selected) {
  stops_ = std::move(stops);
  can_add_ = can_add;
  selected_ = selected;
  setCursor(can_add ? Qt::DragCopyCursor : Qt::ArrowCursor);
  update();
}

QRectF StopBar::Track() const {
  return {kInset, 0.5, width() - kInset * 2, kTrackHeight};
}

QPointF StopBar::HandleOf(int index) const {
  const QRectF track = Track();
  const double at = stops_.at(static_cast<std::size_t>(index)).at;
  return {track.left() + at / kFullPercent * track.width(),
          kHandleTop + kHandle / 2};
}

double StopBar::AtOf(qreal x) const {
  const QRectF track = Track();
  return std::clamp((x - track.left()) / std::max<qreal>(1.0, track.width()),
                    0.0, 1.0) *
         kFullPercent;
}

int StopBar::HandleAt(const QPointF& point) const {
  for (int i = static_cast<int>(stops_.size()) - 1; i >= 0; --i) {
    const QPointF center = HandleOf(i);
    if (std::abs(point.x() - center.x()) <= kHandle / 2 + 2 &&
        point.y() >= kHandleTop - 2) {
      return i;
    }
  }
  return -1;
}

void StopBar::paintEvent(QPaintEvent*) {
  QPainter painter(this);
  painter.setRenderHint(QPainter::Antialiasing);
  const QRectF track = Track();
  const QRectF strip = track.adjusted(-kInset, 0, kInset, 0);
  PaintColorChip(&painter, strip, QString(), 6);
  QLinearGradient gradient(strip.topLeft(), strip.topRight());
  for (const GradientStop& stop : stops_) {
    if (const auto color = ParseColor(stop.color)) {
      const double at = (track.left() - strip.left() +
                         stop.at / kFullPercent * track.width()) /
                        strip.width();
      gradient.setColorAt(std::clamp(at, 0.0, 1.0), ToQColor(*color));
    }
  }
  QPainterPath path;
  path.addRoundedRect(strip, 6, 6);
  painter.fillPath(path, gradient);
  painter.setPen(QPen(App::Get().Token("input"), 1.0));
  painter.drawPath(path);
  for (int i = 0; i < static_cast<int>(stops_.size()); ++i) {
    const QPointF center = HandleOf(i);
    const QRectF handle(center.x() - kHandle / 2, kHandleTop, kHandle, kHandle);
    PaintColorChip(&painter, handle.adjusted(1, 1, -1, -1),
                   stops_.at(static_cast<std::size_t>(i)).color, 3);
    painter.setBrush(Qt::NoBrush);
    painter.setPen(QPen(Qt::white, 2.0));
    painter.drawRoundedRect(handle.adjusted(1, 1, -1, -1), 3, 3);
    if (i == selected_) {
      painter.setPen(QPen(App::Get().Token("ring"), 2.0));
      painter.drawRoundedRect(handle.adjusted(-1.5, -1.5, 1.5, 1.5), 4, 4);
    }
  }
}

void StopBar::mousePressEvent(QMouseEvent* event) {
  setFocus();
  const QPointF point = event->position();
  const int hit = HandleAt(point);
  if (hit >= 0) {
    dragging_ = hit;
    selected_ = hit;
    emit Selected(hit);
    FormHold::Instance()->Begin();
    update();
    return;
  }
  if (can_add_ && Track().adjusted(-kInset, 0, kInset, 0).contains(point)) {
    emit Added(AtOf(point.x()));
  }
}

void StopBar::mouseMoveEvent(QMouseEvent* event) {
  if (dragging_ < 0) return;
  emit Moved(dragging_, AtOf(event->position().x()));
}

void StopBar::mouseReleaseEvent(QMouseEvent*) {
  if (dragging_ < 0) return;
  const int index = dragging_;
  dragging_ = -1;
  emit Dropped(index);
  FormHold::Instance()->End();
}

void StopBar::keyPressEvent(QKeyEvent* event) {
  const int step = (event->modifiers() & Qt::ShiftModifier) != 0 ? kBigStep : 1;
  int direction = 0;
  if (event->key() == Qt::Key_Left) direction = -1;
  if (event->key() == Qt::Key_Right) direction = 1;
  if (direction == 0 || selected_ < 0 ||
      selected_ >= static_cast<int>(stops_.size())) {
    QWidget::keyPressEvent(event);
    return;
  }
  const double at = stops_.at(static_cast<std::size_t>(selected_)).at;
  emit Moved(selected_, at + direction * step);
  emit Dropped(selected_);
}

GradientField::GradientField(const QString& value, const QStringList& swatches,
                             int selected, std::function<void(int)> on_select,
                             QWidget* parent)
    : QWidget(parent),
      selected_(selected),
      on_select_(std::move(on_select)),
      editor_(new QWidget(this)),
      bar_(new StopBar(editor_)),
      kind_(new Segmented(editor_)),
      angle_box_(new QWidget(editor_)),
      angle_(SmallNumber(QStringLiteral("Angle"), 3600, angle_box_)),
      stop_color_(new ColorInput(QStringLiteral("Stop colour"), QString(),
                                 QString(), swatches, editor_)),
      at_(SmallNumber(QStringLiteral("At"), kFullPercent, editor_)),
      remove_(MakeIconButton(QStringLiteral("trash-2"),
                             QStringLiteral("Remove this colour stop"),
                             editor_)),
      note_(MakeLabel(QStringLiteral("This gradient can only be edited as "
                                     "text. Use two or more colours in a "
                                     "straight or round gradient to edit it "
                                     "here."),
                      "muted", this)),
      raw_(new QLineEdit(this)) {
  auto* column = new QVBoxLayout(this);
  column->setContentsMargins(0, 0, 0, 0);
  column->setSpacing(8);
  auto* rows = new QVBoxLayout(editor_);
  rows->setContentsMargins(0, 0, 0, 0);
  rows->setSpacing(8);
  rows->addWidget(bar_);
  auto* kind_row = new QHBoxLayout();
  kind_row->setSpacing(8);
  kind_->SetChoices({{QStringLiteral("linear"), QStringLiteral("Straight")},
                     {QStringLiteral("radial"), QStringLiteral("Round")}});
  kind_->SetOutline();
  kind_row->addWidget(kind_);
  auto* angle_row = new QHBoxLayout(angle_box_);
  angle_row->setContentsMargins(0, 0, 0, 0);
  angle_row->setSpacing(4);
  angle_row->addWidget(MakeLabel(QStringLiteral("Angle"), "muted", angle_box_));
  angle_row->addWidget(angle_);
  angle_row->addWidget(MakeLabel(QStringLiteral("°"), "muted", angle_box_));
  kind_row->addWidget(angle_box_);
  kind_row->addStretch(1);
  rows->addLayout(kind_row);
  auto* stop_row = new QHBoxLayout();
  stop_row->setSpacing(6);
  stop_row->addWidget(stop_color_, 1);
  stop_row->addWidget(MakeLabel(QStringLiteral("At"), "muted", editor_));
  stop_row->addWidget(at_);
  stop_row->addWidget(MakeLabel(QStringLiteral("%"), "muted", editor_));
  stop_row->addWidget(remove_);
  rows->addLayout(stop_row);
  column->addWidget(editor_);
  note_->setWordWrap(true);
  column->addWidget(note_);
  raw_->setObjectName(QStringLiteral("Mono"));
  raw_->setAccessibleName(QStringLiteral("Gradient as text"));
  column->addWidget(raw_);

  connect(bar_, &StopBar::Selected, this, &GradientField::Select);
  connect(bar_, &StopBar::Added, this, [this](double at) {
    Edit(QJsonObject{{QStringLiteral("op"), QStringLiteral("addStop")},
                     {QStringLiteral("at"), at}});
  });
  connect(bar_, &StopBar::Moved, this, [this](int index, double at) {
    Edit(QJsonObject{{QStringLiteral("op"), QStringLiteral("stopAt")},
                     {QStringLiteral("index"), index},
                     {QStringLiteral("at"), at}});
  });
  connect(bar_, &StopBar::Dropped, this, [this](int index) {
    Edit(QJsonObject{{QStringLiteral("op"), QStringLiteral("sort")},
                     {QStringLiteral("index"), index}});
  });
  connect(kind_, &Segmented::Changed, this, [this](const QString& kind) {
    Edit(QJsonObject{{QStringLiteral("op"), QStringLiteral("kind")},
                     {QStringLiteral("kind"), kind}});
  });
  connect(angle_, &QLineEdit::editingFinished, this, [this] {
    if (!angle_->isModified()) return;
    angle_->setModified(false);
    Edit(QJsonObject{{QStringLiteral("op"), QStringLiteral("angle")},
                     {QStringLiteral("angle"), angle_->text().toDouble()}});
  });
  connect(at_, &QLineEdit::editingFinished, this, [this] {
    if (!at_->isModified()) return;
    at_->setModified(false);
    Edit(QJsonObject{{QStringLiteral("op"), QStringLiteral("stopAt")},
                     {QStringLiteral("index"), selected_},
                     {QStringLiteral("at"), at_->text().toDouble()}});
    Edit(QJsonObject{{QStringLiteral("op"), QStringLiteral("sort")},
                     {QStringLiteral("index"), selected_}});
  });
  connect(
      stop_color_, &ColorInput::Changed, this, [this](const QString& color) {
        Edit(QJsonObject{{QStringLiteral("op"), QStringLiteral("stopColor")},
                         {QStringLiteral("index"), selected_},
                         {QStringLiteral("color"), color}});
      });
  connect(remove_, &QToolButton::clicked, this, [this] {
    Edit(QJsonObject{{QStringLiteral("op"), QStringLiteral("removeStop")},
                     {QStringLiteral("index"), selected_}});
  });
  connect(raw_, &QLineEdit::editingFinished, this, [this] {
    if (!raw_->isModified()) return;
    raw_->setModified(false);
    Load(raw_->text().trimmed());
    emit Changed(raw_->text().trimmed());
  });
  Load(value);
}

void GradientField::Load(const QString& css) {
  const bridge::GradientInfo info =
      App::Get().session().gradient_info(ToRust(css));
  css_ = css;
  valid_ = info.valid;
  linear_ = info.linear;
  degrees_ = info.angle;
  can_add_ = info.can_add;
  can_remove_ = info.can_remove;
  stops_.clear();
  for (const auto& stop : info.stops) {
    stops_.push_back(GradientStop{ToQ(stop.color), stop.at});
  }
  Show();
}

void GradientField::Edit(const QJsonObject& edit) {
  const bridge::GradientInfo info =
      App::Get().session().edit_gradient(ToRust(css_), ToRust(JsonText(edit)));
  if (!info.valid) return;
  css_ = ToQ(info.css);
  valid_ = true;
  linear_ = info.linear;
  degrees_ = info.angle;
  can_add_ = info.can_add;
  can_remove_ = info.can_remove;
  stops_.clear();
  for (const auto& stop : info.stops) {
    stops_.push_back(GradientStop{ToQ(stop.color), stop.at});
  }
  if (info.selected >= 0) Select(info.selected);
  Show();
  emit Changed(css_);
}

void GradientField::Select(int index) {
  selected_ = index;
  if (on_select_) on_select_(index);
  Show();
}

void GradientField::Show() {
  editor_->setVisible(valid_ && !stops_.empty());
  note_->setVisible(!valid_ && !css_.isEmpty());
  if (!raw_->hasFocus()) raw_->setText(css_);
  if (stops_.empty()) return;
  selected_ = std::clamp(selected_, 0, static_cast<int>(stops_.size()) - 1);
  bar_->SetStops(stops_, can_add_, selected_);
  kind_->SetValue(linear_ ? QStringLiteral("linear")
                          : QStringLiteral("radial"));
  angle_box_->setVisible(linear_);
  if (!angle_->hasFocus()) angle_->setText(Number(degrees_));
  const GradientStop& stop = stops_.at(static_cast<std::size_t>(selected_));
  stop_color_->SetValue(stop.color);
  stop_color_->setAccessibleName(
      QStringLiteral("Stop %1 colour").arg(selected_ + 1));
  if (!at_->hasFocus()) at_->setText(Number(stop.at));
  remove_->setEnabled(can_remove_);
}

}
