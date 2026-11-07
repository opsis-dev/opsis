#ifndef OPSIS_QT_CPP_BRIDGE_FFI_H_
#define OPSIS_QT_CPP_BRIDGE_FFI_H_

#include <cstdint>

#include "rust/cxx.h"

namespace opsis::bridge {

enum class EventKind : std::uint8_t;
enum class DialogKind : std::uint8_t;
struct Session;

std::int32_t run_app(rust::Box<Session> session);
void post_event(EventKind kind);
void wake(std::uint64_t target);
void post_edited(rust::Vec<rust::String> ids);
void post_replay(rust::Str node_id, rust::Str tab);
void open_view_window(rust::Str id, rust::Str title);
void close_view_window(rust::Str id);
void request_dialog(DialogKind kind, std::uint64_t id, rust::Str title,
                    rust::Str start, rust::Str filter_name,
                    rust::Str filter_extensions);
void open_path(rust::Str path);
void reveal_path(rust::Str path);
void notify(rust::Str title, rust::Str body);
void raise_editor();
void quit_app(std::int32_t code);
rust::String qt_version();

}

#endif
