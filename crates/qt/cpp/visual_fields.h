#ifndef OPSIS_QT_CPP_VISUAL_FIELDS_H_
#define OPSIS_QT_CPP_VISUAL_FIELDS_H_

#include <QString>
#include <QStringList>
#include <QWidget>
#include <functional>

namespace opsis {

namespace bridge {
struct FieldView;
}

using CommitValue = std::function<void(const QString& json)>;

QWidget* MakeThemeField(const bridge::FieldView& field,
                        const QStringList& swatches, bool open,
                        const std::function<void(bool)>& on_open,
                        const CommitValue& commit);

}

#endif
