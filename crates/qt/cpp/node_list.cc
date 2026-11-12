#include "node_list.h"

#include <QApplication>
#include <QFrame>
#include <QGridLayout>
#include <QHBoxLayout>
#include <QHeaderView>
#include <QMimeData>
#include <QPainter>
#include <QScrollBar>
#include <QToolButton>
#include <QVBoxLayout>

#include "app.h"
#include "look_glyph.h"
#include "motion.h"
#include "opsis-qt/src/bridge.rs.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr QLatin1StringView kNodeMime("application/x-opsis-node");
constexpr int kTypeRole = Qt::UserRole + 1;
constexpr int kSearchRole = Qt::UserRole + 2;
constexpr int kEntryRole = Qt::UserRole + 3;
constexpr int kHoverDelayMs = 500;
constexpr int kCardWidth = 280;
constexpr int kCardOffset = 12;
constexpr int kCardBadge = 40;
constexpr int kDotBox = 14;
constexpr double kDotRadius = 4.0;
constexpr double kRingInner = 3.0;
constexpr double kRingOuter = 6.0;
constexpr double kStackDot = 1.75;
constexpr double kStackGap = 4.5;
constexpr int kInDot = 0;
constexpr int kInName = 1;
constexpr int kGap = 2;
constexpr int kOutName = 3;
constexpr int kOutDot = 4;
constexpr int kBadge = 20;
constexpr int kPopoverWidth = 256;
constexpr int kPopoverHeight = 400;

constexpr double kBadgeRadius = 4;
constexpr double kBadgeInset = 4;

QIcon BadgeIcon(const Look& look, const QColor& color,
                const LookFrame& frame = {}, int size = kBadge) {
  const qreal ratio = qApp->devicePixelRatio();
  const double scale = static_cast<double>(size) / kBadge;
  const double radius = kBadgeRadius * scale;
  const double inset = kBadgeInset * scale;
  QPixmap pixmap(QSize(size, size) * ratio);
  pixmap.setDevicePixelRatio(ratio);
  pixmap.fill(Qt::transparent);
  QPainter painter(&pixmap);
  painter.setRenderHint(QPainter::Antialiasing);
  painter.setRenderHint(QPainter::SmoothPixmapTransform);
  painter.setPen(Qt::NoPen);
  painter.setBrush(color);
  const QRectF badge(0, 0, size, size);
  painter.drawRoundedRect(badge, radius, radius);
  PaintLookGlyph(&painter, badge, badge.adjusted(inset, inset, -inset, -inset),
                 look, frame, color);
  painter.end();
  return {pixmap};
}

Look LookOf(const bridge::PaletteEntry& entry) {
  return {ToQ(entry.glyph), ToQ(entry.motion), ToQ(entry.morph)};
}

class PortDot : public QWidget {
 public:
  PortDot(const bridge::PalettePort& port, QWidget* parent)
      : QWidget(parent),
        color_(QColor::fromString(ToQ(port.color))),
        shape_(port.shape) {
    setObjectName(QStringLiteral("PortDot"));
    setFixedSize(kDotBox, kDotBox);
  }

 protected:
  void paintEvent(QPaintEvent*) override {
    QPainter painter(this);
    painter.setRenderHint(QPainter::Antialiasing);
    const QPointF middle = QRectF(rect()).center();
    painter.setPen(Qt::NoPen);
    painter.setBrush(color_);
    switch (shape_) {
      case bridge::PortShape::Numbered:
        for (int step = -1; step <= 1; ++step) {
          painter.drawEllipse(middle + QPointF(0, step * kStackGap), kStackDot,
                              kStackDot);
        }
        break;
      case bridge::PortShape::Many:
        painter.drawEllipse(middle, kRingInner, kRingInner);
        painter.setPen(QPen(color_, 1.0));
        painter.setBrush(Qt::NoBrush);
        painter.drawEllipse(middle, kRingOuter, kRingOuter);
        break;
      default:
        painter.drawEllipse(middle, kDotRadius, kDotRadius);
        break;
    }
  }

