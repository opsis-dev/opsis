#include "widgets.h"

#include <QAbstractItemView>
#include <QEasingCurve>
#include <QHBoxLayout>
#include <QLayout>
#include <QPainter>
#include <QPainterPath>
#include <QTextLayout>

#include "app.h"
#include "motion.h"

namespace opsis {

namespace {

constexpr int kComboMinimumChars = 4;
constexpr int kComboPopupPadding = 32;

constexpr int kSwitchWidth = 32;
constexpr int kSwitchHeight = 18;
constexpr int kSwitchMs = 120;
constexpr int kOutlineGap = 6;
constexpr double kChevronHalf = 3.5;
constexpr double kChevronPen = 1.5;
constexpr double kChevronInset = 14.0;

}

FlowLayout::FlowLayout(QWidget* parent, int spacing)
    : QLayout(parent), gap_(spacing) {
  setContentsMargins(0, 0, 0, 0);
}

FlowLayout::~FlowLayout() {
  while (QLayoutItem* item = takeAt(0)) delete item;
}

void FlowLayout::addItem(QLayoutItem* item) { items_.append(item); }

int FlowLayout::count() const { return static_cast<int>(items_.size()); }

QLayoutItem* FlowLayout::itemAt(int index) const {
  return index >= 0 && index < items_.size() ? items_.at(index) : nullptr;
}

QLayoutItem* FlowLayout::takeAt(int index) {
  return index >= 0 && index < items_.size() ? items_.takeAt(index) : nullptr;
}

int FlowLayout::heightForWidth(int width) const {
  const QMargins margins = contentsMargins();
  return Arrange(QRect(0, 0, width, 0).marginsRemoved(margins), false) +
         margins.top() + margins.bottom();
}

QSize FlowLayout::minimumSize() const {
  QSize size;
  for (const QLayoutItem* item : items_) {
    size = size.expandedTo(item->minimumSize());
  }
  const QMargins margins = contentsMargins();
  return size.grownBy(margins);
}

QSize FlowLayout::sizeHint() const {
  int width = 0;
  int height = 0;
  for (const QLayoutItem* item : items_) {
    const QSize hint = item->sizeHint();
    width += hint.width() + (width > 0 ? gap_ : 0);
    height = std::max(height, hint.height());
  }
  const QMargins margins = contentsMargins();
  return {width + margins.left() + margins.right(),
          height + margins.top() + margins.bottom()};
}

void FlowLayout::setGeometry(const QRect& rect) {
  QLayout::setGeometry(rect);
  Arrange(rect.marginsRemoved(contentsMargins()), true);
}

int FlowLayout::Arrange(const QRect& rect, bool apply) const {
  int x = rect.x();
  int y = rect.y();
  int row = 0;
  for (QLayoutItem* item : items_) {
    const QSize hint = item->sizeHint();
    if (x + hint.width() > rect.right() + 1 && row > 0) {
      x = rect.x();
      y += row + gap_;
      row = 0;
    }
    if (apply) item->setGeometry(QRect(QPoint(x, y), hint));
    x += hint.width() + gap_;
    row = std::max(row, hint.height());
  }
  return y + row - rect.y();
}

ClampedLabel::ClampedLabel(const QString& text, int lines, QWidget* parent)
    : QLabel(parent), full_(text), lines_(lines) {
  setWordWrap(false);
  setAlignment(Qt::AlignLeft | Qt::AlignTop);
  setToolTip(text);
  QSizePolicy policy(QSizePolicy::Ignored, QSizePolicy::Preferred);
  policy.setHeightForWidth(true);
  setSizePolicy(policy);
}

QString ClampedLabel::Clamped(int width) const {
  if (width <= 0) return full_;
  QTextLayout layout(full_, font());
  layout.beginLayout();
  QStringList rows;
  int start = 0;
  for (int row = 0; row < lines_; ++row) {
    QTextLine line = layout.createLine();
    if (!line.isValid()) break;
    line.setLineWidth(width);
    start = line.textStart();
    if (row + 1 == lines_) {
      rows << fontMetrics().elidedText(full_.mid(start), Qt::ElideRight, width);
    } else {
      rows << full_.mid(start, line.textLength()).trimmed();
    }
  }
  layout.endLayout();
  return rows.join(QLatin1Char('\n'));
}

int ClampedLabel::heightForWidth(int width) const {
  const int rows =
      static_cast<int>(Clamped(width).count(QLatin1Char('\n'))) + 1;
  return rows * fontMetrics().lineSpacing();
}

QSize ClampedLabel::sizeHint() const {
  return {fontMetrics().horizontalAdvance(full_) / lines_,
          lines_ * fontMetrics().lineSpacing()};
}

QSize ClampedLabel::minimumSizeHint() const {
  return {0, fontMetrics().lineSpacing()};
}

void ClampedLabel::resizeEvent(QResizeEvent* event) {
  QLabel::resizeEvent(event);
  setText(Clamped(width()));
}

Combo::Combo(QWidget* parent) : QComboBox(parent) {
  setSizeAdjustPolicy(QComboBox::AdjustToMinimumContentsLengthWithIcon);
  setMinimumContentsLength(kComboMinimumChars);
  setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Fixed);
}

