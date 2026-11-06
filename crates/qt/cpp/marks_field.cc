#include "marks_field.h"

#include <QButtonGroup>
#include <QFontMetricsF>
#include <QHBoxLayout>
#include <QJsonDocument>
#include <QKeyEvent>
#include <QLineF>
#include <QMouseEvent>
#include <QPainter>
#include <QPainterPath>
#include <QPushButton>
#include <QStyle>
#include <QToolButton>
#include <QtMath>
#include <algorithm>
#include <array>
#include <cmath>
#include <memory>
#include <utility>

#include "app.h"
#include "opsis-qt/src/bridge.rs.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr double kHeadLength = 4.5;
constexpr double kHeadWidth = 3.2;
constexpr double kSmallestHead = 12.0;
constexpr double kHitWidth = 12.0;
constexpr qreal kHandleRadius = 6.0;
constexpr double kDefaultLine = 6.0;
constexpr double kDefaultSize = 32.0;
constexpr int kBoldFrom = 600;
constexpr double kStepText = 0.6;
constexpr double kTextPad = 0.3;
constexpr double kTextCorner = 0.25;
constexpr int kToolIcon = 16;

struct Tool {
  const char* name;
  const char* label;
  const char* icon;
};

constexpr std::array<Tool, 6> kTools = {{
    {"select", "Select and move", "mouse-pointer-2"},
    {"arrow", "Arrow", "arrow-up-right"},
    {"rectangle", "Box", "square"},
    {"oval", "Oval", "circle"},
    {"step", "Step number", "hash"},
    {"text", "Text", "type"},
}};

QString KindOf(const QJsonObject& mark) {
  const QString kind = mark.value(QStringLiteral("kind")).toString();
  for (const Tool& tool : kTools) {
    if (kind == QLatin1String(tool.name) && kind != QStringLiteral("select")) {
      return kind;
    }
  }
  return QStringLiteral("arrow");
}

bool TwoEnded(const QString& kind) {
  return kind == QStringLiteral("arrow") ||
         kind == QStringLiteral("rectangle") || kind == QStringLiteral("oval");
}

QString KindWord(const QString& kind) {
  if (kind == QStringLiteral("rectangle")) return QStringLiteral("Box");
  if (kind == QStringLiteral("oval")) return QStringLiteral("Oval");
  if (kind == QStringLiteral("step")) return QStringLiteral("Step");
  if (kind == QStringLiteral("text")) return QStringLiteral("Text");
  return QStringLiteral("Arrow");
}

MarkLine LineOf(const QJsonObject& mark) {
  return MarkLine{mark.value(QStringLiteral("x")).toDouble(),
                  mark.value(QStringLiteral("y")).toDouble(),
                  mark.value(QStringLiteral("x2")).toDouble(),
                  mark.value(QStringLiteral("y2")).toDouble()};
}

bridge::MarkPointsView ToView(const MarkLine& line) {
  return bridge::MarkPointsView{line.x, line.y, line.x2, line.y2};
}

MarkLine FromView(const bridge::MarkPointsView& view) {
  return MarkLine{view.x, view.y, view.x2, view.y2};
}

QJsonObject WithLine(QJsonObject mark, const MarkLine& line) {
  mark.insert(QStringLiteral("x"), line.x);
  mark.insert(QStringLiteral("y"), line.y);
  mark.insert(QStringLiteral("x2"), line.x2);
  mark.insert(QStringLiteral("y2"), line.y2);
  return mark;
}

double NumberIn(const QJsonObject& mark, const char* key, double fallback) {
  const QJsonValue value = mark.value(QLatin1String(key));
  return value.isDouble() ? value.toDouble() : fallback;
}

QString TextIn(const QJsonObject& mark, const char* key) {
  return mark.value(QLatin1String(key)).toString();
}

double SegmentDistance(const QPointF& point, const QPointF& a,
                       const QPointF& b) {
  const QPointF ab = b - a;
  const double length = QPointF::dotProduct(ab, ab);
  const double t =
      length <= 0
          ? 0.0
          : std::clamp(QPointF::dotProduct(point - a, ab) / length, 0.0, 1.0);
  return QLineF(point, a + ab * t).length();
}

