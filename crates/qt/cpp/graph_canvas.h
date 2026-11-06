#ifndef OPSIS_QT_CPP_GRAPH_CANVAS_H_
#define OPSIS_QT_CPP_GRAPH_CANVAS_H_

#include <QGraphicsItem>
#include <QGraphicsPathItem>
#include <QGraphicsScene>
#include <QGraphicsView>
#include <QHash>
#include <QLabel>
#include <QPointer>
#include <QTimer>
#include <optional>

#include "opsis-qt/src/bridge.rs.h"

namespace opsis {

class GraphCanvas;
class Minimap;

struct PortHit {
  QString node;
  QString port;
  bool output = false;
  QPointF at;
};

class CardItem : public QGraphicsItem {
 public:
  CardItem(GraphCanvas* canvas, const bridge::CardView& card);

  QRectF boundingRect() const override;
  QPainterPath shape() const override;
  void paint(QPainter* painter, const QStyleOptionGraphicsItem* option,
             QWidget* widget) override;

  void SetCard(const bridge::CardView& card);
  const bridge::CardView& card() const { return card_; }
  QString id() const { return id_; }
  std::optional<PortHit> PortAt(const QPointF& local) const;
  QPointF PortPoint(const QString& port, bool output) const;

 protected:
  QVariant itemChange(GraphicsItemChange change,
                      const QVariant& value) override;
  void mousePressEvent(QGraphicsSceneMouseEvent* event) override;
  void mouseMoveEvent(QGraphicsSceneMouseEvent* event) override;
  void mouseReleaseEvent(QGraphicsSceneMouseEvent* event) override;

 private:
  void PaintCard(QPainter* painter, bool detailed);
  void PaintRing(QPainter* painter, const QRectF& body);
  double PaintBadge(QPainter* painter);
  void PaintGroup(QPainter* painter);
  void PaintNote(QPainter* painter);

  GraphCanvas* canvas_;
  bridge::CardView card_;
  QString id_;
  bool connecting_ = false;
};

class EdgeItem : public QGraphicsPathItem {
 public:
  explicit EdgeItem(const bridge::EdgeView& edge);
  void SetEdge(const bridge::EdgeView& edge);
  const bridge::EdgeView& edge() const { return edge_; }
  QString id() const { return id_; }
  QString source() const { return source_; }
  QString target() const { return target_; }
  void Route(const QPointF& from, const QPointF& to);
  QPainterPath shape() const override;
  void paint(QPainter* painter, const QStyleOptionGraphicsItem* option,
             QWidget* widget) override;

 private:
  bridge::EdgeView edge_;
  QString id_;
  QString source_;
  QString target_;
};

class GraphCanvas : public QGraphicsView {
  Q_OBJECT

 public:
  explicit GraphCanvas(QWidget* parent = nullptr);

  void Sync();
  void FitAll();
  void SelectAll();
  void ZoomBy(double factor, const QPointF& anchor);
  double zoom() const;
  double spin_angle() const { return spin_angle_; }
  void SetMinimapVisible(bool visible);
  void Reveal(const QString& id);
  Q_INVOKABLE void copy();
  Q_INVOKABLE void paste();
  Q_INVOKABLE void selectAll() { SelectAll(); }
  void Duplicate();
  bool minimap_visible() const;
  void CardMoved(const QString& id);
  void BeginConnect(const PortHit& from);
  void UpdateConnect(const QPointF& scene_point);
  void FinishConnect(const QPointF& scene_point);
  void CommitMoves();
  void Toast(const QString& message, bool error);
  QString Measure(int frames);
  QPoint ViewPointOf(const QString& node) const;
  QPoint ViewPointOfPort(const QString& node, const QString& port,
                         bool output) const;
  int card_count() const { return static_cast<int>(cards_.size()); }

 signals:
  void ZoomChanged(double zoom);

 protected:
  void drawBackground(QPainter* painter, const QRectF& rect) override;
  void wheelEvent(QWheelEvent* event) override;
  void mousePressEvent(QMouseEvent* event) override;
  void mouseMoveEvent(QMouseEvent* event) override;
  void mouseReleaseEvent(QMouseEvent* event) override;
  void keyPressEvent(QKeyEvent* event) override;
  void resizeEvent(QResizeEvent* event) override;
  void dragEnterEvent(QDragEnterEvent* event) override;
  void dragMoveEvent(QDragMoveEvent* event) override;
  void dropEvent(QDropEvent* event) override;
  bool viewportEvent(QEvent* event) override;
  void scrollContentsBy(int dx, int dy) override;

 private:
  void SyncSelection();
  void RemoveSelected();
  void RouteEdge(EdgeItem* edge);
  std::optional<PortHit> PortAtScene(const QPointF& point) const;
  void PlaceOverlays();
  bool TouchpadMode() const;

  QGraphicsScene scene_;
  QHash<QString, CardItem*> cards_;
  QHash<QString, EdgeItem*> edges_;
  QGraphicsPathItem* pending_ = nullptr;
  std::optional<PortHit> from_;
  QWidget* toolbar_ = nullptr;
  QLabel* zoom_label_ = nullptr;
  Minimap* minimap_ = nullptr;
  QLabel* toast_ = nullptr;
  QTimer toast_timer_;
  QTimer sync_timer_;
  QTimer spin_timer_;
  double spin_angle_ = 0;
  QPoint pan_from_;
  bool panning_ = false;
  bool syncing_ = false;
  QString canvas_mode_;
};

class Minimap : public QGraphicsView {
 public:
  Minimap(GraphCanvas* canvas, QGraphicsScene* scene);
  void Refit();

 protected:
  void drawForeground(QPainter* painter, const QRectF& rect) override;
  void mousePressEvent(QMouseEvent* event) override;
  void mouseMoveEvent(QMouseEvent* event) override;

 private:
  GraphCanvas* canvas_;
};

}

#endif