 private:
  QColor color_;
  bridge::PortShape shape_;
};

void AddPort(QGridLayout* grid, const bridge::PalettePort& port, int row,
             bool output, QWidget* card) {
  auto* name =
      MakeLabel(ToQ(port.label), port.optional ? "small" : "port", card);
  auto* dot = new PortDot(port, card);
  if (output) {
    grid->addWidget(name, row, kOutName, Qt::AlignRight | Qt::AlignVCenter);
    grid->addWidget(dot, row, kOutDot);
    return;
  }
  grid->addWidget(dot, row, kInDot);
  if (port.note.empty()) {
    grid->addWidget(name, row, kInName);
    return;
  }
  auto* named = new QHBoxLayout();
  named->setSpacing(6);
  named->addWidget(name);
  named->addWidget(MakeLabel(ToQ(port.note), "small", card));
  named->addStretch(1);
  grid->addLayout(named, row, kInName);
}

QGridLayout* PortRows(const bridge::PaletteEntry& entry, QWidget* card) {
  auto* grid = new QGridLayout();
  grid->setHorizontalSpacing(6);
  grid->setVerticalSpacing(6);
  grid->setColumnStretch(kGap, 1);
  int row = 0;
  for (const auto& port : entry.inputs) AddPort(grid, port, row++, false, card);
  row = 0;
  for (const auto& port : entry.outputs) AddPort(grid, port, row++, true, card);
  return grid;
}

}

NodeTree::NodeTree(QWidget* parent)
    : QTreeWidget(parent), sticky_(new QLabel(viewport())) {
  sticky_->setObjectName(QStringLiteral("StickyHeading"));
  sticky_->setAttribute(Qt::WA_StyledBackground);
  sticky_->hide();
  setMouseTracking(true);
  connect(verticalScrollBar(), &QScrollBar::valueChanged, this,
          &NodeTree::UpdateSticky);
  setHeaderHidden(true);
  setRootIsDecorated(false);
  setIndentation(0);
  setIconSize(QSize(kBadge, kBadge));
  setDragEnabled(true);
  setDragDropMode(QAbstractItemView::DragOnly);
  setSelectionMode(QAbstractItemView::SingleSelection);
  setVerticalScrollMode(QAbstractItemView::ScrollPerPixel);
  setUniformRowHeights(true);
}

QStringList NodeTree::mimeTypes() const { return {kNodeMime}; }

void NodeTree::UpdateSticky() {
  QTreeWidgetItem* top = itemAt(QPoint(1, 1));
  QTreeWidgetItem* section = top != nullptr ? top->parent() : nullptr;
  if (section == nullptr) {
    sticky_->hide();
    return;
  }
  sticky_->setText(section->text(0));
  sticky_->setGeometry(0, 0, viewport()->width(),
                       visualItemRect(section).height());
  sticky_->show();
  sticky_->raise();
}

void NodeTree::resizeEvent(QResizeEvent* event) {
  QTreeWidget::resizeEvent(event);
  UpdateSticky();
}

void NodeTree::leaveEvent(QEvent* event) {
  QTreeWidget::leaveEvent(event);
  emit itemEntered(nullptr, 0);
}

QMimeData* NodeTree::mimeData(const QList<QTreeWidgetItem*>& items) const {
  auto* data = new QMimeData();
  if (!items.isEmpty()) {
    data->setData(kNodeMime,
                  items.first()->data(0, kTypeRole).toString().toUtf8());
  }
  return data;
}

