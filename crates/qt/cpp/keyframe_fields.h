#ifndef OPSIS_QT_CPP_KEYFRAME_FIELDS_H_
#define OPSIS_QT_CPP_KEYFRAME_FIELDS_H_

#include <QJsonObject>
#include <QWidget>
#include <functional>

namespace opsis {

namespace bridge {
struct FieldView;
}

using KeyframeEdit = std::function<void(const QJsonObject& edit)>;

QWidget* MakeEnvelopeField(const bridge::FieldView& field,
                           const KeyframeEdit& edit);
QWidget* MakeCameraField(const bridge::FieldView& field,
                         const KeyframeEdit& edit);

}

#endif
