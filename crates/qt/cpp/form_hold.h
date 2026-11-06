#ifndef OPSIS_QT_CPP_FORM_HOLD_H_
#define OPSIS_QT_CPP_FORM_HOLD_H_

#include <QObject>

namespace opsis {

class FormHold : public QObject {
  Q_OBJECT

 public:
  static FormHold* Instance();
  void Begin();
  void End();
  bool Held() const { return depth_ > 0; }

 signals:
  void Released();

 private:
  FormHold() = default;

  int depth_ = 0;
};

}

#endif
