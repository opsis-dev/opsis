#include "node_list.h"

#include <QApplication>
#include <QFrame>
#include <QHeaderView>
#include <QMimeData>
#include <QPainter>
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

NodeTree::NodeTree(QWidget* parent) : QTreeWidget(parent) {
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

  const auto palette = App::Get().session().palette();
  QTreeWidgetItem* section = nullptr;
  QString current;
  QFont heading = font();
  heading.setPixelSize(11);
  heading.setWeight(QFont::DemiBold);
  int shown = 0;
  for (const auto& entry : palette) {
    if (!category.isEmpty() && ToQ(entry.category_id) != category) continue;
    const QString group = ToQ(entry.category);
    if (category.isEmpty() && group != current) {
      current = group;
      section = new QTreeWidgetItem(tree_, {group.toUpper()});
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
    item->setToolTip(0, ToQ(entry.description));
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
  auto* show = MakeIconButton(QStringLiteral("panel-left-open"),
                              QStringLiteral("Show node list"), this);
  layout->addWidget(show, 0, Qt::AlignHCenter);
  connect(show, &QToolButton::clicked, this, &CategoryRail::ShowList);
  auto* rule = MakeRule(this);
  rule->setFixedWidth(24);
  layout->addWidget(rule, 0, Qt::AlignHCenter);
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

void NodeList::Filter(const QString& text) {
  const QString needle = text.trimmed();
  const auto matches = [&needle](QTreeWidgetItem* item) {
    const bool match =
        needle.isEmpty() || item->text(0).contains(needle, Qt::CaseInsensitive);
    item->setHidden(!match);
    return match;
  };
  for (int i = 0; i < tree_->topLevelItemCount(); ++i) {
    QTreeWidgetItem* top = tree_->topLevelItem(i);
    if (top->childCount() == 0) {
      matches(top);
      continue;
    }
    int shown = 0;
    for (int j = 0; j < top->childCount(); ++j) {
      shown += matches(top->child(j)) ? 1 : 0;
    }
    top->setHidden(shown == 0);
  }
}

}
