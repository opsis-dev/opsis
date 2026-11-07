#include "fonts_page.h"

#include <QCheckBox>
#include <QFontDatabase>
#include <QHBoxLayout>
#include <QHash>
#include <QLabel>
#include <QLineEdit>
#include <QListWidget>
#include <QPushButton>
#include <QScrollArea>
#include <QToolButton>
#include <QVBoxLayout>
#include <utility>

#include "app.h"
#include "event_hub.h"
#include "opsis-qt/src/bridge.rs.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr int kMaxListed = 100;
constexpr int kDialogWidth = 520;
constexpr int kListHeight = 320;
constexpr int kSampleSize = 16;

bool& OnlineAllowed() {
  static bool allowed = false;
  return allowed;
}

void Notice(const QString& text) { emit EventHub::Instance()->Notice(text, {}); }

bool Confirm(QWidget* parent, const QString& title, const QString& body,
             const QString& yes, const char* variant) {
  QDialog dialog(parent);
  dialog.setObjectName(QStringLiteral("ConfirmDialog"));
  dialog.setWindowTitle(title);
  dialog.setMinimumWidth(kDialogWidth - 80);
  auto* layout = new QVBoxLayout(&dialog);
  layout->setContentsMargins(24, 22, 24, 20);
  layout->setSpacing(10);
  layout->addWidget(MakeLabel(title, "title", &dialog));
  auto* text = MakeLabel(body, "muted", &dialog);
  text->setWordWrap(true);
  layout->addWidget(text);
  auto* buttons = new QHBoxLayout();
  buttons->addStretch(1);
  auto* cancel = MakeButton(QStringLiteral("Cancel"), "outline", QString(), &dialog);
  auto* accept = MakeButton(yes, variant, QString(), &dialog);
  QObject::connect(cancel, &QPushButton::clicked, &dialog, &QDialog::reject);
  QObject::connect(accept, &QPushButton::clicked, &dialog, &QDialog::accept);
  buttons->addWidget(cancel);
  buttons->addWidget(accept);
  layout->addLayout(buttons);
  return dialog.exec() == QDialog::Accepted;
}

}

QFont SampleFont(const QString& file, const QString& family) {
  static QHash<QString, QString> loaded;
  QString name = family;
  if (!file.isEmpty()) {
    if (!loaded.contains(file)) {
      const int id = QFontDatabase::addApplicationFont(file);
      const QStringList families =
          id < 0 ? QStringList() : QFontDatabase::applicationFontFamilies(id);
      loaded.insert(file, families.isEmpty() ? QString() : families.first());
    }
    if (!loaded.value(file).isEmpty()) name = loaded.value(file);
  }
  QFont font(name);
  font.setPixelSize(kSampleSize);
  return font;
}

SearchListDialog::SearchListDialog(QWidget* parent, const QString& name,
                                   const QString& title,
                                   const QString& placeholder)
    : QDialog(parent),
      description_(MakeLabel(QString(), "muted", this)),
      search_(new QLineEdit(this)),
      status_holder_(new QWidget(this)),
      list_(new QListWidget(this)),
      footer_(new QVBoxLayout()) {
  setObjectName(name);
  setWindowTitle(title);
  setAttribute(Qt::WA_DeleteOnClose);
  setMinimumWidth(kDialogWidth);
  auto* layout = new QVBoxLayout(this);
  layout->setContentsMargins(24, 22, 24, 20);
  layout->setSpacing(10);
  layout->addWidget(MakeLabel(title, "title", this));
  description_->setWordWrap(true);
  layout->addWidget(description_);
  search_->setPlaceholderText(placeholder);
  search_->setAccessibleName(QStringLiteral("Search"));
  connect(search_, &QLineEdit::textChanged, this,
          &SearchListDialog::QueryChanged);
  layout->addWidget(search_);
  auto* status = new QVBoxLayout(status_holder_);
  status->setContentsMargins(0, 0, 0, 0);
  layout->addWidget(status_holder_);
  list_->setObjectName(QStringLiteral("SearchList"));
  list_->setFixedHeight(kListHeight);
  list_->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
  layout->addWidget(list_);
  layout->addLayout(footer_);
  search_->setFocus();
}

