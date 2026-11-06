#ifndef OPSIS_QT_SPIKE_CPP_NODE_LIST_H_
#define OPSIS_QT_SPIKE_CPP_NODE_LIST_H_

#include <QLineEdit>
#include <QTreeWidget>
#include <QWidget>

namespace opsis {

class NodeTree : public QTreeWidget {
 public:
  explicit NodeTree(QWidget* parent = nullptr);

 protected:
  QMimeData* mimeData(const QList<QTreeWidgetItem*>& items) const override;
  QStringList mimeTypes() const override;
};

class NodeList : public QWidget {
  Q_OBJECT

 public:
  explicit NodeList(QWidget* parent = nullptr);

 signals:
  void AddRequested(const QString& node_type);

 private:
  void Filter(const QString& text);

  QLineEdit* search_;
  NodeTree* tree_;
};

}

#endif
