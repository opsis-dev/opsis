#ifndef OPSIS_QT_CPP_SCRIPT_PANEL_H_
#define OPSIS_QT_CPP_SCRIPT_PANEL_H_

#include <QLabel>
#include <QPlainTextEdit>
#include <QPushButton>
#include <QSpinBox>
#include <QSplitter>
#include <QSyntaxHighlighter>
#include <QTimer>
#include <QWidget>
#include <cstdint>
#include <memory>
#include <vector>

#include "widgets.h"

namespace opsis {

namespace bridge {
struct ScriptView;
}

class ScriptHighlighter : public QSyntaxHighlighter {
  Q_OBJECT

 public:
  struct Range {
    int start = 0;
    int length = 0;
    QTextCharFormat format;
  };

  explicit ScriptHighlighter(QTextDocument* document);
  void SetRanges(std::vector<Range> ranges);

 protected:
  void highlightBlock(const QString& text) override;

 private:
  std::vector<Range> ranges_;
};

class ScriptEditor : public QPlainTextEdit {
  Q_OBJECT

 public:
  explicit ScriptEditor(QWidget* parent = nullptr);
  void MarkLine(int line);
  void ClearMark();
  void SetMarks(const QColor& bracket, const QColor& error);

 signals:
  void RunRequested();

 protected:
  bool event(QEvent* event) override;
  void keyPressEvent(QKeyEvent* event) override;

 private:
  bool Handle(const QString& key);
  void ShowBracket();

  int marked_ = 0;
  QColor bracket_;
  QColor error_;
};

class ScriptPanel : public QWidget {
  Q_OBJECT

 public:
  explicit ScriptPanel(QWidget* parent = nullptr);
  ~ScriptPanel() override;
  ScriptPanel(const ScriptPanel&) = delete;
  ScriptPanel& operator=(const ScriptPanel&) = delete;
  ScriptPanel(ScriptPanel&&) = delete;
  ScriptPanel& operator=(ScriptPanel&&) = delete;
  void Rebuild();
  void Run();
  void Save();
  void Rename();
  void Delete();
  QSplitter* split() const { return split_; }
  ScriptEditor* editor() const { return editor_; }

 protected:
  void resizeEvent(QResizeEvent* event) override;
  bool eventFilter(QObject* watched, QEvent* event) override;

 private:
  void Load(const QString& file);
  void Highlight();
  void ApplyLook();
  void ShowOutput(const bridge::ScriptView& view);
  void GoToLine(int line);

  Combo* files_;
  QPushButton* new_;
  QPushButton* rename_;
  QPushButton* delete_;
  QPushButton* run_;
  QPushButton* stop_;
  QLabel* badge_;
  QLabel* status_;
  Combo* theme_;
  QSpinBox* size_;
  QSplitter* split_;
  ScriptEditor* editor_;
  ScriptHighlighter* highlighter_;
  QPlainTextEdit* output_;
  QTimer save_;
  QTimer highlight_;
  QString open_;
  QString dir_;
  bool loading_ = false;
  std::uint64_t last_line_ = 0;
  std::vector<int> output_lines_;
  std::unique_ptr<bridge::ScriptView> last_;
  QString error_colour_;
  QString warning_colour_;
};

}

#endif
