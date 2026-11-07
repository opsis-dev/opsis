#include <QApplication>
#include <QDesktopServices>
#include <QFileDialog>
#include <QFileInfo>
#include <QUrl>
#include <QWidget>
#include <utility>

#include "app.h"
#include "event_hub.h"
#include "opsis-qt/src/bridge.rs.h"
#include "stream_window.h"

namespace opsis::bridge {

namespace {

QString FromStr(rust::Str text) {
  return QString::fromUtf8(text.data(), static_cast<qsizetype>(text.size()));
}

template <typename Work>
bool OnGuiThread(Work work) {
  QCoreApplication* app = QCoreApplication::instance();
  if (app == nullptr) return false;
  QMetaObject::invokeMethod(app, std::move(work), Qt::QueuedConnection);
  return true;
}

QWidget* EditorWindow() {
  for (QWidget* widget : QApplication::topLevelWidgets()) {
    if (widget->objectName() == QStringLiteral("Window")) return widget;
  }
  return QApplication::activeWindow();
}

}

void request_dialog(DialogKind kind, std::uint64_t id, rust::Str title,
                    rust::Str start, rust::Str filter_name,
                    rust::Str filter_extensions) {
  const QString heading = FromStr(title);
  const QString folder = FromStr(start);
  QString filter;
  if (!filter_name.empty()) {
    QStringList patterns;
    for (const QString& extension :
         FromStr(filter_extensions)
             .split(QLatin1Char(' '), Qt::SkipEmptyParts)) {
      patterns.append(QStringLiteral("*.") + extension);
    }
    filter = QStringLiteral("%1 (%2)").arg(FromStr(filter_name),
                                           patterns.join(QLatin1Char(' ')));
  }
  const bool queued = OnGuiThread([kind, id, heading, folder, filter] {
    QWidget* parent = QApplication::activeWindow();
    rust::Vec<rust::String> paths;
    switch (kind) {
      case DialogKind::Folder: {
        const QString chosen =
            QFileDialog::getExistingDirectory(parent, heading, folder);
        if (!chosen.isEmpty()) paths.push_back(opsis::ToRust(chosen));
        break;
      }
      case DialogKind::File: {
        const QString chosen =
            QFileDialog::getOpenFileName(parent, heading, folder, filter);
        if (!chosen.isEmpty()) paths.push_back(opsis::ToRust(chosen));
        break;
      }
      default: {
        for (const QString& chosen :
             QFileDialog::getOpenFileNames(parent, heading, folder, filter)) {
          paths.push_back(opsis::ToRust(chosen));
        }
        break;
      }
    }
    opsis::App::Get().session().answer_dialog(id, std::move(paths));
  });
  if (!queued) opsis::App::Get().session().answer_dialog(id, {});
}

void open_path(rust::Str path) {
  const QUrl url = QUrl::fromLocalFile(FromStr(path));
  OnGuiThread([url] { QDesktopServices::openUrl(url); });
}

void reveal_path(rust::Str path) {
  const QUrl folder =
      QUrl::fromLocalFile(QFileInfo(FromStr(path)).absolutePath());
  OnGuiThread([folder] { QDesktopServices::openUrl(folder); });
}

void notify(rust::Str title, rust::Str body) {
  const QString heading = FromStr(title);
  const QString text = FromStr(body);
  OnGuiThread([heading, text] {
    if (QWidget* window = EditorWindow()) QApplication::alert(window);
    emit opsis::EventHub::Instance() -> Notice(heading, text);
  });
}

void raise_editor() {
  OnGuiThread([] {
    QWidget* window = EditorWindow();
    if (window == nullptr) return;
    if (window->isMinimized()) window->showNormal();
    window->raise();
    window->activateWindow();
  });
}

void open_view_window(rust::Str id, rust::Str title) {
  const QString view = FromStr(id);
  const QString heading = FromStr(title);
  OnGuiThread([view, heading] { opsis::StreamWindow::Open(view, heading); });
}

void close_view_window(rust::Str id) {
  const QString view = FromStr(id);
  OnGuiThread([view] { opsis::StreamWindow::Close(view); });
}

void quit_app(std::int32_t code) {
  OnGuiThread([code] { QCoreApplication::exit(code); });
}

rust::String qt_version() { return {qVersion()}; }

}