QFont MarkFont(const QJsonObject& mark, double pixels) {
  QFont font(TextIn(mark, "font").isEmpty() ? QStringLiteral("Inter")
                                            : TextIn(mark, "font"));
  font.setPixelSize(std::max(1, static_cast<int>(std::lround(pixels))));
  font.setBold(NumberIn(mark, "weight", 700) >= kBoldFrom);
  return font;
}

QRectF TextBox(const QJsonObject& mark, const QPointF& at, double size) {
  const QString text = TextIn(mark, "text").isEmpty() ? QStringLiteral("Text")
                                                      : TextIn(mark, "text");
  const QFontMetricsF metrics(MarkFont(mark, size));
  const double pad =
      TextIn(mark, "textBackground").isEmpty() ? 0 : size * kTextPad;
  return {at.x() - pad, at.y() - pad, metrics.horizontalAdvance(text) + pad * 2,
          metrics.height() + pad * 2};
}

}

MarksFrame::MarksFrame(QJsonArray marks, std::vector<std::optional<int>> steps,
                       int first_step, QJsonObject fresh, int most,
                       double aspect, const QString& picture, QWidget* parent)
    : PictureFrame(aspect, picture, parent),
      marks_(std::move(marks)),
      steps_(std::move(steps)),
      first_step_(first_step),
      fresh_(std::move(fresh)),
      most_(most) {
  setMouseTracking(true);
}

void MarksFrame::SetTool(const QString& tool) {
  tool_ = tool;
  setCursor(tool == QStringLiteral("select") ? Qt::ArrowCursor
                                             : Qt::CrossCursor);
  update();
}

void MarksFrame::Select(int index) {
  selected_ = index;
  update();
}

double MarksFrame::Scale() const {
  const int natural = PictureWidth();
  return natural > 0 && width() > 0 ? static_cast<double>(width()) / natural
                                    : 1.0;
}

QPointF MarksFrame::ToWidget(double x, double y) const {
  const QRectF frame = QRectF(rect()).adjusted(0.5, 0.5, -0.5, -0.5);
  return {frame.left() + x * frame.width(), frame.top() + y * frame.height()};
}

QPointF MarksFrame::FractionAt(const QPointF& point) const {
  return {point.x() / std::max(1, width()), point.y() / std::max(1, height())};
}

MarkLine MarksFrame::Shown(int index) const {
  if (index == selected_ &&
      (gesture_ == Gesture::kMove || gesture_ == Gesture::kEnd)) {
    return live_;
  }
  return LineOf(marks_.at(index).toObject());
}

