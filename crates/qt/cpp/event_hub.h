#ifndef OPSIS_QT_SPIKE_CPP_EVENT_HUB_H_
#define OPSIS_QT_SPIKE_CPP_EVENT_HUB_H_

#include <cstdint>

#include <QObject>

namespace opsis {

class EventHub : public QObject {
  Q_OBJECT

 public:
  static EventHub* Instance();
  void Dispatch(std::uint8_t kind);

 signals:
  void DocumentChanged();
  void SelectionChanged();
  void SettingsChanged();
  void ProjectChanged();
  void Saved();

 private:
  explicit EventHub(QObject* parent);
};

}

#endif