NodeList::NodeList(const QString& category, const QString& label,
                   QWidget* parent)
    : QWidget(parent), search_(new QLineEdit(this)), tree_(new NodeTree(this)) {
  setObjectName(QStringLiteral("Panel"));
  setAttribute(Qt::WA_StyledBackground);
  setMinimumWidth(220);
  auto* layout = new QVBoxLayout(this);
  layout->setContentsMargins(8, 8, 8, 8);
  layout->setSpacing(8);
  search_->setProperty("role", "search");
  search_->addAction(App::Get().Icon(QStringLiteral("search"),
                                     App::Get().Token("muted-foreground")),
                     QLineEdit::LeadingPosition);
  search_->setClearButtonEnabled(true);
  layout->addWidget(search_);
  layout->addWidget(tree_, 1);
  empty_ = MakeLabel(QStringLiteral("No nodes found."), "muted", this);
  empty_->setAlignment(Qt::AlignHCenter | Qt::AlignTop);
  empty_->hide();
  layout->addWidget(empty_, 1);
  hover_timer_.setSingleShot(true);
  hover_timer_.setInterval(kHoverDelayMs);
  connect(&hover_timer_, &QTimer::timeout, this, &NodeList::ShowCard);
  connect(tree_, &QTreeWidget::itemEntered, this, &NodeList::Hover);
  connect(&looks_, &LookPlayer::Changed, this, &NodeList::Repaint);
  connect(tree_->verticalScrollBar(), &QScrollBar::valueChanged, this,
          [this] { Hover(nullptr); });

  const auto palette = App::Get().session().palette();
  QTreeWidgetItem* section = nullptr;
  QString current;
  QFont heading = font();
  heading.setPixelSize(12);
  heading.setWeight(QFont::Medium);
  int shown = 0;
  for (const auto& entry : palette) {
    if (!category.isEmpty() && ToQ(entry.category_id) != category) continue;
    const QString group = ToQ(entry.category);
    if (category.isEmpty() && group != current) {
      current = group;
      section = new QTreeWidgetItem(tree_, {group});
      section->setFlags(Qt::ItemIsEnabled);
      section->setFont(0, heading);
      section->setForeground(0, App::Get().Token("muted-foreground"));
      section->setExpanded(true);
    }
    auto* item = section != nullptr
                     ? new QTreeWidgetItem(section, {ToQ(entry.title)})
                     : new QTreeWidgetItem(tree_, {ToQ(entry.title)});
    item->setIcon(
        0, BadgeIcon(LookOf(entry), QColor::fromString(ToQ(entry.color))));
    rows_.insert(ToQ(entry.node_type), item);
    item->setData(0, kTypeRole, ToQ(entry.node_type));
    item->setData(0, kSearchRole,
                  ToQ(entry.title) + QLatin1Char(' ') + group +
                      QLatin1Char(' ') + ToQ(entry.description));
    item->setData(0, kEntryRole, static_cast<int>(entries_.size()));
    entries_.push_back(entry);
    item->setFlags(Qt::ItemIsEnabled | Qt::ItemIsSelectable |
                   Qt::ItemIsDragEnabled);
    ++shown;
  }
  search_->setPlaceholderText(
      category.isEmpty() ? QStringLiteral("Search %1 nodes…").arg(shown)
                         : QStringLiteral("Search %1…").arg(label.toLower()));
  if (!category.isEmpty()) {
    tree_->setRootIsDecorated(false);
    tree_->setIndentation(0);
  }
  tree_->expandAll();
  tree_->UpdateSticky();
  connect(search_, &QLineEdit::textChanged, this, &NodeList::Filter);
  connect(tree_, &QTreeWidget::itemClicked, this,
          [this](QTreeWidgetItem* item) {
            const QString type = item->data(0, kTypeRole).toString();
            if (!type.isEmpty()) emit AddRequested(type);
          });
}

