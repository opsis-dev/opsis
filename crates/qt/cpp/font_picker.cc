#include "font_picker.h"

#include <QHBoxLayout>
#include <QHideEvent>
#include <QLabel>
#include <QLineEdit>
#include <QListWidget>
#include <QVBoxLayout>
#include <utility>

#include "app.h"
#include "event_hub.h"
#include "fonts_page.h"
#include "form_hold.h"
#include "opsis-qt/src/bridge.rs.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr int kPopoverWidth = 288;
constexpr int kListHeight = 320;
constexpr int kKindRole = Qt::UserRole;
constexpr int kFamilyRole = Qt::UserRole + 1;
constexpr int kIdRole = Qt::UserRole + 2;

enum class Row : int { kHeader = 0, kEmpty, kFont, kLibrary, kManage };

void AddGroup(QListWidget* list, const QString& heading,
              const rust::Vec<bridge::FontPickView>& fonts, Row kind) {
  if (fonts.empty()) return;
  auto* header = new QListWidgetItem(heading, list);
  header->setFlags(Qt::NoItemFlags);
  header->setData(kKindRole, static_cast<int>(Row::kHeader));
  QFont small = header->font();
  small.setPixelSize(11);
  small.setWeight(QFont::Medium);
  header->setFont(small);
  header->setForeground(App::Get().Token(QStringLiteral("muted-foreground")));
  const QColor ink = App::Get().Token(QStringLiteral("foreground"));
  for (const auto& font : fonts) {
    const QString family = ToQ(font.family);
    auto* item = new QListWidgetItem(family, list);
    item->setData(kKindRole, static_cast<int>(kind));
    item->setData(kFamilyRole, family);
    item->setData(kIdRole, ToQ(font.id));
    item->setFont(SampleFont(ToQ(font.sample), family));
    if (font.selected)
      item->setIcon(App::Get().Icon(QStringLiteral("check"), ink, 14));
  }
}

}

FontPopover::FontPopover(QWidget* anchor, const QString& value,
                         const QString& empty_label, OnFontPick on_pick)
    : QFrame(anchor->window(), Qt::Popup),
      search_(new QLineEdit(this)),
      list_(new QListWidget(this)),
      nothing_(MakeLabel(QStringLiteral("No font found."), "muted", this)),
      on_pick_(std::move(on_pick)) {
  setObjectName(QStringLiteral("Popover"));
  setAttribute(Qt::WA_DeleteOnClose);
  setAttribute(Qt::WA_StyledBackground);
  setFixedWidth(kPopoverWidth);
  FormHold::Instance()->Begin();
  auto* layout = new QVBoxLayout(this);
  layout->setContentsMargins(6, 6, 6, 6);
  layout->setSpacing(6);
  search_->setPlaceholderText(QStringLiteral("Search fonts…"));
  search_->setAccessibleName(QStringLiteral("Search fonts"));
  layout->addWidget(search_);
  list_->setObjectName(QStringLiteral("FontList"));
  list_->setFixedHeight(kListHeight);
  list_->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
  list_->setIconSize(QSize(14, 14));
  list_->setLayoutDirection(Qt::LeftToRight);
  layout->addWidget(list_);
  nothing_->setAlignment(Qt::AlignCenter);
  nothing_->hide();
  layout->addWidget(nothing_);
  const QColor ink = App::Get().Token(QStringLiteral("foreground"));
  if (!empty_label.isEmpty()) {
    auto* empty = new QListWidgetItem(empty_label, list_);
    empty->setData(kKindRole, static_cast<int>(Row::kEmpty));
    if (value.isEmpty())
      empty->setIcon(App::Get().Icon(QStringLiteral("check"), ink, 14));
  }
  const bridge::FontChoicesView choices =
      App::Get().session().font_choices(ToRust(value));
  AddGroup(list_, QStringLiteral("In this project"), choices.project,
           Row::kFont);
  AddGroup(list_, QStringLiteral("Built in"), choices.built_in, Row::kFont);
  AddGroup(list_, QStringLiteral("Your library"), choices.library,
           Row::kLibrary);
  auto* manage = new QListWidgetItem(
      App::Get().Icon(QStringLiteral("settings-2"), ink, 14),
      QStringLiteral("Add or manage fonts…"), list_);
  manage->setData(kKindRole, static_cast<int>(Row::kManage));
  connect(search_, &QLineEdit::textChanged, this, &FontPopover::Filter);
  connect(list_, &QListWidget::itemClicked, this,
          [this](QListWidgetItem* item) { Choose(list_->row(item)); });
  connect(search_, &QLineEdit::returnPressed, this, [this] {
    for (int i = 0; i < list_->count(); ++i) {
      QListWidgetItem* item = list_->item(i);
      if (!item->isHidden() &&
          item->data(kKindRole).toInt() != static_cast<int>(Row::kHeader)) {
        Choose(i);
        return;
      }
    }
  });
  adjustSize();
  QPoint at = anchor->mapToGlobal(QPoint(0, anchor->height() + 4));
  move(at);
  show();
  search_->setFocus();
}

