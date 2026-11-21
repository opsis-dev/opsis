#ifndef OPSIS_QT_CPP_AGENT_PANEL_H_
#define OPSIS_QT_CPP_AGENT_PANEL_H_

#include <QFrame>
#include <QLabel>
#include <QLineEdit>
#include <QPlainTextEdit>
#include <QScrollArea>
#include <QStackedWidget>
#include <QTimer>
#include <QToolButton>
#include <QVBoxLayout>
#include <QWidget>
#include <vector>

namespace opsis {

namespace bridge {
struct AgentPanelView;
struct AgentEntryView;
struct AgentToolView;
struct AgentAskView;
}

class ElidedLabel : public QLabel {
 public:
  explicit ElidedLabel(QWidget* parent = nullptr);
  void SetFull(const QString& text);
  QSize minimumSizeHint() const override;

 protected:
  void resizeEvent(QResizeEvent* event) override;

 private:
  void Elide();

  QString full_;
};

class AgentPrompt : public QPlainTextEdit {
  Q_OBJECT

 public:
  explicit AgentPrompt(QWidget* parent = nullptr);

 signals:
  void SendRequested();
  void FocusChanged(bool focused);

 protected:
  void keyPressEvent(QKeyEvent* event) override;
  void focusInEvent(QFocusEvent* event) override;
  void focusOutEvent(QFocusEvent* event) override;
};

class AgentPanel : public QWidget {
  Q_OBJECT

 public:
  explicit AgentPanel(QWidget* parent = nullptr);
  void Sync();

 protected:
  void showEvent(QShowEvent* event) override;

 private:
  QWidget* BuildList();
  QWidget* BuildThread();
  void SyncList(const bridge::AgentPanelView& view);
  void SyncThread(const bridge::AgentPanelView& view);
  void SyncEntries(const bridge::AgentPanelView& view);
  void SyncAsks(const bridge::AgentPanelView& view);
  void SyncNotes(const bridge::AgentPanelView& view);
  void SyncComposer(const bridge::AgentPanelView& view);
  void SyncWorking();
  void FilterThreads();
  void NewThread();
  void Send();
  void FitPrompt();
  QWidget* EntryWidget(const bridge::AgentEntryView& entry);
  QWidget* ToolWidget(const bridge::AgentToolView& tool);
  QWidget* AskWidget(const bridge::AgentAskView& ask);
  QFrame* Note(const QString& icon, const QString& tone, const QString& title,
               const QString& text);

  QStackedWidget* pages_ = nullptr;
  QLineEdit* search_ = nullptr;
  QWidget* list_off_ = nullptr;
  QVBoxLayout* threads_ = nullptr;
  QLabel* threads_empty_ = nullptr;
  ElidedLabel* title_ = nullptr;
  QToolButton* menu_ = nullptr;
  QScrollArea* scroll_ = nullptr;
  QWidget* empty_ = nullptr;
  QVBoxLayout* entries_ = nullptr;
  QWidget* asks_ = nullptr;
  QVBoxLayout* asks_layout_ = nullptr;
  QLabel* working_ = nullptr;
  QWidget* notes_ = nullptr;
  QVBoxLayout* notes_layout_ = nullptr;
  QFrame* composer_ = nullptr;
  AgentPrompt* prompt_ = nullptr;
  QToolButton* mode_ = nullptr;
  QToolButton* send_ = nullptr;
  QTimer ticker_;
  std::vector<QString> shown_entries_;
  std::vector<QWidget*> entry_widgets_;
  QString shown_asks_;
  QString shown_notes_;
  QString shown_threads_;
  QString thread_;
  QString status_;
  qint64 working_since_ = 0;
  bool live_ = false;
  bool busy_ = false;
};

}

#endif