void MarksFrame::PaintMark(QPainter* painter, const QJsonObject& mark,
                           const MarkLine& line,
                           std::optional<int> step) const {
  const QString kind = KindOf(mark);
  const double scale = Scale();
  const QColor colour = QColor::fromString(TextIn(mark, "color").isEmpty()
                                               ? QStringLiteral("#ef4444")
                                               : TextIn(mark, "color"));
  const double stroke = NumberIn(mark, "lineWidth", kDefaultLine);
  const double size = NumberIn(mark, "size", kDefaultSize) * scale;
  const double fill = NumberIn(mark, "fill", 0);
  const QPointF a = ToWidget(line.x, line.y);
  const QPointF b = ToWidget(line.x2, line.y2);
  painter->save();
  painter->setRenderHint(QPainter::Antialiasing);
  if (kind == QStringLiteral("arrow")) {
    const double head = NumberIn(mark, "headSize", 1);
    const double length =
        std::max(stroke * kHeadLength, kSmallestHead) * head * scale;
    const double half =
        std::max(stroke * kHeadWidth, kSmallestHead * 0.7) * head * scale / 2;
    const QLineF shaft(a, b);
    const double angle = std::atan2(b.y() - a.y(), b.x() - a.x());
    const QPointF base = b - QPointF(std::cos(angle), std::sin(angle)) *
                                 std::min(length, shaft.length());
    const QPointF normal(-std::sin(angle) * half, std::cos(angle) * half);
    painter->setPen(QPen(colour, stroke * scale, Qt::SolidLine, Qt::FlatCap));
    painter->drawLine(a, base);
    QPainterPath tip;
    tip.moveTo(b);
    tip.lineTo(base + normal);
    tip.lineTo(base - normal);
    tip.closeSubpath();
    painter->fillPath(tip, colour);
  } else if (kind == QStringLiteral("rectangle") ||
             kind == QStringLiteral("oval")) {
    const QRectF box = QRectF(a, b).normalized();
    const double radius = NumberIn(mark, "radius", 0) * scale;
    if (fill > 0) {
      QColor tint = colour;
      tint.setAlphaF(static_cast<float>(std::clamp(fill, 0.0, 1.0)));
      painter->setPen(Qt::NoPen);
      painter->setBrush(tint);
      if (kind == QStringLiteral("oval")) {
        painter->drawEllipse(box);
      } else {
        painter->drawRoundedRect(box, radius, radius);
      }
    }
    painter->setPen(QPen(colour, stroke * scale));
    painter->setBrush(Qt::NoBrush);
    if (kind == QStringLiteral("oval")) {
      painter->drawEllipse(box);
    } else {
      painter->drawRoundedRect(box, radius, radius);
    }
  } else if (kind == QStringLiteral("step")) {
    painter->setPen(Qt::NoPen);
    painter->setBrush(colour);
    painter->drawEllipse(a, size / 2, size / 2);
    painter->setFont(MarkFont(mark, size * kStepText));
    painter->setPen(QColor::fromString(TextIn(mark, "textColor").isEmpty()
                                           ? QStringLiteral("#ffffff")
                                           : TextIn(mark, "textColor")));
    painter->drawText(QRectF(a.x() - size / 2, a.y() - size / 2, size, size),
                      Qt::AlignCenter,
                      step ? QString::number(*step) : QString());
  } else {
    const QRectF box = TextBox(mark, a, size);
    const QString background = TextIn(mark, "textBackground");
    if (!background.isEmpty()) {
      painter->setPen(Qt::NoPen);
      painter->setBrush(QColor::fromString(background));
      painter->drawRoundedRect(box, size * kTextCorner, size * kTextCorner);
    }
    painter->setFont(MarkFont(mark, size));
    painter->setPen(colour);
    const double pad = background.isEmpty() ? 0 : size * kTextPad;
    painter->drawText(box.adjusted(pad, pad, -pad, -pad),
                      Qt::AlignLeft | Qt::AlignTop,
                      TextIn(mark, "text").isEmpty() ? QStringLiteral("Text")
                                                     : TextIn(mark, "text"));
  }
  painter->restore();
}

void MarksFrame::paintEvent(QPaintEvent*) {
  QPainter painter(this);
  PaintPicture(&painter);
  for (int i = 0; i < marks_.size(); ++i) {
    const auto at = static_cast<std::size_t>(i);
    PaintMark(&painter, marks_.at(i).toObject(), Shown(i),
              at < steps_.size() ? steps_.at(at) : std::nullopt);
  }
  if (gesture_ == Gesture::kDraw) {
    QJsonObject mark = fresh_;
    mark.insert(QStringLiteral("kind"), tool_);
    const int taken = static_cast<int>(std::count_if(
        steps_.begin(), steps_.end(),
        [](const std::optional<int>& step) { return step.has_value(); }));
    PaintMark(&painter, mark, live_, first_step_ + taken);
  }
  if (selected_ < 0 || selected_ >= marks_.size()) return;
  const QJsonObject mark = marks_.at(selected_).toObject();
  const MarkLine line = Shown(selected_);
  painter.setRenderHint(QPainter::Antialiasing);
  if (tool_ == QStringLiteral("select") && TwoEnded(KindOf(mark))) {
    painter.setPen(QPen(QColor(0x18, 0x18, 0x1b), 1.5));
    painter.setBrush(Qt::white);
    painter.drawEllipse(ToWidget(line.x, line.y), kHandleRadius, kHandleRadius);
    painter.drawEllipse(ToWidget(line.x2, line.y2), kHandleRadius,
                        kHandleRadius);
  }
}