void Combo::showPopup() {
  const int wanted = view()->sizeHintForColumn(0) + kComboPopupPadding;
  view()->setMinimumWidth(std::max(width(), wanted));
  QComboBox::showPopup();
}

void Combo::paintEvent(QPaintEvent* event) {
  QComboBox::paintEvent(event);
  QPainter painter(this);
  painter.setRenderHint(QPainter::Antialiasing);
  painter.setPen(QPen(App::Get().Token("muted-foreground"), kChevronPen,
                      Qt::SolidLine, Qt::RoundCap, Qt::RoundJoin));
  const QPointF middle(width() - kChevronInset, height() / 2.0);
  QPainterPath path;
  path.moveTo(middle + QPointF(-kChevronHalf, -kChevronHalf / 2));
  path.lineTo(middle + QPointF(0, kChevronHalf / 2));
  path.lineTo(middle + QPointF(kChevronHalf, -kChevronHalf / 2));
  painter.drawPath(path);
}

ToggleSwitch::ToggleSwitch(QWidget* parent) : QAbstractButton(parent) {
  setCheckable(true);
  setCursor(Qt::PointingHandCursor);
  setFocusPolicy(Qt::StrongFocus);
  animation_.setEasingCurve(EaseOut());
  connect(&animation_, &QVariantAnimation::valueChanged, this,
          [this](const QVariant& value) {
            knob_ = value.toReal();
            update();
          });
}

QSize ToggleSwitch::sizeHint() const { return {kSwitchWidth, kSwitchHeight}; }

void ToggleSwitch::SetCheckedQuietly(bool checked) {
  const QSignalBlocker blocker(this);
  setChecked(checked);
  animation_.stop();
  knob_ = checked ? 1.0 : 0.0;
  update();
}

void ToggleSwitch::checkStateSet() { Animate(); }

void ToggleSwitch::nextCheckState() {
  QAbstractButton::nextCheckState();
  Animate();
}

void ToggleSwitch::Animate() {
  animation_.stop();
  animation_.setDuration(MotionMs(kSwitchMs));
  animation_.setStartValue(knob_);
  animation_.setEndValue(isChecked() ? 1.0 : 0.0);
  animation_.start();
}

void ToggleSwitch::paintEvent(QPaintEvent*) {
  App& app = App::Get();
  QPainter painter(this);
  painter.setRenderHint(QPainter::Antialiasing);
  const QRectF track(0.5, 0.5, kSwitchWidth - 1, kSwitchHeight - 1);
  const QColor off = app.Token("input");
  const QColor on = app.Token("primary");
  QColor fill;
  const auto blend = [this](float from, float to) {
    return static_cast<float>(from + (to - from) * knob_);
  };
  fill.setRgbF(blend(off.redF(), on.redF()), blend(off.greenF(), on.greenF()),
               blend(off.blueF(), on.blueF()),
               blend(off.alphaF(), on.alphaF()));
  painter.setPen(hasFocus() ? QPen(app.Token("ring"), 1) : Qt::NoPen);
  painter.setBrush(fill);
  painter.drawRoundedRect(track, kSwitchHeight / 2.0, kSwitchHeight / 2.0);
  const qreal diameter = kSwitchHeight - 4;
  const qreal x = 2 + (kSwitchWidth - diameter - 4) * knob_;
  painter.setPen(Qt::NoPen);
  painter.setBrush(knob_ > 0.5 ? app.Token("primary-foreground")
                               : app.Token("foreground"));
  painter.drawEllipse(QRectF(x, 2, diameter, diameter));
}

Segmented::Segmented(QWidget* parent) : QFrame(parent) {
  setObjectName(QStringLiteral("Segmented"));
  auto* layout = new FlowLayout(this, 2);
  layout->setContentsMargins(2, 2, 2, 2);
  QSizePolicy policy(QSizePolicy::Preferred, QSizePolicy::Preferred);
  policy.setHeightForWidth(true);
  setSizePolicy(policy);
  group_.setExclusive(true);
  connect(&group_, &QButtonGroup::idClicked, this, [this](int id) {
    emit Changed(values_.at(static_cast<std::size_t>(id)));
  });
}

void Segmented::SetChoices(
    const std::vector<std::pair<QString, QString>>& choices) {
  values_.clear();
  int id = 0;
  for (const auto& [value, label] : choices) {
    auto* button = new QPushButton(label, this);
    button->setProperty("variant", "segment");
    button->setCheckable(true);
    button->setCursor(Qt::PointingHandCursor);
    group_.addButton(button, id++);
    layout()->addWidget(button);
    values_.push_back(value);
  }
}

void Segmented::SetOutline() {
  setObjectName(QStringLiteral("SegmentedOutline"));
  layout()->setContentsMargins(0, 0, 0, 0);
  static_cast<FlowLayout*>(layout())->SetGap(kOutlineGap);
  for (QAbstractButton* button : group_.buttons()) {
    button->setProperty("variant", "toggle-outline");
  }
}