void FontPopover::Filter(const QString& query) {
  const QString wanted = query.trimmed();
  int shown = 0;
  QListWidgetItem* header = nullptr;
  bool header_has = false;
  const auto close_header = [&] {
    if (header != nullptr) header->setHidden(!header_has);
  };
  for (int i = 0; i < list_->count(); ++i) {
    QListWidgetItem* item = list_->item(i);
    const auto kind = static_cast<Row>(item->data(kKindRole).toInt());
    if (kind == Row::kHeader) {
      close_header();
      header = item;
      header_has = false;
      continue;
    }
    const bool match =
        wanted.isEmpty() || item->text().contains(wanted, Qt::CaseInsensitive);
    item->setHidden(!match);
    if (match && (kind == Row::kFont || kind == Row::kLibrary)) {
      header_has = true;
    }
    if (match && kind != Row::kManage) shown += 1;
  }
  close_header();
  nothing_->setVisible(shown == 0);
}

void FontPopover::Choose(int row) {
  QListWidgetItem* item = list_->item(row);
  if (item == nullptr) return;
  const auto kind = static_cast<Row>(item->data(kKindRole).toInt());
  if (kind == Row::kHeader) return;
  OnFontPick pick = on_pick_;
  const QString family = item->data(kFamilyRole).toString();
  const QString id = item->data(kIdRole).toString();
  close();
  switch (kind) {
    case Row::kManage:
      emit EventHub::Instance() -> SettingsAsked(QStringLiteral("fonts"));
      return;
    case Row::kEmpty:
      pick(QString());
      return;
    case Row::kLibrary:
      try {
        pick(ToQ(App::Get().session().use_library_font(ToRust(id))));
      } catch (const rust::Error& error) {
        emit EventHub::Instance()
            -> Notice(QStringLiteral("Couldn't use %1").arg(family),
                      QString::fromUtf8(error.what()));
      }
      return;
    default:
      pick(family);
      return;
  }
}

void FontPopover::hideEvent(QHideEvent* event) {
  QFrame::hideEvent(event);
  if (ended_) return;
  ended_ = true;
  FormHold::Instance()->End();
}

FontPicker::FontPicker(QString value, QString empty_label, OnFontPick on_pick,
                       QWidget* parent)
    : QPushButton(parent),
      value_(std::move(value)),
      empty_label_(std::move(empty_label)),
      on_pick_(std::move(on_pick)) {
  setObjectName(QStringLiteral("FontPicker"));
  setProperty("variant", "outline");
  setAccessibleName(value_.isEmpty() ? empty_label_ : value_);
  auto* row = new QHBoxLayout(this);
  row->setContentsMargins(10, 0, 8, 0);
  row->setSpacing(6);
  auto* name = MakeLabel(value_.isEmpty() ? empty_label_ : value_,
                         value_.isEmpty() ? "muted" : nullptr, this);
  name->setAttribute(Qt::WA_TransparentForMouseEvents);
  if (!value_.isEmpty()) {
    const bridge::FontChoicesView choices =
        App::Get().session().font_choices(ToRust(value_));
    QFont sample = SampleFont(ToQ(choices.current_sample), value_);
    sample.setPixelSize(13);
    name->setFont(sample);
  }
  row->addWidget(name, 1);
  auto* chevrons = new QLabel(this);
  chevrons->setPixmap(
      App::Get()
          .Icon(QStringLiteral("chevrons-up-down"),
                App::Get().Token(QStringLiteral("muted-foreground")), 14)
          .pixmap(14, 14));
  chevrons->setAttribute(Qt::WA_TransparentForMouseEvents);
  row->addWidget(chevrons);
  setMinimumHeight(32);
  setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Fixed);
  connect(this, &QPushButton::clicked, this,
          [this] { new FontPopover(this, value_, empty_label_, on_pick_); });
}

}
