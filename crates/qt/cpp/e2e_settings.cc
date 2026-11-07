#include <QApplication>
#include <QLabel>
#include <QLineEdit>
#include <QListWidget>
#include <QPlainTextEdit>
#include <QPushButton>
#include <QToolButton>
#include <QtTest/QTest>
#include <memory>

#include "app.h"
#include "opsis-qt/src/bridge.rs.h"
#include "scenario.h"
#include "widgets.h"

namespace opsis {

namespace {

QString Expect(bool ok, const QString& what) { return ok ? QString() : what; }

QWidget* Settings() {
  for (QWidget* top : QApplication::topLevelWidgets()) {
    if (top->isVisible() &&
        top->objectName() == QStringLiteral("SettingsWindow")) {
      return top;
    }
  }
  return nullptr;
}

QStringList Pages() {
  QStringList labels;
  QWidget* settings = Settings();
  auto* list = settings == nullptr ? nullptr : settings->findChild<QListWidget*>();
  for (int i = 0; list != nullptr && i < list->count(); ++i) {
    labels << list->item(i)->text();
  }
  return labels;
}

void OpenPage(const QString& label) {
  QWidget* settings = Settings();
  auto* list = settings == nullptr ? nullptr : settings->findChild<QListWidget*>();
  for (int i = 0; list != nullptr && i < list->count(); ++i) {
    if (list->item(i)->text() == label) {
      QTest::mouseClick(list->viewport(), Qt::LeftButton, {},
                        list->visualItemRect(list->item(i)).center());
    }
  }
}

template <typename T>
T* Named(const QString& name) {
  QWidget* settings = Settings();
  return settings == nullptr ? nullptr : settings->findChild<T*>(name);
}

bool ShowsText(const QString& text) {
  QWidget* settings = Settings();
  if (settings == nullptr) return false;
  for (const QLabel* label : settings->findChildren<QLabel*>()) {
    if (label->isVisible() && label->text().contains(text)) return true;
  }
  return false;
}

QString LastRunsLabel() {
  for (const auto& choice : App::Get().session().settings().trace_retentions) {
    if (ToQ(choice.value) == QStringLiteral("last-n")) return ToQ(choice.label);
  }
  return {};
}

}

void Scenario::BuildSettings() {
  auto started = std::make_shared<bridge::SettingsView>(
      App::Get().session().settings());
  Add(
      QStringLiteral("settings-lists-every-page"),
      [started] { *started = App::Get().session().settings(); }, 200,
      [] {
        const QStringList want = {
            QStringLiteral("General"),   QStringLiteral("Projects"),
            QStringLiteral("Editor"),    QStringLiteral("Engine"),
            QStringLiteral("Recording"), QStringLiteral("API"),
            QStringLiteral("Licences"),  QStringLiteral("About")};
        return Expect(Pages() == want, QStringLiteral("the pages are %1")
                                           .arg(Pages().join(", ")));
      });
  Add(
      QStringLiteral("settings-live-view-switch"),
      [] {
        OpenPage(QStringLiteral("Recording"));
        if (auto* toggle = Named<ToggleSwitch>(QStringLiteral("LiveAutoOpen"))) {
          QTest::mouseClick(toggle, Qt::LeftButton);
        }
      },
      600,
      [started] {
        return Expect(App::Get().session().settings().live_auto_open !=
                          started->live_auto_open,
                      QStringLiteral("the live view switch didn't change"));
      });
  Add(
      QStringLiteral("settings-keep-the-last-few-traces"),
      [this] {
        if (QWidget* settings = Settings()) {
          Click(settings, LastRunsLabel());
        }
      },
      600,
      [] {
        auto* keep = Named<QLineEdit>(QStringLiteral("KeepLast"));
        return Expect(App::Get().session().settings().trace_retention ==
                              ToRust(QStringLiteral("last-n")) &&
                          keep != nullptr && keep->isVisible(),
                      QStringLiteral("choosing the last few runs didn't show "
                                     "how many"));
      });
  Add(
      QStringLiteral("settings-type-how-many-traces"),
      [] {
        auto* keep = Named<QLineEdit>(QStringLiteral("KeepLast"));
        if (keep == nullptr) return;
        keep->setFocus();
        keep->selectAll();
        QTest::keyClicks(keep, QStringLiteral("7"));
        QTest::keyClick(keep, Qt::Key_Return);
      },
      600,
      [] {
        return Expect(App::Get().session().settings().keep_last == 7,
                      QStringLiteral("the traces to keep are %1")
                          .arg(App::Get().session().settings().keep_last));
      });
  Add(
      QStringLiteral("settings-engine-page"),
      [] { OpenPage(QStringLiteral("Engine")); }, 400,
      [this] {
        auto* state = Named<QLabel>(QStringLiteral("EngineState"));
        return Expect(state != nullptr && state->isVisible() &&
                          state->text() == QStringLiteral("Not set up") &&
                          Find(Settings(), QStringLiteral("Download engine")) !=
                              nullptr,
                      QStringLiteral("the Engine page doesn't offer the "
                                     "download"));
      });
  Add(
      QStringLiteral("settings-licences-page"),
      [this] {
        OpenPage(QStringLiteral("Licences"));
        if (QWidget* settings = Settings()) {
          Click(settings, QStringLiteral("Show the licence"));
        }
      },
      500,
      [] {
        QWidget* settings = Settings();
        bool text = false;
        for (const QPlainTextEdit* shown :
             settings->findChildren<QPlainTextEdit*>()) {
          text = text ||
                 (shown->isVisible() && shown->toPlainText().size() > 200);
        }
        return Expect(ShowsText(QStringLiteral("Built-in fonts")) &&
                          ShowsText(QStringLiteral("Inter")) && text,
                      QStringLiteral("the licences didn't list or show"));
      });
  Add(
      QStringLiteral("settings-about-page"),
      [] { OpenPage(QStringLiteral("About")); }, 400,
      [] {
        return Expect(ShowsText(QStringLiteral("KDDockWidgets 2.5.1")) &&
                          ShowsText(QStringLiteral("Qt")),
                      QStringLiteral("About doesn't list Qt and "
                                     "KDDockWidgets"));
      });
  Add(QStringLiteral("settings-put-back"), [this, started] {
    OpenPage(QStringLiteral("Recording"));
    if (auto* toggle = Named<ToggleSwitch>(QStringLiteral("LiveAutoOpen"))) {
      QTest::mouseClick(toggle, Qt::LeftButton);
    }
    for (const auto& choice : started->trace_retentions) {
      if (choice.value == started->trace_retention) {
        Click(Settings(), ToQ(choice.label));
      }
    }
  }, 600, [started] {
    const bridge::SettingsView now = App::Get().session().settings();
    return Expect(now.live_auto_open == started->live_auto_open &&
                      now.trace_retention == started->trace_retention,
                  QStringLiteral("the settings didn't go back"));
  });
}

}
