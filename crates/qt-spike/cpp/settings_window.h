#ifndef OPSIS_QT_SPIKE_CPP_SETTINGS_WINDOW_H_
#define OPSIS_QT_SPIKE_CPP_SETTINGS_WINDOW_H_

#include <QLineEdit>
#include <QListWidget>
#include <QStackedWidget>
#include <QWidget>

namespace opsis {

class Segmented;
class ToggleSwitch;

class SettingsWindow : public QWidget {
  Q_OBJECT

 public:
  explicit SettingsWindow(QWidget* parent = nullptr);
  void ShowPage(int index);
  int page_count() const;

 private:
  QWidget* GeneralPage();
  QWidget* ProjectsPage();
  QWidget* EditorPage();
  QWidget* ApiPage();
  QWidget* AboutPage();
  QWidget* Page(const QString& title, const QList<QWidget*>& cards);
  void Load();
  void Patch(const QString& section, const QString& key, const QJsonValue& value);

  QListWidget* sidebar_;
  QStackedWidget* pages_;
  Segmented* on_start_ = nullptr;
  ToggleSwitch* confirm_exit_ = nullptr;
  Segmented* encode_speed_ = nullptr;
  QLineEdit* default_dir_ = nullptr;
  Segmented* canvas_mode_ = nullptr;
  ToggleSwitch* api_enabled_ = nullptr;
};

}

#endif