int MarksFrame::HandleAt(const QPointF& point) const {
  if (selected_ < 0 || selected_ >= marks_.size() ||
      tool_ != QStringLiteral("select") ||
      !TwoEnded(KindOf(marks_.at(selected_).toObject()))) {
    return -1;
  }
  const MarkLine line = LineOf(marks_.at(selected_).toObject());
  if (QLineF(point, ToWidget(line.x2, line.y2)).length() <= kHandleRadius + 2) {
    return 1;
  }
  if (QLineF(point, ToWidget(line.x, line.y)).length() <= kHandleRadius + 2) {
    return 0;
  }
  return -1;
}

int MarksFrame::MarkAt(const QPointF& point) const {
  const double scale = Scale();
  for (int i = static_cast<int>(marks_.size()) - 1; i >= 0; --i) {
    const QJsonObject mark = marks_.at(i).toObject();
    const QString kind = KindOf(mark);
    const MarkLine line = LineOf(mark);
    const QPointF a = ToWidget(line.x, line.y);
    const QPointF b = ToWidget(line.x2, line.y2);
    const double reach =
        std::max(NumberIn(mark, "lineWidth", kDefaultLine) * scale, kHitWidth) /
        2;
    const double size = NumberIn(mark, "size", kDefaultSize) * scale;
    bool hit = false;
    if (kind == QStringLiteral("arrow")) {
      hit = SegmentDistance(point, a, b) <= reach;
    } else if (kind == QStringLiteral("rectangle") ||
               kind == QStringLiteral("oval")) {
      hit = QRectF(a, b)
                .normalized()
                .adjusted(-reach, -reach, reach, reach)
                .contains(point);
    } else if (kind == QStringLiteral("step")) {
      hit = QLineF(point, a).length() <= size / 2;
    } else {
      hit = TextBox(mark, a, size).contains(point);
    }
    if (hit) return i;
  }
  return -1;
}

void MarksFrame::mousePressEvent(QMouseEvent* event) {
  setFocus();
  const QPointF point = event->position();
  start_ = FractionAt(point);
  if (tool_ == QStringLiteral("select")) {
    const int handle = HandleAt(point);
    if (handle >= 0) {
      gesture_ = Gesture::kEnd;
      second_end_ = handle == 1;
      origin_ = LineOf(marks_.at(selected_).toObject());
      live_ = origin_;
      return;
    }
    const int hit = MarkAt(point);
    selected_ = hit;
    emit Selected(hit);
    if (hit >= 0) {
      gesture_ = Gesture::kMove;
      origin_ = LineOf(marks_.at(hit).toObject());
      live_ = origin_;
    }
    update();
    return;
  }
  if (marks_.size() >= most_) return;
  gesture_ = Gesture::kDraw;
  live_ = MarkLine{start_.x(), start_.y(), start_.x(), start_.y()};
  update();
}

void MarksFrame::mouseMoveEvent(QMouseEvent* event) {
  const QPointF at = FractionAt(event->position());
  auto& session = App::Get().session();
  switch (gesture_) {
    case Gesture::kMove:
      live_ = FromView(session.mark_moved(ToView(origin_), at.x() - start_.x(),
                                          at.y() - start_.y()));
      break;
    case Gesture::kEnd:
      live_ = FromView(
          session.mark_end(ToView(origin_), second_end_, at.x(), at.y()));
      break;
    case Gesture::kDraw:
      live_ = MarkLine{start_.x(), start_.y(), at.x(), at.y()};
      break;
    case Gesture::kNone:
      if (tool_ == QStringLiteral("select")) {
        setCursor(HandleAt(event->position()) >= 0
                      ? Qt::CrossCursor
                      : (MarkAt(event->position()) >= 0 ? Qt::SizeAllCursor
                                                        : Qt::ArrowCursor));
      }
      return;
  }
  update();
}

