#ifndef OPSIS_QT_CPP_MIXER_PANEL_H_
#define OPSIS_QT_CPP_MIXER_PANEL_H_

#include <QJsonObject>
#include <QScrollArea>
#include <QSet>
#include <QStackedWidget>
#include <QString>
#include <QWidget>
#include <optional>

namespace opsis {

namespace bridge {
struct MixerStripView;
struct MixerView;
}

class MixerPanel : public QWidget {
  Q_OBJECT

 public:
  explicit MixerPanel(QWidget* parent = nullptr);
  void Rebuild();

 private:
  QWidget* ChannelStrip(const bridge::MixerStripView& strip,
                        const bridge::MixerView& view, QWidget* parent);
  QWidget* BusStrip(const bridge::MixerStripView& strip,
                    const bridge::MixerView& view, QWidget* parent);
  QWidget* MasterStrip(const bridge::MixerView& view, QWidget* parent);
  QWidget* Toolbar(const bridge::MixerView& view, QWidget* parent);
  void Edit(const QJsonObject& edit);
  void SelectStrip(const QString& id, const QStringList& order,
                   Qt::KeyboardModifiers modifiers, const QString& source);

  QStackedWidget* stack_;
  QWidget* body_ = nullptr;
  QJsonObject book_;
  QSet<QString> folded_;
  std::optional<QJsonObject> clipboard_;
  QString anchor_;
};

}

#endif
