#ifndef OPSIS_QT_CPP_LOOK_GLYPH_H_
#define OPSIS_QT_CPP_LOOK_GLYPH_H_

#include <QColor>
#include <QElapsedTimer>
#include <QHash>
#include <QObject>
#include <QRectF>
#include <QString>
#include <QTimer>

class QPainter;

namespace opsis {

struct Look {
  QString glyph;
  QString motion;
  QString morph;
};

struct LookFrame {
  double motion_ms = -1;
  double morph_at = 0;
  bool morphing = false;
};

class LookPlayer : public QObject {
  Q_OBJECT

 public:
  explicit LookPlayer(QObject* parent = nullptr);
  void Enter(const QString& id, const Look& look);
  void Leave(const QString& id);
  LookFrame FrameOf(const QString& id) const;
  double MotionElapsed(const QString& id) const;
  double MorphAt(const QString& id) const;

 signals:
  void Changed(const QString& id);

 private:
  struct Spring {
    double position = 0;
    double velocity = 0;
    double target = 0;
    bool settled = true;
  };

  void Tick();
  void Run();

  QTimer timer_;
  QElapsedTimer clock_;
  qint64 last_tick_ = 0;
  QHash<QString, qint64> started_;
  QHash<QString, qint64> lengths_;
  QHash<QString, Spring> springs_;
};

void PaintLookGlyph(QPainter* painter, const QRectF& badge, const QRectF& box,
                    const Look& look, const LookFrame& frame,
                    const QColor& accent);

}

#endif