CategoryRail::CategoryRail(QWidget* parent) : QWidget(parent) {
  setObjectName(QStringLiteral("CategoryRail"));
  setAttribute(Qt::WA_StyledBackground);
  setFixedWidth(kWidth);
  auto* layout = new QVBoxLayout(this);
  layout->setContentsMargins(0, 8, 0, 8);
  layout->setSpacing(4);
  for (const auto& category : App::Get().session().categories()) {
    auto* button = new QToolButton(this);
    button->setIcon(BadgeIcon(Look{ToQ(category.icon), {}, {}},
                              QColor::fromString(ToQ(category.color))));
    button->setIconSize(QSize(kBadge, kBadge));
    button->setToolTip(ToQ(category.label));
    button->setCursor(Qt::PointingHandCursor);
    button->setAutoRaise(true);
    layout->addWidget(button, 0, Qt::AlignHCenter);
    const QString id = ToQ(category.id);
    const QString label = ToQ(category.label);
    connect(button, &QToolButton::clicked, this,
            [this, button, id, label] { OpenCategory(button, id, label); });
  }
  layout->addStretch(1);
}

void CategoryRail::OpenCategory(QWidget* anchor, const QString& id,
                                const QString& label) {
  auto* popup = new QFrame(this, Qt::Popup);
  popup->setObjectName(QStringLiteral("Popover"));
  popup->setAttribute(Qt::WA_DeleteOnClose);
  popup->setAttribute(Qt::WA_StyledBackground);
  auto* layout = new QVBoxLayout(popup);
  layout->setContentsMargins(0, 0, 0, 0);
  auto* list = new NodeList(id, label, popup);
  layout->addWidget(list);
  connect(list, &NodeList::AddRequested, this,
          [this, popup](const QString& type) {
            emit AddRequested(type);
            popup->close();
          });
  popup->resize(kPopoverWidth, kPopoverHeight);
  popup->move(anchor->mapToGlobal(QPoint(anchor->width() + 12, 0)));
  FadeIn(popup, kMotionMs, kRisePx);
}

void NodeList::SetPlaceholder(const QString& text) {
  search_->setPlaceholderText(text);
  search_->setFocus();
}

void NodeList::Filter(const QString& text) {
  const QString needle = text.trimmed();
  int shown = 0;
  const auto matches = [&needle, &shown](QTreeWidgetItem* item) {
    const bool match =
        needle.isEmpty() || item->data(0, kSearchRole)
                                .toString()
                                .contains(needle, Qt::CaseInsensitive);
    item->setHidden(!match);
    shown += match ? 1 : 0;
    return match;
  };
  for (int i = 0; i < tree_->topLevelItemCount(); ++i) {
    QTreeWidgetItem* top = tree_->topLevelItem(i);
    if (top->childCount() == 0) {
      matches(top);
      continue;
    }
    bool any = false;
    for (int j = 0; j < top->childCount(); ++j) {
      any = matches(top->child(j)) || any;
    }
    top->setHidden(!any);
  }
  tree_->setVisible(shown > 0);
  empty_->setVisible(shown == 0);
  tree_->UpdateSticky();
}

QString NodeList::Problems() const {
  QStringList problems;
  for (int i = 0; i < tree_->topLevelItemCount(); ++i) {
    const QString text = tree_->topLevelItem(i)->text(0);
    if (tree_->topLevelItem(i)->childCount() > 0 && text == text.toUpper() &&
        text != text.toLower()) {
      problems << QStringLiteral("heading '%1' is upper case").arg(text);
    }
  }
  return problems.join(QStringLiteral("; "));
}

const bridge::PaletteEntry* NodeList::EntryOf(QTreeWidgetItem* item) const {
  if (item == nullptr) return nullptr;
  const int index = item->data(0, kEntryRole).toInt();
  if (item->data(0, kTypeRole).toString().isEmpty() || index < 0 ||
      index >= static_cast<int>(entries_.size())) {
    return nullptr;
  }
  return &entries_.at(static_cast<std::size_t>(index));
}

