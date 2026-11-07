#ifndef OPSIS_QT_CPP_WAKE_HUB_H_
#define OPSIS_QT_CPP_WAKE_HUB_H_

#include <QObject>
#include <QPointer>
#include <cstdint>
#include <functional>
#include <mutex>
#include <unordered_map>
#include <unordered_set>
#include <vector>

namespace opsis {

class WakeHub {
 public:
  static WakeHub& Get();
  void Listen(std::uint64_t target, QObject* owner,
              std::function<void()> on_wake);
  void Forget(std::uint64_t target, QObject* owner);
  void Wake(std::uint64_t target);
  int delivered() const { return delivered_; }

 private:
  struct Listener {
    QPointer<QObject> owner;
    std::function<void()> on_wake;
  };

  void Deliver();

  std::mutex mutex_;
  std::unordered_set<std::uint64_t> pending_;
  bool scheduled_ = false;
  std::unordered_map<std::uint64_t, std::vector<Listener>> listeners_;
  int delivered_ = 0;
};

}

#endif
