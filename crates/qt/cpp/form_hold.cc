#include "form_hold.h"

#include "app.h"
#include "opsis-qt/src/bridge.rs.h"

namespace opsis {

FormHold* FormHold::Instance() {
  static auto* hold = new FormHold();
  return hold;
}

void FormHold::Begin() {
  if (depth_++ == 0) App::Get().session().begin_gesture();
}

void FormHold::End() {
  if (depth_ == 0) return;
  if (--depth_ == 0) {
    App::Get().session().end_gesture();
    emit Released();
  }
}

}