void Segmented::SetValue(const QString& value) {
  for (std::size_t i = 0; i < values_.size(); ++i) {
    if (values_.at(i) == value) {
      group_.button(static_cast<int>(i))->setChecked(true);
    }
  }
}

QString Segmented::value() const {
  const int id = group_.checkedId();
  return id < 0 ? QString() : values_.at(static_cast<std::size_t>(id));
}

Card::Card(const QString& title, const QString& description, QWidget* parent)
    : QFrame(parent) {
  setProperty("role", "card");
  auto* layout = new QVBoxLayout(this);
  layout->setContentsMargins(20, 18, 20, 20);
  layout->setSpacing(4);
  layout->addWidget(MakeLabel(title, "title", this));
  if (!description.isEmpty()) {
    auto* text = MakeLabel(description, "muted", this);
    text->setWordWrap(true);
    layout->addWidget(text);
  }
  layout->addSpacing(12);
  body_ = new QVBoxLayout();
  body_->setSpacing(16);
  layout->addLayout(body_);
}

QLabel* MakeLabel(const QString& text, const char* role, QWidget* parent) {
  auto* label = new QLabel(text, parent);
  if (role != nullptr) label->setProperty("role", role);
  return label;
}

QFrame* MakeRule(QWidget* parent) {
  auto* rule = new QFrame(parent);
  rule->setProperty("role", "rule");
  return rule;
}

QToolButton* MakeIconButton(const QString& icon, const QString& tip,
                            QWidget* parent, int size) {
  auto* button = new QToolButton(parent);
  button->setIcon(App::Get().Icon(icon, App::Get().Token("foreground"), size));
  button->setIconSize(QSize(size, size));
  button->setToolTip(tip);
  button->setCursor(Qt::PointingHandCursor);
  button->setAutoRaise(true);
  return button;
}

QPushButton* MakeButton(const QString& text, const char* variant,
                        const QString& icon, QWidget* parent) {
  auto* button = new QPushButton(text, parent);
  if (variant != nullptr) button->setProperty("variant", variant);
  if (!icon.isEmpty()) {
    const bool primary = variant != nullptr && QByteArray(variant) == "primary";
    button->setIcon(App::Get().Icon(
        icon, App::Get().Token(primary ? "primary-foreground" : "foreground"),
        16));
  }
  button->setCursor(Qt::PointingHandCursor);
  return button;
}

QWidget* MakeField(const QString& label, QWidget* control,
                   const QString& description, QWidget* parent) {
  auto* field = new QWidget(parent);
  auto* layout = new QVBoxLayout(field);
  layout->setContentsMargins(0, 0, 0, 0);
  layout->setSpacing(6);
  auto* title = MakeLabel(label, nullptr, field);
  QFont font = title->font();
  font.setWeight(QFont::Medium);
  title->setFont(font);
  layout->addWidget(title);
  layout->addWidget(control);
  if (!description.isEmpty()) {
    auto* text = MakeLabel(description, "muted", field);
    text->setWordWrap(true);
    layout->addWidget(text);
  }
  return field;
}

QWidget* MakeSwitchField(const QString& label, ToggleSwitch* toggle,
                         const QString& description, QWidget* parent) {
  auto* field = new QWidget(parent);
  auto* row = new QHBoxLayout(field);
  row->setContentsMargins(0, 0, 0, 0);
  row->setSpacing(12);
  auto* texts = new QVBoxLayout();
  texts->setSpacing(4);
  auto* title = MakeLabel(label, nullptr, field);
  QFont font = title->font();
  font.setWeight(QFont::Medium);
  title->setFont(font);
  texts->addWidget(title);
  if (!description.isEmpty()) {
    auto* text = MakeLabel(description, "muted", field);
    text->setWordWrap(true);
    texts->addWidget(text);
  }
  row->addLayout(texts, 1);
  row->addWidget(toggle, 0, Qt::AlignTop);
  return field;
}

QLabel* MakeBadge(const QString& glyph, const QColor& color, int size,
                  QWidget* parent) {
  App& app = App::Get();
  const qreal ratio = parent != nullptr ? parent->devicePixelRatioF() : 1.0;
  QPixmap pixmap(QSize(size, size) * ratio);
  pixmap.setDevicePixelRatio(ratio);
  pixmap.fill(Qt::transparent);
  {
    QPainter painter(&pixmap);
    painter.setRenderHint(QPainter::Antialiasing);
    painter.setPen(Qt::NoPen);
    painter.setBrush(color);
    painter.drawRoundedRect(QRectF(0, 0, size, size), size / 4.0, size / 4.0);
    const int inner = size * 7 / 12;
    painter.drawPixmap((size - inner) / 2, (size - inner) / 2,
                       app.Glyph(glyph, Qt::black, inner));
  }
  auto* label = new QLabel(parent);
  label->setPixmap(pixmap);
  label->setFixedSize(size, size);
  return label;
}

}
