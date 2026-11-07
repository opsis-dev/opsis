#include "wake_hub.h"

#include <QCoreApplication>
#include <utility>
#include <vector>

#include "opsis-qt/src/bridge.rs.h"

namespace opsis {

WakeHub& WakeHub::Get() {
  static WakeHub hub;
  return hub;
}

void WakeHub::Listen(std::uint64_t target, QObject* owner,
                     std::function<void()> on_wake) {
  listeners_[target] = Listener{owner, std::move(on_wake)};
}

void WakeHub::Forget(std::uint64_t target) { listeners_.erase(target); }

void WakeHub::Wake(std::uint64_t target) {
  QCoreApplication* app = QCoreApplication::instance();
  if (app == nullptr) return;
  const std::scoped_lock lock(mutex_);
  pending_.insert(target);
  if (scheduled_) return;
  scheduled_ = true;
  QMetaObject::invokeMethod(app, [this] { Deliver(); }, Qt::QueuedConnection);
}

void WakeHub::Deliver() {
  std::unordered_set<std::uint64_t> woken;
  {
    const std::scoped_lock lock(mutex_);
    woken.swap(pending_);
    scheduled_ = false;
  }
  std::vector<std::function<void()>> calls;
  for (std::uint64_t target : woken) {
    const auto found = listeners_.find(target);
    if (found == listeners_.end()) continue;
    if (found->second.owner.isNull()) {
      listeners_.erase(found);
      continue;
    }
    calls.push_back(found->second.on_wake);
  }
  for (const auto& call : calls) {
    delivered_ += 1;
    call();
  }
}

}

namespace opsis::bridge {

void wake(std::uint64_t target) { WakeHub::Get().Wake(target); }

}
