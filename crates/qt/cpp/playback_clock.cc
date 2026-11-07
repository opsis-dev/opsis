#include "playback_clock.h"

#include <algorithm>
#include <cmath>

#include "app.h"
#include "opsis-qt/src/bridge.rs.h"

namespace opsis {

namespace {

constexpr int kTickMs = 8;
constexpr double kMaxShuttleRate = 4;
constexpr double kShuttleStep = 2;
constexpr double kMsPerSecond = 1000.0;

double Clamped(double frame, double duration) {
  return std::clamp(std::round(frame), 0.0, std::max(0.0, duration - 1));
}

}

PlaybackClock* PlaybackClock::Instance() {
  static auto* clock = new PlaybackClock();
  return clock;
}

PlaybackClock::PlaybackClock() {
  timer_.setTimerType(Qt::PreciseTimer);
  timer_.setInterval(kTickMs);
  connect(&timer_, &QTimer::timeout, this, &PlaybackClock::Tick);
}

void PlaybackClock::SetFilm(int fps, double duration) {
  if (fps == fps_ && duration == duration_) return;
  fps_ = std::max(1, fps);
  duration_ = std::max(1.0, duration);
  if (frame_ >= duration_) Seek(duration_ - 1);
  emit Changed();
}

void PlaybackClock::Seek(double frame) {
  frame_ = Clamped(frame, duration_);
  position_ = frame_;
  App::Get().session().audio_seek(frame_ / fps_);
  Share();
  emit Changed();
}

void PlaybackClock::Play() {
  if (playing_) return;
  if (rate_ != 1) rate_ = 1;
  position_ = frame_;
  playing_ = true;
  elapsed_.start();
  timer_.start();
  App::Get().session().audio_play(frame_ / fps_, rate_);
  Share();
  emit Changed();
}

void PlaybackClock::Pause() {
  if (!playing_) return;
  timer_.stop();
  playing_ = false;
  App::Get().session().audio_pause();
  Share();
  emit Changed();
}

void PlaybackClock::Toggle() {
  if (playing_) {
    Pause();
  } else {
    Play();
  }
}

void PlaybackClock::Step(double frames) {
  Pause();
  Seek(frame_ + frames);
}

void PlaybackClock::Shuttle(int direction) {
  if (direction == 0) {
    Pause();
    rate_ = 1;
    Share();
    emit Changed();
    return;
  }
  const bool same = playing_ && (rate_ > 0) == (direction > 0);
  const double next =
      same ? std::clamp(rate_ * kShuttleStep, -kMaxShuttleRate, kMaxShuttleRate)
           : static_cast<double>(direction);
  if (!playing_) {
    position_ = frame_;
    playing_ = true;
    elapsed_.start();
    timer_.start();
  }
  rate_ = next;
  App::Get().session().audio_play(position_ / fps_, rate_);
  Share();
  emit Changed();
}

void PlaybackClock::SetIn(double frame) {
  in_point_ = frame;
  if (in_point_ >= 0 && out_point_ >= 0 && out_point_ <= in_point_) {
    out_point_ = -1;
  }
  Share();
  emit Changed();
}

void PlaybackClock::SetOut(double frame) {
  out_point_ = frame;
  if (out_point_ >= 0 && in_point_ >= 0 && in_point_ >= out_point_) {
    in_point_ = -1;
  }
  Share();
  emit Changed();
}

void PlaybackClock::ClearRange() {
  in_point_ = -1;
  out_point_ = -1;
  Share();
  emit Changed();
}

void PlaybackClock::Tick() {
  const double seconds = static_cast<double>(elapsed_.restart()) / kMsPerSecond;
  auto& session = App::Get().session();
  const double heard = session.audio_clock();
  const double moved =
      heard >= 0 ? heard * fps_ : position_ + seconds * fps_ * rate_;
  position_ = session.wrap_playhead(moved, in_point_, out_point_, duration_);
  if (std::abs(position_ - moved) > 1) session.audio_seek(position_ / fps_);
  const double frame = std::floor(position_);
  if (frame == frame_) return;
  frame_ = frame;
  Share();
  emit Changed();
}

void PlaybackClock::Share() {
  App::Get().session().share_playhead(frame_, playing_, rate_, in_point_,
                                      out_point_);
}

}
