#ifndef OPSIS_QT_CPP_TIMELINE_PANEL_H_
#define OPSIS_QT_CPP_TIMELINE_PANEL_H_

#include <QButtonGroup>
#include <QLabel>
#include <QPushButton>
#include <QStackedWidget>
#include <QToolButton>
#include <QWidget>

namespace opsis {

class Combo;
class TimelineCanvas;

class TimelinePanel : public QWidget {
  Q_OBJECT

 public:
  explicit TimelinePanel(QWidget* parent = nullptr);
  TimelineCanvas* canvas() const { return canvas_; }

 private:
  void Reload();
  void SyncBar();

  QStackedWidget* stack_;
  TimelineCanvas* canvas_;
  QToolButton* play_;
  QPushButton* clock_;
  QButtonGroup tools_;
  QToolButton* snap_;
  Combo* video_keys_;
  Combo* audio_keys_;
  QWidget* range_;
  QLabel* range_text_;
  QLabel* source_;
};

}

#endif
