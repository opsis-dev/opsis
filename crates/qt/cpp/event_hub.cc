#include "event_hub.h"

#include <QCoreApplication>
#include <algorithm>

#include "app.h"
#include "opsis-qt/src/bridge.rs.h"

namespace opsis {

EventHub* EventHub::Instance() {
  static auto* hub = new EventHub(QCoreApplication::instance());
  return hub;
}

EventHub::EventHub(QObject* parent) : QObject(parent) {}

void EventHub::Post(std::uint8_t kind, const std::vector<QString>& edited,
                    bool all) {
  if (kind >= kKinds) return;
  const std::scoped_lock lock(mutex_);
  if (std::find(order_.begin(), order_.end(), kind) == order_.end()) {
    order_.push_back(kind);
  }
  all_.at(kind) = all_.at(kind) || all || edited.empty();
  for (const QString& id : edited) ids_.at(kind).insert(id);
  if (scheduled_) return;
  scheduled_ = true;
  QMetaObject::invokeMethod(this, [this] { Flush(); }, Qt::QueuedConnection);
}

void EventHub::PostReplay(const QString& node, const QString& tab) {
  QMetaObject::invokeMethod(
      this, [this, node, tab] { emit ReplayAsked(node, tab); },
      Qt::QueuedConnection);
}

void EventHub::Flush() {
  std::vector<std::uint8_t> order;
  std::array<bool, kKinds> all{};
  std::array<QSet<QString>, kKinds> ids;
  {
    const std::scoped_lock lock(mutex_);
    order.swap(order_);
    all.swap(all_);
    ids.swap(ids_);
    scheduled_ = false;
  }
  for (std::uint8_t kind : order) {
    edited_all_ = all.at(kind);
    edited_ = ids.at(kind);
    Dispatch(kind);
  }
  edited_all_ = true;
  edited_.clear();
}

void EventHub::Dispatch(std::uint8_t kind) {
  if (kind < kKinds) counts_.at(kind) += 1;
  switch (static_cast<bridge::EventKind>(kind)) {
    case bridge::EventKind::Document:
      emit DocumentChanged();
      break;
    case bridge::EventKind::Selection:
      emit SelectionChanged();
      break;
    case bridge::EventKind::Settings:
      emit SettingsChanged();
      break;
    case bridge::EventKind::Project:
      emit ProjectChanged();
      break;
    case bridge::EventKind::Saved:
      emit Saved();
      break;
    case bridge::EventKind::CloseAsked:
      emit CloseAsked();
      break;
    case bridge::EventKind::FlushDrafts:
      emit FlushDrafts();
      break;
    case bridge::EventKind::Compiled:
      emit CompiledChanged();
      break;
    case bridge::EventKind::Run:
      emit RunChanged();
      break;
    case bridge::EventKind::Api:
      emit ApiChanged();
      break;
    case bridge::EventKind::Media:
      emit MediaChanged();
      break;
    case bridge::EventKind::Engine:
      emit EngineChanged();
      break;
    default:
      break;
  }
}

}

namespace opsis::bridge {

void post_event(EventKind kind) {
  EventHub::Instance()->Post(static_cast<std::uint8_t>(kind), {}, true);
}

void post_edited(rust::Vec<rust::String> ids) {
  std::vector<QString> edited;
  edited.reserve(ids.size());
  for (const auto& id : ids) edited.push_back(opsis::ToQ(id));
  EventHub::Instance()->Post(static_cast<std::uint8_t>(EventKind::Document),
                             edited, false);
}

void post_replay(rust::Str node_id, rust::Str tab) {
  EventHub::Instance()->PostReplay(
      QString::fromUtf8(node_id.data(), static_cast<qsizetype>(node_id.size())),
      QString::fromUtf8(tab.data(), static_cast<qsizetype>(tab.size())));
}

}
