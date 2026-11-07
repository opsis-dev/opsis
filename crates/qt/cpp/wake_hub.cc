#include "wake_hub.h"

#include <QCoreApplication>
#include <algorithm>
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
  listeners_[target].push_back(Listener{owner, std::move(on_wake)});
}

void WakeHub::Forget(std::uint64_t target, QObject* owner) {
  const auto found = listeners_.find(target);
  if (found == listeners_.end()) return;
  auto& listeners = found->second;
  listeners.erase(std::remove_if(listeners.begin(), listeners.end(),
                                 [owner](const Listener& listener) {
                                   return listener.owner.isNull() ||
                                          listener.owner.data() == owner;
                                 }),
                  listeners.end());
  if (found->second.empty()) listeners_.erase(found);
}

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
    auto& listeners = found->second;
    listeners.erase(std::remove_if(listeners.begin(), listeners.end(),
                                   [](const Listener& listener) {
                                     return listener.owner.isNull();
                                   }),
                    listeners.end());
    for (const Listener& listener : found->second) {
      calls.push_back(listener.on_wake);
    }
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
