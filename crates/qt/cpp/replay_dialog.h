#ifndef OPSIS_QT_CPP_REPLAY_DIALOG_H_
#define OPSIS_QT_CPP_REPLAY_DIALOG_H_

#include <QDialog>
#include <QPixmap>
#include <QPointer>
#include <QString>
#include <QTimer>
#include <QWidget>
#include <memory>

class QLabel;
class QLayout;
class QListWidget;
class QStackedWidget;
class QVBoxLayout;

namespace opsis {

namespace bridge {
struct ReplayView;
struct TraceView;
}

class FilePlayer;
class Segmented;

class TraceShot : public QWidget {
  Q_OBJECT

 public:
  explicit TraceShot(QWidget* parent = nullptr);
  void Show(const QString& file, QSize viewport, const QRectF& box,
            const QPointF& point, bool has_box, bool has_point);
  void Clear();
  bool HasPicture() const { return !picture_.isNull(); }

 protected:
  void paintEvent(QPaintEvent* event) override;

 private:
  QPixmap picture_;
  QSize viewport_;
  QRectF box_;
  QPointF point_;
  bool has_box_ = false;
  bool has_point_ = false;
};

class TracePanel : public QWidget {
  Q_OBJECT

 public:
  explicit TracePanel(const QString& trace_id, QWidget* parent = nullptr);
  ~TracePanel() override;
  TracePanel(const TracePanel&) = delete;
  TracePanel& operator=(const TracePanel&) = delete;
  TracePanel(TracePanel&&) = delete;
  TracePanel& operator=(TracePanel&&) = delete;
  int selected() const { return selected_; }
  void Select(int index);

 private:
  void Build();
  void ShowSelected();
  void FillTabs();

  std::unique_ptr<bridge::TraceView> view_;
  int selected_ = -1;
  QString moment_ = QStringLiteral("after");
  QListWidget* list_ = nullptr;
  QLabel* title_ = nullptr;
  QLabel* badge_ = nullptr;
  Segmented* moments_ = nullptr;
  TraceShot* shot_ = nullptr;
  QLabel* call_ = nullptr;
  QWidget* console_ = nullptr;
  QWidget* network_ = nullptr;
};

void AddReplayButtons(QLayout* row, const QString& node, bool can_replay,
                      bool has_trace, QWidget* parent);

class ReplayDialog : public QDialog {
  Q_OBJECT

 public:
  static void Open(QWidget* parent, const bridge::ReplayView& view);

 private:
  ReplayDialog(QWidget* parent, const bridge::ReplayView& view);
  QWidget* Footage(const bridge::ReplayView& view);
  void Follow();

  QPointer<FilePlayer> player_;
  QListWidget* actions_ = nullptr;
  std::vector<double> times_;
  QTimer follow_;
};

}

#endif
