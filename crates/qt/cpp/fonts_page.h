#ifndef OPSIS_QT_CPP_FONTS_PAGE_H_
#define OPSIS_QT_CPP_FONTS_PAGE_H_

#include <QDialog>
#include <QFrame>
#include <QPointer>
#include <QString>
#include <QWidget>
#include <functional>
#include <vector>

class QLabel;
class QLineEdit;
class QListWidget;
class QVBoxLayout;

namespace opsis {

QFont SampleFont(const QString& file, const QString& family);

class SearchListDialog : public QDialog {
  Q_OBJECT

 public:
  SearchListDialog(QWidget* parent, const QString& name, const QString& title,
                   const QString& placeholder);
  void SetDescription(const QString& text);
  void SetStatus(QWidget* status);
  QVBoxLayout* footer() const { return footer_; }
  QString query() const;
  void Fill(int count, const std::function<bool(int)>& matches,
            const std::function<QWidget*(int, QWidget*)>& make);

 signals:
  void QueryChanged();

 private:
  QLabel* description_;
  QLineEdit* search_;
  QWidget* status_holder_;
  QListWidget* list_;
  QVBoxLayout* footer_;
};

class FontsCard : public QWidget {
  Q_OBJECT

 public:
  explicit FontsCard(QWidget* parent = nullptr);

 private:
  void Reload();
  void Review();
  void ShowInstalled();
  void AskOnline();
  void ShowOnline();

  QVBoxLayout* rows_;
  QPointer<QDialog> review_;
  QPointer<SearchListDialog> online_;
};

}

#endif