void NodeList::Repaint(const QString& look_id) {
  if (!card_type_.isEmpty() && look_id == CardLook(card_type_)) {
    RepaintCard();
    return;
  }
  QTreeWidgetItem* item = rows_.value(look_id);
  const bridge::PaletteEntry* entry = EntryOf(item);
  if (entry == nullptr) return;
  item->setIcon(0,
                BadgeIcon(LookOf(*entry), QColor::fromString(ToQ(entry->color)),
                          looks_.FrameOf(look_id)));
}

void NodeList::RepaintCard() {
  const bridge::PaletteEntry* entry = EntryOf(rows_.value(card_type_));
  if (card_badge_ == nullptr || entry == nullptr) return;
  card_badge_->setPixmap(
      BadgeIcon(LookOf(*entry), QColor::fromString(ToQ(entry->color)),
                looks_.FrameOf(CardLook(card_type_)), kCardBadge)
          .pixmap(kCardBadge, kCardBadge));
}

void NodeList::CloseCard() {
  if (!card_type_.isEmpty()) looks_.Leave(CardLook(card_type_));
  card_type_.clear();
  if (card_ != nullptr) card_->close();
}

void NodeList::Hover(QTreeWidgetItem* item) {
  hover_timer_.stop();
  CloseCard();
  const bridge::PaletteEntry* entry = EntryOf(item);
  const QString type = entry != nullptr ? ToQ(entry->node_type) : QString();
  if (type != playing_) {
    if (!playing_.isEmpty()) looks_.Leave(playing_);
    playing_ = type;
    if (entry != nullptr) looks_.Enter(type, LookOf(*entry));
  }
  hovered_ = item;
  if (item != nullptr && !item->data(0, kTypeRole).toString().isEmpty()) {
    hover_timer_.start();
  }
}

void NodeList::ShowCard() {
  if (hovered_ == nullptr) return;
  const bridge::PaletteEntry* found = EntryOf(hovered_);
  if (found == nullptr) return;
  const bridge::PaletteEntry& entry = *found;
  auto* card = new QFrame(this, Qt::ToolTip | Qt::FramelessWindowHint);
  card->setObjectName(QStringLiteral("HoverCard"));
  card->setAttribute(Qt::WA_DeleteOnClose);
  card->setAttribute(Qt::WA_StyledBackground);
  card->setFixedWidth(kCardWidth);
  auto* layout = new QVBoxLayout(card);
  layout->setContentsMargins(16, 16, 16, 16);
  layout->setSpacing(12);
  auto* head = new QHBoxLayout();
  head->setSpacing(12);
  auto* badge = new QLabel(card);
  badge->setFixedSize(kCardBadge, kCardBadge);
  head->addWidget(badge, 0, Qt::AlignTop);
  card_badge_ = badge;
  card_type_ = ToQ(entry.node_type);
  looks_.Enter(CardLook(card_type_), LookOf(entry));
  RepaintCard();
  auto* words = new QVBoxLayout();
  words->setSpacing(4);
  auto* title = MakeLabel(ToQ(entry.title), nullptr, card);
  title->setObjectName(QStringLiteral("HoverCardTitle"));
  title->setWordWrap(true);
  QFont bold = title->font();
  bold.setWeight(QFont::Medium);
  title->setFont(bold);
  words->addWidget(title);
  if (!entry.description.empty()) {
    auto* about = MakeLabel(ToQ(entry.description), "small", card);
    about->setWordWrap(true);
    words->addWidget(about);
  }
  head->addLayout(words, 1);
  layout->addLayout(head);
  if (!entry.inputs.empty() || !entry.outputs.empty()) {
    layout->addWidget(MakeRule(card));
    layout->addLayout(PortRows(entry, card));
  }
  const QRect row = tree_->visualItemRect(hovered_);
  FitHeightToWidth(card);
  card->move(tree_->viewport()->mapToGlobal(
      QPoint(tree_->viewport()->width() + kCardOffset, row.top())));
  card->show();
  card_ = card;
}

}