void MarksFrame::mouseReleaseEvent(QMouseEvent* event) {
  const Gesture gesture = gesture_;
  gesture_ = Gesture::kNone;
  const QPointF at = FractionAt(event->position());
  auto& session = App::Get().session();
  if (gesture == Gesture::kMove || gesture == Gesture::kEnd) {
    const bool still = live_.x == origin_.x && live_.y == origin_.y &&
                       live_.x2 == origin_.x2 && live_.y2 == origin_.y2;
    if (still) {
      update();
      return;
    }
    QJsonArray next = marks_;
    next.replace(selected_, WithLine(next.at(selected_).toObject(), live_));
    emit Changed(next, selected_);
    return;
  }
  if (gesture != Gesture::kDraw) return;
  const bridge::MarkDrawnView drawn =
      session.mark_drawn(ToRust(tool_), start_.x(), start_.y(), at.x(), at.y());
  update();
  if (!drawn.kept) return;
  QJsonObject mark = WithLine(fresh_, FromView(drawn.points));
  mark.insert(QStringLiteral("kind"), tool_);
  if (tool_ == QStringLiteral("text")) {
    mark.insert(QStringLiteral("text"), QStringLiteral("Text"));
  }
  QJsonArray next = marks_;
  next.append(mark);
  emit Changed(next, static_cast<int>(next.size()) - 1);
}

void MarksFrame::keyPressEvent(QKeyEvent* event) {
  if ((event->key() == Qt::Key_Delete || event->key() == Qt::Key_Backspace) &&
      selected_ >= 0 && selected_ < marks_.size()) {
    QJsonArray next = marks_;
    next.removeAt(selected_);
    emit Changed(next, -1);
    return;
  }
  PictureFrame::keyPressEvent(event);
}

