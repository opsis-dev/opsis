#ifndef OPSIS_QT_CPP_NODE_LIST_H_
#define OPSIS_QT_CPP_NODE_LIST_H_

#include <QFrame>
#include <QLabel>
#include <QLineEdit>
#include <QPointer>
#include <QTimer>
#include <QTreeWidget>
#include <QWidget>
#include <vector>

#include "opsis-qt/src/bridge.rs.h"

namespace opsis {

class NodeTree : public QTreeWidget {
 public:
  explicit NodeTree(QWidget* parent = nullptr);
  void UpdateSticky();

 protected:
  QMimeData* mimeData(const QList<QTreeWidgetItem*>& items) const override;
  QStringList mimeTypes() const override;
  void resizeEvent(QResizeEvent* event) override;
  void leaveEvent(QEvent* event) override;

 private:
  QLabel* sticky_;
};

class NodeList : public QWidget {
  Q_OBJECT

 public:
  explicit NodeList(const QString& category = QString(),
                    const QString& label = QString(),
                    QWidget* parent = nullptr);
  void SetPlaceholder(const QString& text);
  QString Problems() const;

 signals:
  void AddRequested(const QString& node_type);

 private:
  void Filter(const QString& text);

  void Hover(QTreeWidgetItem* item);
  void ShowCard();

  QLineEdit* search_;
  NodeTree* tree_;
  QLabel* empty_;
  std::vector<bridge::PaletteEntry> entries_;
  QTimer hover_timer_;
  QTreeWidgetItem* hovered_ = nullptr;
  QPointer<QFrame> card_;
};

class CategoryRail : public QWidget {
  Q_OBJECT

 public:
  static constexpr int kWidth = 44;

  explicit CategoryRail(QWidget* parent = nullptr);

 signals:
  void AddRequested(const QString& node_type);

 private:
  void OpenCategory(QWidget* anchor, const QString& id, const QString& label);
};

}

#endif
