#ifndef OPSIS_QT_SPIKE_CPP_BRIDGE_FFI_H_
#define OPSIS_QT_SPIKE_CPP_BRIDGE_FFI_H_

#include <cstdint>

namespace opsis::bridge {

enum class EventKind : std::uint8_t;

std::int32_t run_app();
void post_event(EventKind kind);

}

#endif
