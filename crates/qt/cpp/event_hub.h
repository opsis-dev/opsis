#ifndef OPSIS_QT_CPP_EVENT_HUB_H_
#define OPSIS_QT_CPP_EVENT_HUB_H_

#include <QObject>
#include <cstdint>

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
  void CloseAsked();
  void FlushDrafts();
  void CompiledChanged();
  void RunChanged();
  void ApiChanged();
  void MediaChanged();
  void ReplayAsked();
  void Notice(const QString& title, const QString& body);

 private:
  explicit EventHub(QObject* parent);
};

}

#endif
