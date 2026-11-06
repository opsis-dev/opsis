#ifndef OPSIS_QT_CPP_VAR_TEXT_H_
#define OPSIS_QT_CPP_VAR_TEXT_H_

#include <QObject>
#include <QTextEdit>
#include <QTextObjectInterface>
#include <QWidget>

namespace opsis {

class ChipObject : public QObject, public QTextObjectInterface {
  Q_OBJECT
  Q_INTERFACES(QTextObjectInterface)

 public:
  explicit ChipObject(QObject* parent);

  QSizeF intrinsicSize(QTextDocument* document, int position,
                       const QTextFormat& format) override;
  void drawObject(QPainter* painter, const QRectF& rect,
                  QTextDocument* document, int position,
                  const QTextFormat& format) override;
};

class VarSurface : public QTextEdit {
  Q_OBJECT

 public:
  VarSurface(bool multiline, bool pauses, QWidget* parent = nullptr);

  void SetValue(const QString& value);
  QString Value() const;
  void InsertVariable(const QString& name);
  void InsertPause();
  bool HasDraft() const { return Value() != committed_; }
  int MissingChips() const;

 signals:
  void Committed(const QString& value);

 protected:
  bool event(QEvent* event) override;
  void keyPressEvent(QKeyEvent* event) override;
  void focusOutEvent(QFocusEvent* event) override;
  void insertFromMimeData(const QMimeData* source) override;
  bool canInsertFromMimeData(const QMimeData* source) const override;

 private:
  void InsertText(const QString& text);
  void FitHeight();
  void Commit();

  bool multiline_;
  bool pauses_;
  QString committed_;
};

class VarText : public QWidget {
  Q_OBJECT

 public:
  VarText(const QString& value, bool multiline, bool pauses,
          const QString& placeholder, QWidget* parent = nullptr);

  VarSurface* surface() const { return surface_; }

 signals:
  void Committed(const QString& value);

 private:
  void ShowVariables(QWidget* anchor);

  VarSurface* surface_;
};

}

#endif