void SearchListDialog::SetDescription(const QString& text) {
  description_->setText(text);
}

void SearchListDialog::SetStatus(QWidget* status) {
  QLayout* layout = status_holder_->layout();
  while (QLayoutItem* item = layout->takeAt(0)) {
    delete item->widget();
    delete item;
  }
  if (status != nullptr) layout->addWidget(status);
}

QString SearchListDialog::query() const { return search_->text().trimmed(); }

void SearchListDialog::Fill(int count, const std::function<bool(int)>& matches,
                            const std::function<QWidget*(int, QWidget*)>& make) {
  list_->clear();
  int shown = 0;
  int found = 0;
  for (int i = 0; i < count; ++i) {
    if (!matches(i)) continue;
    found += 1;
    if (shown >= kMaxListed) continue;
    auto* item = new QListWidgetItem(list_);
    QWidget* row = make(i, list_);
    item->setSizeHint(row->sizeHint());
    list_->setItemWidget(item, row);
    shown += 1;
  }
  if (found > kMaxListed) {
    auto* item = new QListWidgetItem(list_);
    auto* more = MakeLabel(QStringLiteral("Showing the first %1 matches. Type "
                                          "to narrow the list.")
                               .arg(kMaxListed),
                           "muted", list_);
    item->setSizeHint(more->sizeHint());
    list_->setItemWidget(item, more);
  }
}

FontsCard::FontsCard(QWidget* parent) : QWidget(parent) {
  setObjectName(QStringLiteral("FontsCard"));
  auto* layout = new QVBoxLayout(this);
  layout->setContentsMargins(0, 0, 0, 0);
  auto* card = new Card(
      QStringLiteral("Your library"),
      QStringLiteral("%1 fonts come with Opsis. Fonts you add here work in "
                     "every project; a project keeps its own copy of the "
                     "fonts it uses.")
          .arg(App::Get().session().font_licences(false).size()),
      this);
  layout->addWidget(card);
  auto* rows = new QWidget(card);
  rows_ = new QVBoxLayout(rows);
  rows_->setContentsMargins(0, 0, 0, 0);
  rows_->setSpacing(4);
  card->body()->addWidget(rows);
  auto* actions = new QWidget(card);
  auto* flow = new FlowLayout(actions, 8);
  auto* files = MakeButton(QStringLiteral("Add font files or ZIP…"), "outline",
                           QStringLiteral("file-archive"), actions);
  connect(files, &QPushButton::clicked, this,
          [] { App::Get().session().add_font_files(); });
  auto* installed = MakeButton(QStringLiteral("Add installed fonts…"), "outline",
                               QStringLiteral("hard-drive"), actions);
  connect(installed, &QPushButton::clicked, this, &FontsCard::ShowInstalled);
  auto* online = MakeButton(QStringLiteral("Get fonts online…"), "outline",
                            QStringLiteral("globe"), actions);
  connect(online, &QPushButton::clicked, this, &FontsCard::AskOnline);
  for (QPushButton* button : {files, installed, online}) flow->addWidget(button);
  card->body()->addWidget(actions);
  connect(EventHub::Instance(), &EventHub::FontsChanged, this, [this] {
    Reload();
    Review();
  });
  Reload();
}

