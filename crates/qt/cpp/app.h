#ifndef OPSIS_QT_SPIKE_CPP_APP_H_
#define OPSIS_QT_SPIKE_CPP_APP_H_

#include <QColor>
#include <QHash>
#include <QIcon>
#include <QPixmap>
#include <QString>
#include <optional>

#include "rust/cxx.h"

namespace opsis {

namespace bridge {
struct Session;
}

QString ToQ(const rust::String& text);
rust::String ToRust(const QString& text);

class App {
 public:
  static App& Get();

  int Run(rust::Box<bridge::Session> host);
  bridge::Session& session();
  QColor Token(const QString& name);
  QPixmap Glyph(const QString& name, const QColor& color, int size,
                double stroke = 2.0);
  QIcon Icon(const QString& name, const QColor& color, int size = 16);
  QPixmap Logo(int size);
  void Log(const QString& line);
  void ShowError(const QString& message);

 private:
  App() = default;
  void LoadFonts();
  void ConfigureDocks();

  std::optional<rust::Box<bridge::Session>> session_;
  QHash<QString, QColor> tokens_;
  QHash<QString, QPixmap> glyphs_;
};

}

#endif
