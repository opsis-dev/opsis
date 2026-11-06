#include "app.h"

#include <QApplication>
#include <QtCore/QtResource>
#include <QDir>
#include <QFile>
#include <QFontDatabase>
#include <QMessageBox>
#include <QPainter>
#include <QStyleFactory>
#include <QSvgRenderer>
#include <QTimer>
#include <kddockwidgets/Config.h>
#include <kddockwidgets/KDDockWidgets.h>

#include "dock_style.h"
#include "event_hub.h"
#include "main_window.h"
#include "opsis-qt/src/bridge.rs.h"
#include "scenario.h"

static void InitResources() { Q_INIT_RESOURCE(resources_qrc); }

namespace opsis {

namespace {

constexpr int kSeparatorThickness = 1;

QPalette DarkPalette(App& app) {
  QPalette palette;
  palette.setColor(QPalette::Window, app.Token("background"));
  palette.setColor(QPalette::WindowText, app.Token("foreground"));
  palette.setColor(QPalette::Base, app.Token("card"));
  palette.setColor(QPalette::AlternateBase, app.Token("muted"));
  palette.setColor(QPalette::Text, app.Token("foreground"));
  palette.setColor(QPalette::Button, app.Token("secondary"));
  palette.setColor(QPalette::ButtonText, app.Token("foreground"));
  palette.setColor(QPalette::Highlight, app.Token("accent"));
  palette.setColor(QPalette::HighlightedText, app.Token("accent-foreground"));
  palette.setColor(QPalette::PlaceholderText, app.Token("muted-foreground"));
  palette.setColor(QPalette::ToolTipBase, app.Token("primary"));
  palette.setColor(QPalette::ToolTipText, app.Token("primary-foreground"));
  return palette;
}

}

QString ToQ(const rust::String& text) {
  return QString::fromUtf8(text.data(), static_cast<qsizetype>(text.size()));
}

rust::String ToRust(const QString& text) {
  const QByteArray bytes = text.toUtf8();
  return rust::String(bytes.constData(), static_cast<std::size_t>(bytes.size()));
}

App& App::Get() {
  static App app;
  return app;
}

bridge::Session& App::session() { return **session_; }

QColor App::Token(const QString& name) {
  auto found = tokens_.find(name);
  if (found != tokens_.end()) return *found;
  const QString text = ToQ(session().token(ToRust(name)));
  QColor color;
  if (text.startsWith(QStringLiteral("rgba("))) {
    const QStringList parts =
        text.mid(5, text.size() - 6).split(QLatin1Char(','));
    color = QColor(parts.value(0).trimmed().toInt(),
                   parts.value(1).trimmed().toInt(),
                   parts.value(2).trimmed().toInt(),
                   parts.value(3).trimmed().toInt());
  } else {
    color = QColor::fromString(text);
  }
  tokens_.insert(name, color);
  return color;
}

QPixmap App::Glyph(const QString& name, const QColor& color, int size,
                   double stroke) {
  const qreal ratio = qApp->devicePixelRatio();
  const QString key = QStringLiteral("%1|%2|%3|%4|%5")
                          .arg(name, color.name(QColor::HexArgb))
                          .arg(size)
                          .arg(stroke)
                          .arg(ratio);
  auto found = glyphs_.find(key);
  if (found != glyphs_.end()) return *found;
  const rust::String svg = session().icon_svg(
      ToRust(name), ToRust(color.name(QColor::HexRgb)), stroke);
  QSvgRenderer renderer(QByteArray(svg.data(), static_cast<qsizetype>(svg.size())));
  QPixmap pixmap(QSize(size, size) * ratio);
  pixmap.fill(Qt::transparent);
  {
    QPainter painter(&pixmap);
    painter.setRenderHint(QPainter::Antialiasing);
    painter.setOpacity(color.alphaF());
    renderer.render(&painter);
  }
  pixmap.setDevicePixelRatio(ratio);
  glyphs_.insert(key, pixmap);
  return pixmap;
}

QIcon App::Icon(const QString& name, const QColor& color, int size) {
  QIcon icon(Glyph(name, color, size));
  icon.addPixmap(Glyph(name, Token("muted-foreground"), size), QIcon::Disabled);
  return icon;
}

QPixmap App::Logo(int size) {
  const qreal ratio = qApp->devicePixelRatio();
  QSvgRenderer renderer(QStringLiteral(":/opsis/logo.svg"));
  QPixmap pixmap(QSize(size, size) * ratio);
  pixmap.fill(Qt::transparent);
  QPainter painter(&pixmap);
  painter.setRenderHint(QPainter::Antialiasing);
  renderer.render(&painter);
  painter.end();
  pixmap.setDevicePixelRatio(ratio);
  return pixmap;
}

void App::Log(const QString& line) { session().log(ToRust(line)); }

void App::ShowError(const QString& message) {
  QMessageBox box(QMessageBox::Warning, QStringLiteral("Opsis"), message,
                  QMessageBox::Ok, QApplication::activeWindow());
  box.exec();
}

void App::LoadFonts() {
  const QDir dir(ToQ(session().fonts_dir()));
  const QStringList files = {
      QStringLiteral("inter/inter-latin-wght-normal.woff2"),
      QStringLiteral("jetbrains-mono/jetbrains-mono-latin-wght-normal.woff2"),
  };
  for (const QString& file : files) {
    const int id = QFontDatabase::addApplicationFont(dir.filePath(file));
    Log(QStringLiteral("font %1: %2")
            .arg(file, id < 0 ? QStringLiteral("not loaded")
                              : QFontDatabase::applicationFontFamilies(id)
                                    .join(QLatin1Char(','))));
  }
}

void App::ConfigureDocks() {
  KDDockWidgets::initFrontend(KDDockWidgets::FrontendType::QtWidgets);
  auto& config = KDDockWidgets::Config::self();
  config.setViewFactory(new DockViewFactory());
  config.setSeparatorThickness(kSeparatorThickness);
  config.setFlags(KDDockWidgets::Config::Flag_HideTitleBarWhenTabsVisible |
                  KDDockWidgets::Config::Flag_AlwaysShowTabs |
                  KDDockWidgets::Config::Flag_AllowReorderTabs |
                  KDDockWidgets::Config::Flag_TitleBarHasMaximizeButton);
}

int App::Run(rust::Box<bridge::Session> host) {
  static int argc = 1;
  static char name[] = "opsis-qt";
  static char* argv[] = {name, nullptr};
  QApplication::setHighDpiScaleFactorRoundingPolicy(
      Qt::HighDpiScaleFactorRoundingPolicy::PassThrough);
  QApplication application(argc, argv);
  QApplication::setApplicationName(QStringLiteral("Opsis"));
  QApplication::setOrganizationName(QStringLiteral("Opsis"));
  InitResources();
  session_.emplace(std::move(host));
  Log(QStringLiteral("session ready, platform %1")
          .arg(QApplication::platformName()));
  LoadFonts();
  QApplication::setStyle(QStyleFactory::create(QStringLiteral("Fusion")));
  QApplication::setPalette(DarkPalette(*this));
  QFont font(QStringLiteral("Inter"));
  font.setPixelSize(13);
  QApplication::setFont(font);
  qApp->setStyleSheet(ToQ(session().style_sheet()));
  ConfigureDocks();
  EventHub::Instance();
  session().set_active(QGuiApplication::applicationState() == Qt::ApplicationActive);
  QObject::connect(qApp, &QGuiApplication::applicationStateChanged,
                   [this](Qt::ApplicationState state) {
                     session().set_active(state == Qt::ApplicationActive);
                   });
  session().start_pump();
  auto* window = new MainWindow();
  window->show();
  QTimer::singleShot(0, window, [this] {
    QFile status(QStringLiteral("/proc/self/status"));
    QString rss;
    if (status.open(QIODevice::ReadOnly)) {
      for (const QByteArray& line : status.readAll().split('\n')) {
        if (line.startsWith("VmRSS:")) rss = QString::fromLatin1(line.mid(6).trimmed());
      }
    }
    Log(QStringLiteral("first window shown, rss %1").arg(rss));
  });
  Scenario::MaybeStart(window);
  const int code = QApplication::exec();
  delete window;
  return code;
}

}

namespace opsis::bridge {

std::int32_t run_app(rust::Box<Session> session) {
  return opsis::App::Get().Run(std::move(session));
}

}
