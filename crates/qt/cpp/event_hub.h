#ifndef OPSIS_QT_CPP_EVENT_HUB_H_
#define OPSIS_QT_CPP_EVENT_HUB_H_

#include <QObject>
#include <QSet>
#include <QString>
#include <array>
#include <cstdint>
#include <mutex>
#include <vector>

namespace opsis {

class EventHub : public QObject {
  Q_OBJECT

 public:
  static EventHub* Instance();
  void Post(std::uint8_t kind, const std::vector<QString>& edited, bool all);
  void PostReplay(const QString& node, const QString& tab);
  void Dispatch(std::uint8_t kind);
  bool Edited(const QString& node) const {
    return edited_all_ || edited_.contains(node);
  }
  int dispatched(std::uint8_t kind) const {
    return kind < counts_.size() ? counts_.at(kind) : 0;
  }

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
  void EngineChanged();
  void FontsChanged();
  void UpdateChanged();
  void ReplayAsked(const QString& node, const QString& tab);
  void Notice(const QString& title, const QString& body);
  void SettingsAsked(const QString& page);
  void DiskChangeAsked(bool readable);

 private:
  static constexpr std::size_t kKinds = 16;

  explicit EventHub(QObject* parent);
  void Flush();

  std::mutex mutex_;
  bool scheduled_ = false;
  std::vector<std::uint8_t> order_;
  std::array<bool, kKinds> all_{};
  std::array<QSet<QString>, kKinds> ids_;
  QSet<QString> edited_;
  bool edited_all_ = true;
  std::array<int, kKinds> counts_{};
};

}

#endif