void FontsCard::Reload() {
  while (QLayoutItem* item = rows_->takeAt(0)) {
    delete item->widget();
    delete item;
  }
  const rust::Vec<bridge::FontRowView> library =
      App::Get().session().font_library();
  if (library.empty()) {
    rows_->addWidget(MakeLabel(QStringLiteral("No fonts added yet."), "muted"));
    return;
  }
  for (const auto& font : library) {
    auto* row = new QFrame();
    row->setObjectName(QStringLiteral("FontRow"));
    row->setProperty("role", "list-item");
    auto* line = new QHBoxLayout(row);
    line->setContentsMargins(10, 6, 6, 6);
    line->setSpacing(8);
    auto* words = new QVBoxLayout();
    words->setSpacing(0);
    auto* name = MakeLabel(ToQ(font.family), nullptr, row);
    name->setFont(SampleFont(ToQ(font.sample), ToQ(font.family)));
    words->addWidget(name);
    auto* faces = new ClampedLabel(ToQ(font.faces), 1, row);
    faces->setProperty("role", "muted");
    words->addWidget(faces);
    line->addLayout(words, 1);
    line->addWidget(MakeLabel(ToQ(font.source), "badge-secondary", row));
    if (font.licensed) {
      line->addWidget(MakeLabel(QStringLiteral("Licence included"), "badge", row));
    }
    const QString id = ToQ(font.id);
    const QString family = ToQ(font.family);
    auto* remove = MakeIconButton(QStringLiteral("trash-2"),
                                  QStringLiteral("Remove ") + family, row);
    connect(remove, &QToolButton::clicked, this, [this, id, family] {
      const bool yes = Confirm(
          window(), QStringLiteral("Remove %1?").arg(family),
          QStringLiteral("It's removed from your library. Projects that "
                         "already use it keep their own copy."),
          QStringLiteral("Remove"), "destructive");
      if (!yes) return;
      try {
        App::Get().session().remove_font(ToRust(id));
      } catch (const rust::Error& error) {
        Notice(QStringLiteral("Couldn't remove %1. %2")
                   .arg(family, QString::fromUtf8(error.what())));
      }
      Reload();
    });
    line->addWidget(remove);
    rows_->addWidget(row);
  }
}

void FontsCard::Review() {
  const bridge::FontReviewView review = App::Get().session().font_review();
  if (!review.shown || !review_.isNull()) return;
  auto* dialog = new QDialog(window());
  review_ = dialog;
  dialog->setObjectName(QStringLiteral("FontReviewDialog"));
  dialog->setWindowTitle(QStringLiteral("Add fonts"));
  dialog->setAttribute(Qt::WA_DeleteOnClose);
  dialog->setMinimumWidth(kDialogWidth);
  auto* layout = new QVBoxLayout(dialog);
  layout->setContentsMargins(24, 22, 24, 20);
  layout->setSpacing(10);
  layout->addWidget(MakeLabel(QStringLiteral("Add fonts"), "title", dialog));
  auto* summary = MakeLabel(ToQ(review.summary), "muted", dialog);
  summary->setWordWrap(true);
  layout->addWidget(summary);
  auto* scroll = new QScrollArea(dialog);
  scroll->setWidgetResizable(true);
  scroll->setFrameShape(QFrame::NoFrame);
  scroll->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
  scroll->setMaximumHeight(288);
  auto* list = new QWidget(scroll);
  auto* column = new QVBoxLayout(list);
  column->setContentsMargins(0, 0, 0, 0);
  column->setSpacing(8);
  auto* add = MakeButton(QString(), "primary", QString(), dialog);
  auto checks = std::make_shared<std::vector<QCheckBox*>>();
  const auto count = [add, checks] {
    int chosen = 0;
    for (QCheckBox* check : *checks) chosen += check->isChecked() ? 1 : 0;
    add->setText(chosen == 1 ? QStringLiteral("Add 1 font")
                             : QStringLiteral("Add %1 fonts").arg(chosen));
    add->setEnabled(chosen > 0);
  };
  for (const auto& family : review.families) {
    auto* row = new QWidget(list);
    auto* line = new QHBoxLayout(row);
    line->setContentsMargins(0, 0, 0, 0);
    auto* check = new QCheckBox(row);
    check->setChecked(true);
    check->setAccessibleName(ToQ(family.family));
    check->setProperty("family", ToQ(family.family));
    checks->push_back(check);
    QObject::connect(check, &QCheckBox::toggled, dialog, count);
    line->addWidget(check, 0, Qt::AlignTop);
    auto* words = new QVBoxLayout();
    words->setSpacing(0);
    words->addWidget(MakeLabel(ToQ(family.family), "title-small", row));
    auto* faces = MakeLabel(ToQ(family.faces), "muted", row);
    faces->setWordWrap(true);
    words->addWidget(faces);
    line->addLayout(words, 1);
    column->addWidget(row);
  }
  column->addStretch(1);
  scroll->setWidget(list);
  layout->addWidget(scroll);
  auto* note = MakeLabel(ToQ(review.note), "muted", dialog);
  note->setWordWrap(true);
  layout->addWidget(note);
  auto* buttons = new QHBoxLayout();
  buttons->addStretch(1);
  auto* cancel = MakeButton(QStringLiteral("Cancel"), "outline", QString(), dialog);
  buttons->addWidget(cancel);
  buttons->addWidget(add);
  layout->addLayout(buttons);
  count();
  QObject::connect(cancel, &QPushButton::clicked, dialog, &QDialog::reject);
  QObject::connect(dialog, &QDialog::rejected, dialog,
                   [] { App::Get().session().drop_review(); });
  QObject::connect(add, &QPushButton::clicked, dialog, [dialog, checks] {
    rust::Vec<rust::String> chosen;
    for (QCheckBox* check : *checks) {
      if (check->isChecked()) {
        chosen.push_back(ToRust(check->property("family").toString()));
      }
    }
    try {
      Notice(ToQ(App::Get().session().add_reviewed(std::move(chosen))));
    } catch (const rust::Error& error) {
      Notice(QString::fromUtf8(error.what()));
    }
    dialog->accept();
  });
  dialog->open();
}

