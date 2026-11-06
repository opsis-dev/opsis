#include "node_list.h"

#include <QApplication>
#include <QFrame>
#include <QHBoxLayout>
#include <QHeaderView>
#include <QMimeData>
#include <QPainter>
#include <QScrollBar>
#include <QToolButton>
#include <QVBoxLayout>

#include "app.h"
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
constexpr int kCardBadge = 32;
constexpr int kBadge = 20;
constexpr int kPopoverWidth = 256;
constexpr int kPopoverHeight = 400;

QIcon BadgeIcon(const QString& glyph, const QColor& color) {
  const qreal ratio = qApp->devicePixelRatio();
  QPixmap pixmap(QSize(kBadge, kBadge) * ratio);
  pixmap.setDevicePixelRatio(ratio);
  pixmap.fill(Qt::transparent);
  QPainter painter(&pixmap);
  painter.setRenderHint(QPainter::Antialiasing);
  painter.setPen(Qt::NoPen);
  painter.setBrush(color);
  painter.drawRoundedRect(QRectF(0, 0, kBadge, kBadge), 4, 4);
  painter.drawPixmap(4, 4, App::Get().Glyph(glyph, Qt::black, 12));
  painter.end();
  return {pixmap};
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
        0, BadgeIcon(ToQ(entry.glyph), QColor::fromString(ToQ(entry.color))));
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
    button->setIcon(
        BadgeIcon(ToQ(category.icon), QColor::fromString(ToQ(category.color))));
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

void NodeList::Hover(QTreeWidgetItem* item) {
  hover_timer_.stop();
  if (card_ != nullptr) card_->close();
  hovered_ = item;
  if (item != nullptr && !item->data(0, kTypeRole).toString().isEmpty()) {
    hover_timer_.start();
  }
}

void NodeList::ShowCard() {
  if (hovered_ == nullptr) return;
  const int index = hovered_->data(0, kEntryRole).toInt();
  if (index < 0 || index >= static_cast<int>(entries_.size())) return;
  const bridge::PaletteEntry& entry =
      entries_.at(static_cast<std::size_t>(index));
  auto* card = new QFrame(this, Qt::ToolTip | Qt::FramelessWindowHint);
  card->setObjectName(QStringLiteral("HoverCard"));
  card->setAttribute(Qt::WA_DeleteOnClose);
  card->setAttribute(Qt::WA_StyledBackground);
  card->setFixedWidth(kCardWidth);
  auto* layout = new QVBoxLayout(card);
  layout->setContentsMargins(16, 16, 16, 16);
  layout->setSpacing(8);
  auto* head = new QHBoxLayout();
  head->setSpacing(8);
  auto* badge = new QLabel(card);
  badge->setPixmap(
      BadgeIcon(ToQ(entry.glyph), QColor::fromString(ToQ(entry.color)))
          .pixmap(kCardBadge, kCardBadge));
  head->addWidget(badge);
  auto* names = new QVBoxLayout();
  names->setSpacing(0);
  auto* title = MakeLabel(ToQ(entry.title), nullptr, card);
  QFont bold = title->font();
  bold.setWeight(QFont::Medium);
  title->setFont(bold);
  names->addWidget(title);
  names->addWidget(MakeLabel(ToQ(entry.category), "small", card));
  head->addLayout(names, 1);
  layout->addLayout(head);
  if (!entry.description.empty()) {
    auto* about = MakeLabel(ToQ(entry.description), "small", card);
    about->setWordWrap(true);
    layout->addWidget(about);
  }
  auto* flow = MakeLabel(
      ToQ(entry.inputs) + QStringLiteral("  →  ") + ToQ(entry.outputs), "small",
      card);
  flow->setWordWrap(true);
  layout->addWidget(flow);
  const QRect row = tree_->visualItemRect(hovered_);
  card->adjustSize();
  card->move(tree_->viewport()->mapToGlobal(
      QPoint(tree_->viewport()->width() + kCardOffset, row.top())));
  card->show();
  card_ = card;
}

}