QWidget* MakeMarksField(const bridge::FieldView& field,
                        QHash<QString, QVBoxLayout*>* bodies,
                        const MarksState& state,
                        const std::function<void(const MarksState&)>& on_state,
                        const CommitValue& commit) {
  auto* box = new QWidget();
  auto* column = new QVBoxLayout(box);
  column->setContentsMargins(0, 0, 0, 0);
  column->setSpacing(8);
  const QJsonObject extra =
      QJsonDocument::fromJson(ToQ(field.extra_json).toUtf8()).object();
  const QJsonArray marks =
      QJsonDocument::fromJson(ToQ(field.value_json).toUtf8()).array();
  std::vector<std::optional<int>> steps;
  for (const QJsonValue& step :
       extra.value(QStringLiteral("steps")).toArray()) {
    steps.push_back(step.isDouble() ? std::optional<int>(step.toInt())
                                    : std::nullopt);
  }
  const QString reference = extra.value(QStringLiteral("picture")).toString();
  const QString picture =
      reference.isEmpty()
          ? QString()
          : ToQ(App::Get().session().file_path(ToRust(reference)));
  const int most = static_cast<int>(std::min<std::uint32_t>(
      field.max_items, static_cast<std::uint32_t>(INT32_MAX)));
  const int selected =
      state.selected < static_cast<int>(marks.size()) ? state.selected : -1;

  auto* tools = new QWidget(box);
  auto* tool_row = new QHBoxLayout(tools);
  tool_row->setContentsMargins(0, 0, 0, 0);
  tool_row->setSpacing(4);
  auto* group = new QButtonGroup(tools);
  group->setExclusive(true);
  for (const Tool& tool : kTools) {
    auto* button = MakeIconButton(QLatin1String(tool.icon),
                                  QLatin1String(tool.label), tools, kToolIcon);
    button->setCheckable(true);
    button->setObjectName(QStringLiteral("MarkTool"));
    button->setChecked(state.tool == QLatin1String(tool.name));
    button->setProperty("tool", QLatin1String(tool.name));
    group->addButton(button);
    tool_row->addWidget(button);
  }
  tool_row->addStretch(1);
  column->addWidget(tools);

  auto* frame = new MarksFrame(
      marks, steps, extra.value(QStringLiteral("firstStep")).toInt(1),
      QJsonDocument::fromJson(ToQ(field.new_item_json).toUtf8()).object(), most,
      extra.value(QStringLiteral("aspect")).toDouble(), picture, box);
  frame->setAccessibleName(ToQ(field.label));
  frame->SetTool(state.tool);
  frame->Select(selected);
  column->addWidget(frame);

  auto* list = new QWidget(box);
  auto* flow = new FlowLayout(list, 4);
  flow->setContentsMargins(0, 0, 0, 0);
  auto entries = std::make_shared<std::vector<QPushButton*>>();
  auto items = std::make_shared<std::vector<QFrame*>>();
  for (int i = 0; i < marks.size(); ++i) {
    const QString kind = KindOf(marks.at(i).toObject());
    const auto at = static_cast<std::size_t>(i);
    const std::optional<int> step =
        at < steps.size() ? steps.at(at) : std::nullopt;
    auto* button = MakeButton(
        QStringLiteral("%1. %2%3")
            .arg(i + 1)
            .arg(KindWord(kind))
            .arg(step ? QStringLiteral(" %1").arg(*step) : QString()),
        "ghost", QString(), list);
    entries->push_back(button);
    flow->addWidget(button);
  }
  column->addWidget(list);

  const QString key = ToQ(field.key);
  for (int i = 0; i < marks.size(); ++i) {
    auto* item = new QFrame(box);
    item->setObjectName(QStringLiteral("MarkItem"));
    item->setAttribute(Qt::WA_StyledBackground);
    auto* rows = new QVBoxLayout(item);
    rows->setContentsMargins(12, 12, 12, 12);
    rows->setSpacing(12);
    bodies->insert(key + QLatin1Char('/') + QString::number(i), rows);
    auto* remove = MakeButton(QStringLiteral("Remove"), "outline",
                              QStringLiteral("trash-2"), item);
    QObject::connect(remove, &QPushButton::clicked, box,
                     [marks, i, commit, on_state] {
                       QJsonArray next = marks;
                       next.removeAt(i);
                       on_state(MarksState{-1, QStringLiteral("select")});
                       commit(QString::fromUtf8(
                           QJsonDocument(next).toJson(QJsonDocument::Compact)));
                     });
    auto* footer = new QHBoxLayout();
    footer->addWidget(remove);
    footer->addStretch(1);
    rows->addLayout(footer);
    items->push_back(item);
    column->addWidget(item);
  }

  auto* help = MakeLabel(
      static_cast<int>(marks.size()) >= most
          ? QStringLiteral("Up to %1 marks. Remove one to add another.")
                .arg(most)
          : (marks.isEmpty()
                 ? QStringLiteral("Pick a kind above and drag on the picture.")
                 : QStringLiteral("Select a mark to change it, drag it to move "
                                  "it, or press Delete to remove it.")),
      "muted", box);
  help->setWordWrap(true);
  column->addWidget(help);

  const auto current_tool = [group] {
    QAbstractButton* checked = group->checkedButton();
    return checked != nullptr ? checked->property("tool").toString()
                              : QStringLiteral("select");
  };
  const auto show = [entries, items](int chosen) {
    for (std::size_t i = 0; i < items->size(); ++i) {
      const bool on = static_cast<int>(i) == chosen;
      items->at(i)->setVisible(on);
      QPushButton* entry = entries->at(i);
      entry->setProperty("variant", on ? "secondary" : "ghost");
      entry->style()->unpolish(entry);
      entry->style()->polish(entry);
    }
  };
  show(selected);
  for (std::size_t i = 0; i < entries->size(); ++i) {
    const int index = static_cast<int>(i);
    QObject::connect(entries->at(i), &QPushButton::clicked, box,
                     [frame, group, show, on_state, index] {
                       for (QAbstractButton* button : group->buttons()) {
                         if (button->property("tool").toString() ==
                             QStringLiteral("select")) {
                           button->setChecked(true);
                         }
                       }
                       frame->SetTool(QStringLiteral("select"));
                       frame->Select(index);
                       show(index);
                       on_state(MarksState{index, QStringLiteral("select")});
                     });
  }
  QObject::connect(group, &QButtonGroup::buttonClicked, box,
                   [frame, on_state](QAbstractButton* button) {
                     const QString tool = button->property("tool").toString();
                     frame->SetTool(tool);
                     on_state(MarksState{frame->selected(), tool});
                   });
  QObject::connect(frame, &MarksFrame::Selected, box,
                   [show, on_state, current_tool](int index) {
                     show(index);
                     on_state(MarksState{index, current_tool()});
                   });
  QObject::connect(
      frame, &MarksFrame::Changed, box,
      [commit, on_state, current_tool](const QJsonArray& next, int chosen) {
        on_state(MarksState{chosen, current_tool()});
        commit(QString::fromUtf8(
            QJsonDocument(next).toJson(QJsonDocument::Compact)));
      });
  return box;
}

}
