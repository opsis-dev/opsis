#ifndef OPSIS_QT_CPP_RUN_PANEL_H_
#define OPSIS_QT_CPP_RUN_PANEL_H_

#include <QPlainTextEdit>
#include <QScrollArea>
#include <QWidget>

namespace opsis {

class RunPanel : public QWidget {
  Q_OBJECT

 public:
  explicit RunPanel(QWidget* parent = nullptr);
  void Rebuild();

 private:
  QScrollArea* nodes_;
  QWidget* log_header_;
  QPlainTextEdit* log_;
  QScrollArea* outputs_;
  std::uint64_t last_log_ = 0;
  QString log_scope_;
  QString outputs_key_;
};

}

#endif
