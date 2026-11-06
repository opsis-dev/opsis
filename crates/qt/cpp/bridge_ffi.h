#ifndef OPSIS_QT_SPIKE_CPP_BRIDGE_FFI_H_
#define OPSIS_QT_SPIKE_CPP_BRIDGE_FFI_H_

#include <cstdint>

#include "rust/cxx.h"

namespace opsis::bridge {

enum class EventKind : std::uint8_t;
struct Session;

std::int32_t run_app(rust::Box<Session> session);
void post_event(EventKind kind);

}

#endif
