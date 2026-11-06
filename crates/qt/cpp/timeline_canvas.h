#ifndef OPSIS_QT_CPP_TIMELINE_CANVAS_H_
#define OPSIS_QT_CPP_TIMELINE_CANVAS_H_

#include <QAbstractScrollArea>
#include <QHash>
#include <QJsonObject>
#include <QLineEdit>
#include <QPixmap>
#include <QPointer>
#include <QSet>
#include <QString>
#include <memory>
#include <optional>
#include <vector>

#include "opsis-qt/src/bridge.rs.h"

namespace opsis {

class TimelineCanvas : public QAbstractScrollArea {
  Q_OBJECT

 public:
  explicit TimelineCanvas(QWidget* parent = nullptr);

  void Reload();
  bool has_film() const { return view_ != nullptr && view_->has_film; }
  const bridge::TimelineView* view() const { return view_.get(); }
  double px_per_frame() const { return ppf_; }
  QString tool() const { return tool_; }
  bool snapping() const { return snapping_; }
  bool timecode() const { return timecode_; }
  QString selected_marker() const { return selected_marker_; }

  void SetTool(const QString& tool);
  void SetSnapping(bool on);
  void SetTimecode(bool on);
  void ZoomBy(double factor);
  void Fit();
  void SplitAtPlayhead();
  void AddMarker();
  void RemoveSelection();
  QString TimeText(double frame) const;
  double FrameX(double frame) const;
  QRectF BarRect(const QString& key) const;
  QRectF MarkerRect(const QString& id) const;
  QPointF CameraKeyPoint(const QString& key) const;
  int RulerBottom() const;

 signals:
  void StateChanged();

 protected:
  void paintEvent(QPaintEvent* event) override;
  void resizeEvent(QResizeEvent* event) override;
  void mousePressEvent(QMouseEvent* event) override;
  void mouseMoveEvent(QMouseEvent* event) override;
  void mouseReleaseEvent(QMouseEvent* event) override;
  void mouseDoubleClickEvent(QMouseEvent* event) override;
  void wheelEvent(QWheelEvent* event) override;
  void keyPressEvent(QKeyEvent* event) override;
  void scrollContentsBy(int dx, int dy) override;

 private:
  enum class Press {
    kNone,
    kScrub,
    kEdit,
    kMarker,
    kCameraKey,
    kMarquee,
  };

  struct Drag {
    Press press = Press::kNone;
    QPointF start;
    QPointF last;
    bool started = false;
    QString key;
    QString edit;
    bool ripple = false;
    bool audio = false;
    double from = 0;
    double to = 0;
    double origin = 0;
    QString node;
    bool shift = false;
    QString error;
  };

  void PaintRuler(QPainter* painter);
  void PaintMarkers(QPainter* painter);
  void PaintRows(QPainter* painter);
  void PaintBar(QPainter* painter, const bridge::TimelineBarView& bar);
  void PaintAudio(QPainter* painter, const bridge::TimelineAudioView& bar);
  void PaintHeader(QPainter* painter);
  void UpdateScroll();
  double ContentWidth() const;
  int VideoRowTop(std::uint32_t lane) const;
  int AudioRowTop(std::uint32_t lane) const;
  double FrameAt(double x) const;
  std::vector<double> SnapTargets(const QString& exclude, bool markers) const;
  double SnapThreshold(bool alt) const;
  const bridge::TimelineBarView* VideoAt(const QPointF& point) const;
  const bridge::TimelineAudioView* AudioAt(const QPointF& point) const;
  const bridge::TimelineMarkerView* MarkerAt(const QPointF& point) const;
  const bridge::TimelineCameraKeyView* CameraKeyAt(const QPointF& point) const;
  QString EditKind(const bridge::TimelineBarView& bar, const QPointF& point,
                   bool* ripple) const;
  void Select(const QStringList& ids, bool additive);
  void PressBar(QMouseEvent* event);
  void MoveEdit(QMouseEvent* event);
  void FinishMarquee(QMouseEvent* event);
  void SendEdit(const QJsonObject& edit, QString* error);
  void ShowError(const QString& error);
  bool ConfirmRemoval(std::size_t count);
  void StartRename(const QString& id);
  void ZoomAround(double factor, double x);
  void FollowPlayhead();
  void PaintFilmstrip(QPainter* painter, const QString& key, const QRectF& box);
  void PaintWaveform(QPainter* painter, const QString& key, const QRectF& box);
  const QPixmap& Picture(const QString& path);

  std::unique_ptr<bridge::TimelineView> view_;
  double ppf_ = 2;
  bool fit_ = true;
  QString tool_ = QStringLiteral("select");
  bool snapping_ = true;
  bool timecode_ = false;
  QString selected_marker_;
  Drag drag_;
  std::optional<double> guide_;
  std::optional<QRectF> marquee_;
  QPointer<QLineEdit> rename_;
  QHash<QString, QPixmap> pictures_;
};

}

#endif
