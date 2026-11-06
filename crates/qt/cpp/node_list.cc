#include "node_list.h"

#include <QApplication>
#include <QHeaderView>
#include <QPainter>
#include <QMimeData>
#include <QVBoxLayout>

#include "app.h"
#include "opsis-qt/src/bridge.rs.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr char kNodeMime[] = "application/x-opsis-node";
constexpr int kTypeRole = Qt::UserRole + 1;
constexpr int kBadge = 20;

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
  return QIcon(pixmap);
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

QStringList NodeTree::mimeTypes() const {
  return {QLatin1String(kNodeMime)};
}

QMimeData* NodeTree::mimeData(const QList<QTreeWidgetItem*>& items) const {
  auto* data = new QMimeData();
  if (!items.isEmpty()) {
    data->setData(QLatin1String(kNodeMime),
                  items.first()->data(0, kTypeRole).toString().toUtf8());
  }
  return data;
}

NodeList::NodeList(QWidget* parent) : QWidget(parent) {
  setObjectName(QStringLiteral("Panel"));
  setAttribute(Qt::WA_StyledBackground);
  setMinimumWidth(220);
  auto* layout = new QVBoxLayout(this);
  layout->setContentsMargins(8, 8, 8, 8);
  layout->setSpacing(8);
  search_ = new QLineEdit(this);
  search_->setPlaceholderText(QStringLiteral("Search nodes"));
  search_->setProperty("role", "search");
  search_->addAction(App::Get().Icon(QStringLiteral("search"),
                                     App::Get().Token("muted-foreground")),
                     QLineEdit::LeadingPosition);
  search_->setClearButtonEnabled(true);
  layout->addWidget(search_);
  tree_ = new NodeTree(this);
  layout->addWidget(tree_, 1);

  const auto palette = App::Get().session().palette();
  QTreeWidgetItem* section = nullptr;
  QString current;
  QFont heading = font();
  heading.setPixelSize(11);
  heading.setWeight(QFont::DemiBold);
  for (const auto& entry : palette) {
    const QString category = ToQ(entry.category);
    if (category != current) {
      current = category;
      section = new QTreeWidgetItem(tree_, {category.toUpper()});
      section->setFlags(Qt::ItemIsEnabled);
      section->setFont(0, heading);
      section->setForeground(0, App::Get().Token("muted-foreground"));
      section->setExpanded(true);
    }
    auto* item = new QTreeWidgetItem(section, {ToQ(entry.title)});
    item->setIcon(0, BadgeIcon(ToQ(entry.glyph), QColor::fromString(ToQ(entry.color))));
    item->setData(0, kTypeRole, ToQ(entry.node_type));
    item->setToolTip(0, ToQ(entry.description));
    item->setFlags(Qt::ItemIsEnabled | Qt::ItemIsSelectable |
                   Qt::ItemIsDragEnabled);
  }
  tree_->expandAll();
  connect(search_, &QLineEdit::textChanged, this, &NodeList::Filter);
  connect(tree_, &QTreeWidget::itemDoubleClicked, this,
          [this](QTreeWidgetItem* item) {
            const QString type = item->data(0, kTypeRole).toString();
            if (!type.isEmpty()) emit AddRequested(type);
          });
}

void NodeList::Filter(const QString& text) {
  const QString needle = text.trimmed();
  for (int i = 0; i < tree_->topLevelItemCount(); ++i) {
    QTreeWidgetItem* section = tree_->topLevelItem(i);
    int shown = 0;
    for (int j = 0; j < section->childCount(); ++j) {
      QTreeWidgetItem* item = section->child(j);
      const bool match = needle.isEmpty() ||
                         item->text(0).contains(needle, Qt::CaseInsensitive);
      item->setHidden(!match);
      shown += match ? 1 : 0;
    }
    section->setHidden(shown == 0);
  }
}

}
