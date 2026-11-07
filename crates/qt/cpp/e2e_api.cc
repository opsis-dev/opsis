#include <QApplication>
#include <QCheckBox>
#include <QFile>

#include "app.h"
#include "main_window.h"
#include "opsis-qt/src/bridge.rs.h"
#include "scenario.h"

namespace opsis {

namespace {

constexpr int kApiChecksMs = 600'000;
constexpr int kDialogMs = 900;

QString Expect(bool ok, const QString& what) { return ok ? QString() : what; }

bool ShowsCardTitled(const QString& title) {
  for (const auto& card : App::Get().session().graph().nodes) {
    if (ToQ(card.title) == title) return true;
  }
  return false;
}

QCheckBox* DontAskAgain() {
  QWidget* dialog = QApplication::activeModalWidget();
  if (dialog == nullptr) return nullptr;
  for (QCheckBox* box : dialog->findChildren<QCheckBox*>()) {
    if (box->text() == QStringLiteral("Don't ask again")) return box;
  }
  return nullptr;
}

}

void Scenario::BuildApi() {
  const QString handoff = qEnvironmentVariable("OPSIS_QT_HANDOFF");
  Until(
      QStringLiteral("api-checks-finish"), [] {}, kApiChecksMs,
      [handoff] { return !handoff.isEmpty() && QFile::exists(handoff); },
      [handoff] {
        return Expect(!handoff.isEmpty() && QFile::exists(handoff),
                      QStringLiteral("the API checks never finished")) +
               Expect(ShowsCardTitled(QStringLiteral("API click")),
                      QStringLiteral(" the canvas doesn't show the node the "
                                     "API added and renamed (%1 cards, %2)")
                          .arg(App::Get().session().graph().nodes.size())
                          .arg(App::Get().session().has_project()
                                   ? QStringLiteral("a project is open")
                                   : QStringLiteral("no project is open")));
      });
  Add(
      QStringLiteral("close-asks-first"),
      [this] { ClickLater(window_, QStringLiteral("Close")); }, kDialogMs,
      [] {
        QWidget* dialog = QApplication::activeModalWidget();
        return Expect(dialog != nullptr &&
                          dialog->objectName() == QStringLiteral("CloseDialog"),
                      QStringLiteral("closing didn't ask"));
      });
  Add(
      QStringLiteral("dont-ask-again"),
      [] {
        if (QCheckBox* box = DontAskAgain()) box->setChecked(true);
      },
      300,
      [] {
        const QCheckBox* box = DontAskAgain();
        return Expect(box != nullptr && box->isChecked(),
                      QStringLiteral("no Don't ask again to tick"));
      });
  Add(QStringLiteral("exit"), [this] {
    if (QWidget* dialog = QApplication::activeModalWidget()) {
      ClickLater(dialog, QStringLiteral("Exit"));
    }
  });
}

}
