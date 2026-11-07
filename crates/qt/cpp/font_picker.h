#ifndef OPSIS_QT_CPP_FONT_PICKER_H_
#define OPSIS_QT_CPP_FONT_PICKER_H_

#include <QFrame>
#include <QPushButton>
#include <QString>
#include <functional>

class QLabel;
class QLineEdit;
class QListWidget;

namespace opsis {

using OnFontPick = std::function<void(const QString& family)>;

class FontPopover : public QFrame {
  Q_OBJECT

 public:
  FontPopover(QWidget* anchor, const QString& value, const QString& empty_label,
              OnFontPick on_pick);

 protected:
  void hideEvent(QHideEvent* event) override;

 private:
  void Filter(const QString& query);
  void Choose(int row);

  QLineEdit* search_;
  QListWidget* list_;
  QLabel* nothing_;
  OnFontPick on_pick_;
  bool ended_ = false;
};

class FontPicker : public QPushButton {
  Q_OBJECT

 public:
  FontPicker(QString value, QString empty_label, OnFontPick on_pick,
             QWidget* parent = nullptr);

 private:
  QString value_;
  QString empty_label_;
  OnFontPick on_pick_;
};

}

#endif