void FontsCard::ShowInstalled() {
  auto families = std::make_shared<rust::Vec<bridge::InstalledFontView>>(
      App::Get().session().installed_fonts());
  auto* dialog = new SearchListDialog(window(), QStringLiteral("InstalledFonts"),
                                      QStringLiteral("Add an installed font"),
                                      QStringLiteral("Search installed fonts…"));
  dialog->SetDescription(
      QStringLiteral("%1 font families are installed on this computer. Pick "
                     "one to copy into your library.")
          .arg(families->size()));
  const auto fill = [dialog, families] {
    const QString query = dialog->query();
    dialog->Fill(
        static_cast<int>(families->size()),
        [families, query](int i) {
          return query.isEmpty() ||
                 ToQ(families->at(static_cast<std::size_t>(i)).family)
                     .contains(query, Qt::CaseInsensitive);
        },
        [dialog, families](int i, QWidget* parent) -> QWidget* {
          const auto& font = families->at(static_cast<std::size_t>(i));
          const QString family = ToQ(font.family);
          auto* row = MakeButton(QString(), "row", QString(), parent);
          row->setAccessibleName(family);
          auto* line = new QHBoxLayout(row);
          line->setContentsMargins(8, 4, 8, 4);
          auto* name = MakeLabel(family, nullptr, row);
          name->setFont(SampleFont(QString(), family));
          line->addWidget(name, 1);
          line->addWidget(MakeLabel(ToQ(font.styles), "muted", row));
          row->setMinimumHeight(row->sizeHint().height());
          QObject::connect(row, &QPushButton::clicked, dialog, [dialog, family] {
            App::Get().session().inspect_installed(ToRust(family));
            dialog->accept();
          });
          return row;
        });
  };
  connect(dialog, &SearchListDialog::QueryChanged, dialog, fill);
  fill();
  dialog->open();
}

void FontsCard::AskOnline() {
  if (!OnlineAllowed()) {
    OnlineAllowed() = Confirm(
        window(), QStringLiteral("Go online to get fonts?"),
        QStringLiteral("Opsis will connect to the internet to show the list of "
                       "free fonts, and to download the ones you pick. Each "
                       "font is saved to your library and works offline "
                       "afterwards. Nothing about you or your projects is "
                       "sent."),
        QStringLiteral("Connect"), "primary");
  }
  if (OnlineAllowed()) ShowOnline();
}

