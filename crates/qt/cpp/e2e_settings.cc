#include <QApplication>
#include <QCheckBox>
#include <QDialog>
#include <QDir>
#include <QFileDialog>
#include <QTimer>
#include <QLabel>
#include <QLineEdit>
#include <QListWidget>
#include <QPlainTextEdit>
#include <QPushButton>
#include <QToolButton>
#include <QtTest/QTest>
#include <memory>

#include "app.h"
#include "main_window.h"
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

constexpr int kDialogMs = 15'000;

QWidget* Dialog(const QString& name) {
  for (QWidget* top : QApplication::topLevelWidgets()) {
    if (top->isVisible() && top->objectName() == name) return top;
  }
  return nullptr;
}

int Library() {
  QWidget* settings = Settings();
  if (settings == nullptr) return -1;
  return static_cast<int>(
      settings->findChildren<QFrame*>(QStringLiteral("FontRow")).size());
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
            QStringLiteral("Recording"), QStringLiteral("Fonts"),
            QStringLiteral("API"),
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
  Add(
      QStringLiteral("settings-fonts-page-starts-empty"),
      [] { OpenPage(QStringLiteral("Fonts")); }, 400,
      [] {
        return Expect(ShowsText(QStringLiteral("No fonts added yet.")),
                      QStringLiteral("the library isn't empty"));
      });
  Add(
      QStringLiteral("settings-add-font-files-asks-for-files"),
      [this] { Click(Settings(), QStringLiteral("Add font files or ZIP…")); },
      900,
      [] {
        return Expect(qobject_cast<QFileDialog*>(
                          QApplication::activeModalWidget()) != nullptr,
                      QStringLiteral("no file dialog opened"));
      });
  Until(
      QStringLiteral("settings-pick-a-font-file"),
      [] {
        auto* dialog =
            qobject_cast<QFileDialog*>(QApplication::activeModalWidget());
        if (dialog == nullptr) return;
        QDir fixtures(qEnvironmentVariable("OPSIS_QT_FIXTURES"));
        const QStringList fonts = fixtures.entryList(
            {QStringLiteral("sample-font.*")}, QDir::Files);
        auto* name = dialog->findChild<QLineEdit*>(QStringLiteral("fileNameEdit"));
        if (fonts.isEmpty() || name == nullptr) {
          dialog->reject();
          return;
        }
        name->setFocus();
        QTest::keyClicks(name, fixtures.filePath(fonts.first()));
        QTest::keyClick(name, Qt::Key_Return);
      },
      kDialogMs, [] { return Dialog(QStringLiteral("FontReviewDialog")) != nullptr; },
      [] {
        QWidget* review = Dialog(QStringLiteral("FontReviewDialog"));
        return Expect(review != nullptr &&
                          !review->findChildren<QCheckBox*>().isEmpty(),
                      QStringLiteral("the font file wasn't offered to add"));
      });
  Until(
      QStringLiteral("settings-add-the-reviewed-font"),
      [this] { Click(Dialog(QStringLiteral("FontReviewDialog")), QStringLiteral("Add 1 font")); },
      kDialogMs, [] { return Library() == 1; },
      [] {
        return Expect(Library() == 1 &&
                          Dialog(QStringLiteral("FontReviewDialog")) == nullptr,
                      QStringLiteral("the library has %1 fonts").arg(Library()));
      });
  Add(
      QStringLiteral("settings-licences-list-your-font"),
      [] { OpenPage(QStringLiteral("Licences")); }, 500,
      [] {
        return Expect(ShowsText(QStringLiteral("Your fonts")),
                      QStringLiteral("the licences don't list your fonts"));
      });
  Add(QStringLiteral("settings-remove-asks-first"), [this] {
    OpenPage(QStringLiteral("Fonts"));
    QWidget* settings = Settings();
    for (QToolButton* button : settings->findChildren<QToolButton*>()) {
      if (button->isVisible() &&
          button->toolTip().startsWith(QStringLiteral("Remove "))) {
        QTimer::singleShot(0, button, [button] {
          QTest::mouseClick(button, Qt::LeftButton);
        });
        return;
      }
    }
    Miss(QStringLiteral("no remove button"));
  }, 700, [] {
    return Expect(Dialog(QStringLiteral("ConfirmDialog")) != nullptr,
                  QStringLiteral("removing didn't ask first"));
  });
  Add(
      QStringLiteral("settings-remove-the-font"),
      [this] { Click(Dialog(QStringLiteral("ConfirmDialog")), QStringLiteral("Remove")); },
      700,
      [] {
        return Expect(Library() == 0 &&
                          ShowsText(QStringLiteral("No fonts added yet.")),
                      QStringLiteral("the font is still in the library"));
      });
  Until(
      QStringLiteral("settings-installed-fonts-list"),
      [this] { Click(Settings(), QStringLiteral("Add installed fonts…")); },
      kDialogMs, [] { return Dialog(QStringLiteral("InstalledFonts")) != nullptr; },
      [] {
        QWidget* dialog = Dialog(QStringLiteral("InstalledFonts"));
        auto* list = dialog == nullptr
                         ? nullptr
                         : dialog->findChild<QListWidget*>(QStringLiteral("SearchList"));
        const int count = list == nullptr ? 0 : list->count();
        if (dialog != nullptr) dialog->close();
        return Expect(count > 0, QStringLiteral("no installed fonts listed"));
      });
  Add(QStringLiteral("settings-online-asks-before-connecting"), [this] {
    if (QAbstractButton* online =
            Find(Settings(), QStringLiteral("Get fonts online…"))) {
      QTimer::singleShot(0, online, [online] {
        QTest::mouseClick(online, Qt::LeftButton);
      });
    }
  }, 700, [] {
    QWidget* ask = Dialog(QStringLiteral("ConfirmDialog"));
    const bool asked = ask != nullptr;
    if (ask != nullptr) qobject_cast<QDialog*>(ask)->reject();
    return Expect(asked && Dialog(QStringLiteral("OnlineFonts")) == nullptr,
                  QStringLiteral("going online didn't ask first"));
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

void Scenario::BuildFontPicker(MainWindow* window) {
  const auto picker = [window]() -> QPushButton* {
    for (auto* button :
         window->inspector_dock()->widget()->findChildren<QPushButton*>(
             QStringLiteral("FontPicker"))) {
      if (button->isVisible() &&
          button->accessibleName() == QStringLiteral("Same as text")) {
        return button;
      }
    }
    return nullptr;
  };
  const auto popover = []() -> QWidget* {
    QWidget* open = QApplication::activePopupWidget();
    return open != nullptr &&
                   open->findChild<QListWidget*>(QStringLiteral("FontList")) != nullptr
               ? open
               : nullptr;
  };
  Add(
      QStringLiteral("font-picker-opens"),
      [picker] {
        App::Get().session().select({});
        QTimer::singleShot(300, [picker] {
          if (QPushButton* button = picker()) {
            QTest::mouseClick(button, Qt::LeftButton);
          }
        });
      },
      900,
      [popover] {
        QWidget* open = popover();
        auto* list = open == nullptr
                         ? nullptr
                         : open->findChild<QListWidget*>(QStringLiteral("FontList"));
        return Expect(list != nullptr && list->count() > 10,
                      QStringLiteral("the font list didn't open"));
      });
  Add(
      QStringLiteral("font-picker-search-and-pick"),
      [popover] {
        QWidget* open = popover();
        auto* search = open == nullptr ? nullptr : open->findChild<QLineEdit*>();
        if (search == nullptr) return;
        QTest::keyClicks(search, QStringLiteral("Fraunces"));
        QTest::keyClick(search, Qt::Key_Return);
      },
      700,
      [popover] {
        return Expect(App::Get().session().project().heading_font ==
                              ToRust(QStringLiteral("Fraunces")) &&
                          popover() == nullptr,
                      QStringLiteral("the heading font is '%1'")
                          .arg(ToQ(App::Get().session().project().heading_font)));
      });
  Add(
      QStringLiteral("undo-font-pick"),
      [this, window] { Click(window, QStringLiteral("Undo")); }, 600,
      [] {
        return Expect(App::Get().session().project().heading_font.empty(),
                      QStringLiteral("undo didn't bring the heading font back"));
      });
  Add(
      QStringLiteral("font-picker-manage-opens-settings"),
      [picker, popover] {
        if (QPushButton* button = picker()) {
          QTest::mouseClick(button, Qt::LeftButton);
        }
        QWidget* open = popover();
        auto* list = open == nullptr
                         ? nullptr
                         : open->findChild<QListWidget*>(QStringLiteral("FontList"));
        if (list == nullptr) return;
        QListWidgetItem* manage = list->item(list->count() - 1);
        QTest::mouseClick(list->viewport(), Qt::LeftButton, {},
                          list->visualItemRect(manage).center());
      },
      900,
      [] {
        QWidget* settings = Settings();
        auto* pages = settings == nullptr ? nullptr : settings->findChild<QListWidget*>();
        const bool fonts = pages != nullptr && pages->currentItem() != nullptr &&
                           pages->currentItem()->text() == QStringLiteral("Fonts");
        if (settings != nullptr) settings->close();
        return Expect(fonts, QStringLiteral("Add or manage fonts didn't open the "
                                            "Fonts settings"));
      });
}

}
