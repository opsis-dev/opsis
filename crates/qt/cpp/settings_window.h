#ifndef OPSIS_QT_CPP_SETTINGS_WINDOW_H_
#define OPSIS_QT_CPP_SETTINGS_WINDOW_H_

#include <QLabel>
#include <QLineEdit>
#include <QListWidget>
#include <QProgressBar>
#include <QPushButton>
#include <QSlider>
#include <QStackedWidget>
#include <QVBoxLayout>
#include <QWidget>

namespace opsis {

class Segmented;
class ToggleSwitch;

class SettingsWindow : public QWidget {
  Q_OBJECT

 public:
  static constexpr int kGeneral = 0;
  static constexpr int kProjects = 1;
  static constexpr int kEditor = 2;
  static constexpr int kEngine = 3;
  static constexpr int kRecording = 4;
  static constexpr int kFonts = 5;
  static constexpr int kAssistant = 6;
  static constexpr int kLicences = 7;
  static constexpr int kAbout = 8;

  explicit SettingsWindow(QWidget* parent = nullptr);
  void ShowPage(int index);
  int page_count() const;

 private:
  QWidget* GeneralPage();
  QWidget* ProjectsPage();
  QWidget* EditorPage();
  QWidget* EnginePage();
  QWidget* RecordingPage();
  QWidget* FontsPage();
  QWidget* AssistantPage();
  QWidget* LicencesPage();
  QWidget* AboutPage();
  QWidget* Page(const QString& title, const QString& description,
                const QList<QWidget*>& cards);
  void Load();
  void Patch(const QString& section, const QString& key,
             const QJsonValue& value);

  QListWidget* sidebar_;
  QStackedWidget* pages_;
  Segmented* on_start_ = nullptr;
  ToggleSwitch* confirm_exit_ = nullptr;
  Segmented* encode_speed_ = nullptr;
  QLineEdit* export_threads_ = nullptr;
  QLineEdit* default_dir_ = nullptr;
  Segmented* canvas_mode_ = nullptr;
  ToggleSwitch* assistant_enabled_ = nullptr;
  QLabel* default_dir_note_ = nullptr;
  QPushButton* default_dir_reset_ = nullptr;
  Segmented* trace_retention_ = nullptr;
  QWidget* keep_last_row_ = nullptr;
  QLineEdit* keep_last_ = nullptr;
  QSlider* live_fps_ = nullptr;
  QLabel* live_fps_value_ = nullptr;
  ToggleSwitch* live_auto_open_ = nullptr;
  QLabel* assistant_state_ = nullptr;
  QLabel* assistant_error_ = nullptr;
};

}

#endif
