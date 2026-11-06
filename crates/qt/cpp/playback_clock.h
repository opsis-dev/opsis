#ifndef OPSIS_QT_CPP_PLAYBACK_CLOCK_H_
#define OPSIS_QT_CPP_PLAYBACK_CLOCK_H_

#include <QElapsedTimer>
#include <QObject>
#include <QTimer>

namespace opsis {

class PlaybackClock : public QObject {
  Q_OBJECT

 public:
  static PlaybackClock* Instance();

  double frame() const { return frame_; }
  bool playing() const { return playing_; }
  double rate() const { return rate_; }
  double in_point() const { return in_point_; }
  double out_point() const { return out_point_; }
  double duration() const { return duration_; }
  int fps() const { return fps_; }

  void SetFilm(int fps, double duration);
  void Seek(double frame);
  void Play();
  void Pause();
  void Toggle();
  void Step(double frames);
  void Shuttle(int direction);
  void SetIn(double frame);
  void SetOut(double frame);
  void ClearRange();

 signals:
  void Changed();

 private:
  PlaybackClock();
  void Tick();
  void Share();

  QTimer timer_;
  QElapsedTimer elapsed_;
  double position_ = 0;
  double frame_ = 0;
  bool playing_ = false;
  double rate_ = 1;
  double in_point_ = -1;
  double out_point_ = -1;
  double duration_ = 1;
  int fps_ = 30;
};

}

#endif
