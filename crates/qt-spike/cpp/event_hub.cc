#include "event_hub.h"

#include <QCoreApplication>

#include "opsis-qt-spike/src/bridge.rs.h"

namespace opsis {

EventHub* EventHub::Instance() {
  static EventHub* hub = new EventHub(QCoreApplication::instance());
  return hub;
}

EventHub::EventHub(QObject* parent) : QObject(parent) {}

void EventHub::Dispatch(std::uint8_t kind) {
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
    default:
      break;
  }
}

}

namespace opsis::bridge {

void post_event(EventKind kind) {
  EventHub* hub = EventHub::Instance();
  const auto raw = static_cast<std::uint8_t>(kind);
  QMetaObject::invokeMethod(
      hub, [hub, raw] { hub->Dispatch(raw); }, Qt::QueuedConnection);
}

}