void FontsCard::ShowOnline() {
  if (!online_.isNull()) {
    online_->raise();
    return;
  }
  auto* dialog = new SearchListDialog(
      window(), QStringLiteral("OnlineFonts"), QStringLiteral("Get fonts online"),
      QStringLiteral("Search by name or style (serif, monospace…)"));
  online_ = dialog;
  auto* done = MakeButton(QStringLiteral("Done"), "outline", QString(), dialog);
  connect(done, &QPushButton::clicked, dialog, &QDialog::accept);
  dialog->footer()->addWidget(done, 0, Qt::AlignRight);
  const auto fill = [dialog] {
    const auto catalog =
        std::make_shared<bridge::FontCatalogView>(App::Get().session().font_catalog());
    const QString status = ToQ(catalog->status);
    dialog->SetDescription(
        status == QStringLiteral("ready")
            ? QStringLiteral("%1 free fonts you can use in your videos.")
                  .arg(catalog->entries.size())
        : status == QStringLiteral("error")
            ? QStringLiteral("The font list didn't load.")
            : QStringLiteral("Loading the font list…"));
    if (status == QStringLiteral("error")) {
      auto* box = new QWidget();
      auto* column = new QVBoxLayout(box);
      column->setContentsMargins(0, 0, 0, 0);
      auto* error = MakeLabel(ToQ(catalog->error), "error", box);
      error->setWordWrap(true);
      column->addWidget(error);
      auto* retry = MakeButton(QStringLiteral("Try again"), "outline", QString(), box);
      QObject::connect(retry, &QPushButton::clicked, box,
                       [] { App::Get().session().load_font_catalog(); });
      column->addWidget(retry, 0, Qt::AlignLeft);
      dialog->SetStatus(box);
    } else if (status == QStringLiteral("loading")) {
      dialog->SetStatus(MakeLabel(QStringLiteral("Loading…"), "muted"));
    } else {
      dialog->SetStatus(nullptr);
    }
    const QString query = dialog->query();
    const QString busy = ToQ(catalog->busy);
    dialog->Fill(
        static_cast<int>(catalog->entries.size()),
        [catalog, query](int i) {
          const auto& entry = catalog->entries.at(static_cast<std::size_t>(i));
          return query.isEmpty() ||
                 ToQ(entry.family).contains(query, Qt::CaseInsensitive) ||
                 ToQ(entry.detail).contains(query, Qt::CaseInsensitive);
        },
        [catalog, busy](int i, QWidget* parent) -> QWidget* {
          const auto& entry = catalog->entries.at(static_cast<std::size_t>(i));
          auto* row = new QWidget(parent);
          auto* line = new QHBoxLayout(row);
          line->setContentsMargins(8, 4, 8, 4);
          auto* words = new QVBoxLayout();
          words->setSpacing(0);
          words->addWidget(new ClampedLabel(ToQ(entry.family), 1, row));
          auto* detail = new ClampedLabel(ToQ(entry.detail), 1, row);
          detail->setProperty("role", "muted");
          words->addWidget(detail);
          line->addLayout(words, 1);
          const QString id = ToQ(entry.id);
          auto* download = MakeButton(
              entry.added ? QStringLiteral("Added") : QStringLiteral("Download"),
              entry.added ? "ghost" : "outline",
              entry.added || busy == id ? QString() : QStringLiteral("download"),
              row);
          download->setEnabled(!entry.added && busy.isEmpty());
          QObject::connect(download, &QPushButton::clicked, row, [id] {
            App::Get().session().download_font(ToRust(id));
          });
          line->addWidget(download);
          return row;
        });
  };
  connect(dialog, &SearchListDialog::QueryChanged, dialog, fill);
  connect(EventHub::Instance(), &EventHub::FontsChanged, dialog, fill);
  App::Get().session().load_font_catalog();
  fill();
  dialog->open();
}

}
