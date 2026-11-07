#include "mixer_panel.h"

#include <QApplication>
#include <QHBoxLayout>
#include <QJsonArray>
#include <QJsonDocument>
#include <QLabel>
#include <QLineEdit>
#include <QMenu>
#include <QMouseEvent>
#include <QPushButton>
#include <QScrollBar>
#include <QToolButton>
#include <QVBoxLayout>
#include <algorithm>
#include <cmath>
#include <functional>

#include "app.h"
#include "event_hub.h"
#include "form_hold.h"
#include "mixer_controls.h"
#include "opsis-qt/src/bridge.rs.h"
#include "playback_clock.h"
#include "top_bar.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr int kStripWidth = 104;
constexpr int kMasterWidth = 250;
constexpr int kSmallKnob = 24;
constexpr int kSmallFont = 10;
constexpr int kNameFont = 12;
constexpr double kQuietLufs = -70;

using Patch = std::function<void(const QJsonObject& patch)>;

QString Compact(const QJsonObject& object) {
  return QString::fromUtf8(
      QJsonDocument(object).toJson(QJsonDocument::Compact));
}

void Preview(const QString& target, const QString& kind, double current,
             double next, const QJsonObject& band = QJsonObject()) {
  App::Get().session().preview_mix(
      ToRust(target), ToRust(kind), current, next,
      ToRust(band.isEmpty() ? QString() : Compact(band)));
}

void PreviewBands(const QString& target, const QJsonObject& before,
                  const QJsonObject& after) {
  if (!after.value(QStringLiteral("on")).toBool()) return;
  const QJsonArray old_bands = before.value(QStringLiteral("bands")).toArray();
  const QJsonArray new_bands = after.value(QStringLiteral("bands")).toArray();
  for (int i = 0; i < new_bands.size(); ++i) {
    const QJsonObject band = new_bands.at(i).toObject();
    const QJsonObject old_band = old_bands.at(i).toObject();
    if (band == old_band) continue;
    Preview(target, QStringLiteral("eq%1").arg(i),
            old_band.value(QStringLiteral("gain")).toDouble(),
            band.value(QStringLiteral("gain")).toDouble(), band);
  }
}

QJsonObject ObjectOf(const rust::String& json) {
  return QJsonDocument::fromJson(ToQ(json).toUtf8()).object();
}

QLabel* Small(const QString& text, const char* role, QWidget* parent) {
  auto* label = MakeLabel(text, role, parent);
  QFont font = label->font();
  font.setPixelSize(kSmallFont);
  label->setFont(font);
  return label;
}

QFrame* Shell(QWidget* parent, int width, QVBoxLayout** column) {
  auto* shell = new QFrame(parent);
  shell->setObjectName(QStringLiteral("MixerStrip"));
  shell->setAttribute(Qt::WA_StyledBackground);
  shell->setFixedWidth(width);
  *column = new QVBoxLayout(shell);
  (*column)->setContentsMargins(6, 6, 6, 6);
  (*column)->setSpacing(4);
  return shell;
}

QPushButton* Toggle(const QString& text, const QString& tip, bool on,
                    QWidget* parent) {
  auto* button = new QPushButton(text, parent);
  button->setObjectName(QStringLiteral("StripToggle"));
  button->setCheckable(true);
  button->setChecked(on);
  button->setToolTip(tip);
  button->setAccessibleName(tip.section(QLatin1Char('.'), 0, 0));
  return button;
}

void AddTone(QVBoxLayout* column, const QJsonObject& book,
             const QJsonObject& settings, const QString& target, bool reduction,
             const Patch& patch, QWidget* parent) {
  const QJsonObject highpass =
      settings.value(QStringLiteral("highpass")).toObject();
  const bool low_cut = highpass.value(QStringLiteral("on")).toBool();
  const double cut =
      low_cut ? highpass.value(QStringLiteral("freq")).toDouble() : 0;
  const QJsonObject eq = settings.value(QStringLiteral("eq")).toObject();
  auto* eq_button = new QPushButton(parent);
  eq_button->setObjectName(QStringLiteral("EqButton"));
  eq_button->setAccessibleName(QStringLiteral("Equaliser"));
  eq_button->setToolTip(QStringLiteral("Equaliser"));
  auto* eq_column = new QVBoxLayout(eq_button);
  eq_column->setContentsMargins(0, 2, 0, 2);
  eq_column->setSpacing(1);
  eq_column->addWidget(
      new EqCurve(eq, cut, QSize(kStripWidth - 16, 20), eq_button), 0,
      Qt::AlignHCenter);
  eq_column->addWidget(Small(QStringLiteral("EQ"), "muted", eq_button), 0,
                       Qt::AlignHCenter);
  eq_button->setFixedHeight(40);
  QObject::connect(eq_button, &QPushButton::clicked, parent,
                   [eq_button, book, eq, cut, patch, target] {
                     OpenEqPopover(
                         eq_button, book, eq, cut,
                         [patch, eq, target](const QJsonObject& next) {
                           PreviewBands(target, eq, next);
                           patch(QJsonObject{{QStringLiteral("eq"), next}});
                         });
                   });
  column->addWidget(eq_button);
  auto* row = new QHBoxLayout();
  row->setSpacing(4);
  auto* cut_button =
      Toggle(QStringLiteral("Low cut"),
             QStringLiteral("Low cut. Cut rumble below the low-cut frequency"),
             low_cut, parent);
  QObject::connect(cut_button, &QPushButton::toggled, parent,
                   [highpass, patch](bool on) {
                     QJsonObject next = highpass;
                     next.insert(QStringLiteral("on"), on);
                     patch(QJsonObject{{QStringLiteral("highpass"), next}});
                   });
  row->addWidget(cut_button);
  const QJsonObject comp = settings.value(QStringLiteral("comp")).toObject();
  auto* comp_button = new QPushButton(QStringLiteral("Comp"), parent);
  comp_button->setObjectName(QStringLiteral("StripToggle"));
  comp_button->setAccessibleName(QStringLiteral("Compressor"));
  comp_button->setProperty("on", comp.value(QStringLiteral("on")).toBool());
  if (reduction && comp.value(QStringLiteral("on")).toBool()) {
    const bridge::MeterView level = App::Get().session().meter(
        ToRust(target), PlaybackClock::Instance()->frame());
    if (!std::isnan(level.reduction)) {
      comp_button->setText(QStringLiteral("Comp ") +
                           MixText(QStringLiteral("signed"), -level.reduction));
    }
  }
  QObject::connect(comp_button, &QPushButton::clicked, parent,
                   [comp_button, book, comp, patch] {
                     OpenCompPopover(
                         comp_button, book, comp,
                         [patch](const QJsonObject& next) {
                           patch(QJsonObject{{QStringLiteral("comp"), next}});
                         });
                   });
  row->addWidget(comp_button, 1);
  column->addLayout(row);
}

void AddLevel(QVBoxLayout* column, const QJsonObject& book,
              const QJsonObject& settings, const QString& label,
              const QString& target, bool reduction, bool mute_solo,
              const Patch& patch, QWidget* parent) {
  auto* meters = new QHBoxLayout();
  meters->setSpacing(8);
  meters->addStretch(1);
  const double fader = settings.value(QStringLiteral("fader")).toDouble();
  auto* reading =
      Small(MixText(QStringLiteral("db"), fader) + QStringLiteral(" dB"),
            nullptr, parent);
  meters->addWidget(new Fader(
      label, fader, book.value(QStringLiteral("minDb")).toDouble(-60),
      book.value(QStringLiteral("maxDb")).toDouble(12),
      [reading, patch, target, fader](double value) {
        reading->setText(MixText(QStringLiteral("db"), value) +
                         QStringLiteral(" dB"));
        Preview(target, QStringLiteral("fader"), fader, value);
        patch(QJsonObject{{QStringLiteral("fader"), value}});
      },
      parent));
  meters->addWidget(new Meter(target, reduction, parent));
  meters->addStretch(1);
  column->addLayout(meters, 1);
  column->addWidget(reading, 0, Qt::AlignHCenter);
  if (!mute_solo) return;
  auto* row = new QHBoxLayout();
  row->setSpacing(4);
  row->addStretch(1);
  auto* mute = Toggle(
      QStringLiteral("M"),
      QStringLiteral("Mute. Saved with the project and left out of exports."),
      settings.value(QStringLiteral("mute")).toBool(), parent);
  mute->setProperty("tone", "mute");
  auto* solo =
      Toggle(QStringLiteral("S"),
             QStringLiteral("Solo. While anything is soloed, only soloed "
                            "sounds play, in exports too."),
             settings.value(QStringLiteral("solo")).toBool(), parent);
  solo->setProperty("tone", "solo");
  QObject::connect(mute, &QPushButton::toggled, parent, [patch](bool on) {
    patch(QJsonObject{{QStringLiteral("mute"), on}});
  });
  QObject::connect(solo, &QPushButton::toggled, parent, [patch](bool on) {
    patch(QJsonObject{{QStringLiteral("solo"), on}});
  });
  row->addWidget(mute);
  row->addWidget(solo);
  row->addStretch(1);
  column->addLayout(row);
}

QWidget* HighpassKnob(const QJsonObject& book, const QJsonObject& settings,
                      const QString& target, const Patch& patch,
                      QWidget* parent) {
  const QJsonObject highpass =
      settings.value(QStringLiteral("highpass")).toObject();
  return MakeKnob(
      SpecFrom(book.value(QStringLiteral("highpass")).toObject(),
               QStringLiteral("Hz"), 1, QStringLiteral("hz"),
               QStringLiteral("Hz"), true),
      highpass.value(QStringLiteral("freq")).toDouble(), kSmallKnob,
      [highpass, patch, target](double freq) {
        QJsonObject next = highpass;
        next.insert(QStringLiteral("freq"), freq);
        Preview(target, QStringLiteral("highpass"),
                highpass.value(QStringLiteral("freq")).toDouble(), freq);
        patch(QJsonObject{{QStringLiteral("highpass"), next}});
      },
      !highpass.value(QStringLiteral("on")).toBool(), parent);
}

QToolButton* MenuButton(const QString& label, QWidget* parent) {
  auto* button = MakeIconButton(QStringLiteral("ellipsis"), label, parent, 14);
  button->setAccessibleName(label);
  return button;
}

void AddPresets(QMenu* menu, const QJsonObject& book, bool enabled,
                const Patch& apply) {
  for (const QJsonValue& entry :
       book.value(QStringLiteral("presets")).toArray()) {
    const QJsonObject preset = entry.toObject();
    QAction* action =
        menu->addAction(preset.value(QStringLiteral("label")).toString());
    action->setEnabled(enabled);
    const QJsonObject strip = preset.value(QStringLiteral("strip")).toObject();
    QObject::connect(action, &QAction::triggered, menu,
                     [apply, strip] { apply(strip); });
  }
}

QJsonObject StripSettings(const QJsonObject& strip) {
  QJsonObject copy;
  for (const char* key : {"trim", "highpass", "eq", "comp", "fader", "pan"}) {
    if (strip.contains(QLatin1String(key))) {
      copy.insert(QLatin1String(key), strip.value(QLatin1String(key)));
    }
  }
  return copy;
}

QJsonObject BusPart(const QJsonObject& settings) {
  QJsonObject part;
  for (const char* key : {"highpass", "eq", "comp", "fader"}) {
    if (settings.contains(QLatin1String(key))) {
      part.insert(QLatin1String(key), settings.value(QLatin1String(key)));
    }
  }
  return part;
}

QWidget* EmptyState(QWidget* parent) {
  auto* box = new QWidget(parent);
  auto* column = new QVBoxLayout(box);
  column->addStretch(1);
  auto* icon = new QLabel(box);
  icon->setPixmap(App::Get().Glyph(
      QStringLiteral("sliders-vertical"),
      App::Get().Token(QStringLiteral("muted-foreground")), 28));
  icon->setAlignment(Qt::AlignCenter);
  column->addWidget(icon);
  auto* title =
      MakeLabel(QStringLiteral("No sound in this film"), nullptr, box);
  title->setAlignment(Qt::AlignCenter);
  column->addWidget(title);
  auto* about = MakeLabel(
      QStringLiteral("Add music, a sound effect or a clip with sound, and its "
                     "channel appears here."),
      "muted", box);
  about->setAlignment(Qt::AlignCenter);
  about->setWordWrap(true);
  column->addWidget(about);
  column->addStretch(2);
  return box;
}

}

MixerPanel::MixerPanel(QWidget* parent)
    : QWidget(parent), stack_(new QStackedWidget(this)) {
  setObjectName(QStringLiteral("Panel"));
  setAttribute(Qt::WA_StyledBackground);
  auto* layout = new QVBoxLayout(this);
  layout->setContentsMargins(0, 0, 0, 0);
  layout->addWidget(stack_);
  stack_->addWidget(EmptyState(stack_));
  const auto refresh = [this] {
    if (!FormHold::Instance()->Held()) Refresh();
  };
  for (auto signal : {&EventHub::CompiledChanged, &EventHub::DocumentChanged,
                      &EventHub::SelectionChanged, &EventHub::ProjectChanged,
                      &EventHub::MediaChanged}) {
    connect(EventHub::Instance(), signal, this, refresh);
  }
  connect(FormHold::Instance(), &FormHold::Released, this, &MixerPanel::Refresh,
          Qt::QueuedConnection);
  connect(EventHub::Instance(), &EventHub::MediaChanged, this, [this] {
    if (body_ == nullptr) return;
    for (Meter* meter : body_->findChildren<Meter*>()) meter->update();
  });
  connect(PlaybackClock::Instance(), &PlaybackClock::Changed, this, [this] {
    if (body_ == nullptr) return;
    for (Meter* meter : body_->findChildren<Meter*>()) meter->update();
  });
  Rebuild();
}

void MixerPanel::Edit(const QJsonObject& edit) {
  try {
    App::Get().session().edit_mix(ToRust(Compact(edit)));
  } catch (const rust::Error& error) {
    App::Get().ShowError(QString::fromUtf8(error.what()));
  }
}

void MixerPanel::SelectStrip(const QString& id, const QStringList& order,
                             Qt::KeyboardModifiers modifiers,
                             const QString& source) {
  QStringList picked{id};
  const QString current = ToQ(App::Get().session().selected());
  if ((modifiers & Qt::ShiftModifier) != 0 && order.contains(anchor_)) {
    const auto a = order.indexOf(anchor_);
    const auto b = order.indexOf(id);
    picked = order.mid(std::min(a, b), std::abs(a - b) + 1);
  } else if ((modifiers & (Qt::ControlModifier | Qt::MetaModifier)) != 0) {
    picked = current.isEmpty() ? QStringList{} : QStringList{current};
    if (picked.contains(id)) {
      picked.removeAll(id);
    } else {
      picked.append(id);
    }
  }
  if ((modifiers & Qt::ShiftModifier) == 0) anchor_ = id;
  rust::Vec<rust::String> ids;
  for (const QString& one : picked) ids.push_back(ToRust(one));
  App::Get().session().select_pinned(std::move(ids), ToRust(source));
}

QWidget* MixerPanel::ChannelStrip(const bridge::MixerStripView& strip,
                                  const bridge::MixerView& view,
                                  QWidget* parent) {
  QVBoxLayout* column = nullptr;
  QFrame* shell = Shell(parent, kStripWidth, &column);
  shell->setProperty("dim", !strip.audible);
  const QString id = ToQ(strip.id);
  const QJsonObject settings = ObjectOf(strip.settings_json);
  const Patch patch = [this, id](const QJsonObject& change) {
    Edit(QJsonObject{{QStringLiteral("op"), QStringLiteral("channels")},
                     {QStringLiteral("ids"), QJsonArray{id}},
                     {QStringLiteral("patch"), change}});
  };
  QStringList order;
  for (const auto& channel : view.channels) order.append(ToQ(channel.id));
  const QString kind = ToQ(strip.kind);
  const QString icon =
      kind == QStringLiteral("music")
          ? QStringLiteral("music")
          : (kind == QStringLiteral("effect") ? QStringLiteral("audio-lines")
                                              : QStringLiteral("clapperboard"));
  auto* name = MakeButton(ToQ(strip.name), "ghost", icon, shell);
  name->setObjectName(QStringLiteral("StripName"));
  name->setProperty("selected", strip.selected);
  name->setToolTip(
      ToQ(strip.name) +
      QStringLiteral(". Ctrl-click or Shift-click to select several strips."));
  const QString source = ToQ(view.source_id);
  connect(name, &QPushButton::clicked, this, [this, id, order, source] {
    SelectStrip(id, order, QApplication::keyboardModifiers(), source);
  });
  column->addWidget(name);
  auto* bus_row = new QHBoxLayout();
  bus_row->setSpacing(2);
  auto* bus = new Combo(shell);
  bus->setAccessibleName(QStringLiteral("Send to bus"));
  for (const auto& one : view.buses) bus->addItem(ToQ(one.name), ToQ(one.id));
  bus->setCurrentIndex(std::max(0, bus->findData(ToQ(strip.bus))));
  connect(bus, &QComboBox::activated, this, [bus, patch](int) {
    patch(QJsonObject{{QStringLiteral("bus"), bus->currentData().toString()}});
  });
  bus_row->addWidget(bus, 1);
  QStringList group;
  if (strip.selected) {
    for (const auto& channel : view.channels) {
      if (channel.selected) group.append(ToQ(channel.id));
    }
  }
  if (group.isEmpty()) group.append(id);
  auto* menu_button = MenuButton(QStringLiteral("Strip settings"), shell);
  connect(
      menu_button, &QToolButton::clicked, this,
      [this, menu_button, group, settings] {
        const bool several = group.size() > 1;
        QMenu menu(menu_button);
        QJsonArray ids;
        for (const QString& one : group) ids.append(one);
        const Patch apply = [this, ids](const QJsonObject& change) {
          Edit(QJsonObject{{QStringLiteral("op"), QStringLiteral("channels")},
                           {QStringLiteral("ids"), ids},
                           {QStringLiteral("patch"), change}});
        };
        menu.addSection(several
                            ? QStringLiteral("Presets for %1 selected strips")
                                  .arg(group.size())
                            : QStringLiteral("Presets"));
        AddPresets(&menu, book_, true, apply);
        menu.addSeparator();
        QAction* copy = menu.addAction(QStringLiteral("Copy settings"));
        connect(copy, &QAction::triggered, this,
                [this, settings] { clipboard_ = StripSettings(settings); });
        QAction* paste = menu.addAction(
            several ? QStringLiteral("Paste onto %1 strips").arg(group.size())
                    : QStringLiteral("Paste settings"));
        paste->setEnabled(clipboard_.has_value());
        connect(paste, &QAction::triggered, this, [this, apply] {
          if (clipboard_) apply(*clipboard_);
        });
        QAction* reset = menu.addAction(
            several ? QStringLiteral("Reset %1 strips").arg(group.size())
                    : QStringLiteral("Reset"));
        connect(reset, &QAction::triggered, this, [this, ids] {
          Edit(QJsonObject{
              {QStringLiteral("op"), QStringLiteral("resetChannels")},
              {QStringLiteral("ids"), ids}});
        });
        menu.exec(menu_button->mapToGlobal(QPoint(0, menu_button->height())));
      });
  bus_row->addWidget(menu_button);
  column->addLayout(bus_row);
  auto* knobs = new QHBoxLayout();
  knobs->setSpacing(0);
  const QString target = QStringLiteral("ch:") + id;
  const double trim = settings.value(QStringLiteral("trim")).toDouble();
  const double pan = settings.value(QStringLiteral("pan")).toDouble();
  knobs->addWidget(MakeKnob(
      SpecFrom(book_.value(QStringLiteral("trim")).toObject(),
               QStringLiteral("Trim"), 0.5, QStringLiteral("db"),
               QStringLiteral("dB")),
      settings.value(QStringLiteral("trim")).toDouble(), kSmallKnob,
      [patch, target, trim](double v) {
        Preview(target, QStringLiteral("trim"), trim, v);
        patch(QJsonObject{{QStringLiteral("trim"), v}});
      },
      false, shell));
  knobs->addWidget(HighpassKnob(book_, settings, target, patch, shell));
  knobs->addWidget(MakeKnob(
      SpecFrom(book_.value(QStringLiteral("pan")).toObject(),
               QStringLiteral("Pan"), 0.01, QStringLiteral("pan"), QString()),
      settings.value(QStringLiteral("pan")).toDouble(), kSmallKnob,
      [patch, target, pan](double v) {
        Preview(target, QStringLiteral("pan"), pan, v);
        patch(QJsonObject{{QStringLiteral("pan"), v}});
      },
      false, shell));
  column->addLayout(knobs);
  AddTone(column, book_, settings, target, true, patch, shell);
  AddLevel(column, book_, settings, QStringLiteral("Fader"), target, true, true,
           patch, shell);
  return shell;
}

QWidget* MixerPanel::BusStrip(const bridge::MixerStripView& strip,
                              const bridge::MixerView& view, QWidget* parent) {
  QVBoxLayout* column = nullptr;
  QFrame* shell = Shell(parent, kStripWidth, &column);
  shell->setProperty("bus", true);
  const QString id = ToQ(strip.id);
  const QJsonObject settings = ObjectOf(strip.settings_json);
  const Patch patch = [this, id](const QJsonObject& change) {
    Edit(QJsonObject{{QStringLiteral("op"), QStringLiteral("bus")},
                     {QStringLiteral("id"), id},
                     {QStringLiteral("patch"), change}});
  };
  auto* name_row = new QHBoxLayout();
  name_row->setSpacing(2);
  auto* name = new QLineEdit(ToQ(strip.name), shell);
  name->setObjectName(QStringLiteral("BusName"));
  name->setAccessibleName(QStringLiteral("Bus name"));
  name->setToolTip(QStringLiteral("Bus name"));
  connect(name, &QLineEdit::editingFinished, this, [name, patch] {
    if (!name->isModified() || name->text().trimmed().isEmpty()) return;
    patch(QJsonObject{{QStringLiteral("name"), name->text().trimmed()}});
  });
  name_row->addWidget(name, 1);
  if (!strip.built_in) {
    auto* remove = MakeIconButton(QStringLiteral("trash-2"),
                                  QStringLiteral("Remove bus"), shell, 14);
    connect(remove, &QToolButton::clicked, this, [this, id] {
      Edit(QJsonObject{{QStringLiteral("op"), QStringLiteral("removeBus")},
                       {QStringLiteral("id"), id}});
    });
    name_row->addWidget(remove);
  }
  column->addLayout(name_row);
  QJsonArray members;
  for (const auto& channel : view.channels) {
    if (channel.bus == strip.id) members.append(ToQ(channel.id));
  }
  const bool folded = folded_.contains(id);
  auto* fold_row = new QHBoxLayout();
  fold_row->setSpacing(2);
  auto* fold = MakeButton(
      members.size() == 1 ? QStringLiteral("1 sound")
                          : QStringLiteral("%1 sounds").arg(members.size()),
      "ghost",
      folded ? QStringLiteral("chevron-right") : QStringLiteral("chevron-left"),
      shell);
  fold->setObjectName(QStringLiteral("FoldButton"));
  fold->setEnabled(!members.isEmpty());
  fold->setToolTip(folded ? QStringLiteral("Show the sounds on this bus")
                          : QStringLiteral("Hide the sounds on this bus"));
  connect(fold, &QPushButton::clicked, this, [this, id] {
    if (!folded_.remove(id)) folded_.insert(id);
    Rebuild();
  });
  fold_row->addWidget(fold, 1);
  auto* menu_button = MenuButton(QStringLiteral("Bus settings"), shell);
  connect(
      menu_button, &QToolButton::clicked, this,
      [this, menu_button, id, members, patch] {
        QMenu menu(menu_button);
        const Patch to_bus = [patch](const QJsonObject& change) {
          patch(BusPart(change));
        };
        const Patch to_sounds = [this, members](const QJsonObject& change) {
          Edit(QJsonObject{{QStringLiteral("op"), QStringLiteral("channels")},
                           {QStringLiteral("ids"), members},
                           {QStringLiteral("patch"), change}});
        };
        menu.addSection(QStringLiteral("This bus"));
        AddPresets(&menu, book_, true, to_bus);
        QAction* paste = menu.addAction(QStringLiteral("Paste settings"));
        paste->setEnabled(clipboard_.has_value());
        connect(paste, &QAction::triggered, this, [this, to_bus] {
          if (clipboard_) to_bus(*clipboard_);
        });
        QAction* reset = menu.addAction(QStringLiteral("Reset"));
        connect(reset, &QAction::triggered, this, [this, id] {
          Edit(QJsonObject{{QStringLiteral("op"), QStringLiteral("resetBus")},
                           {QStringLiteral("id"), id}});
        });
        menu.addSeparator();
        const bool sounds = !members.isEmpty();
        menu.addSection(members.size() == 1
                            ? QStringLiteral("The sound on this bus")
                            : QStringLiteral("All %1 sounds on this bus")
                                  .arg(members.size()));
        AddPresets(&menu, book_, sounds, to_sounds);
        QAction* paste_sounds =
            menu.addAction(QStringLiteral("Paste settings"));
        paste_sounds->setEnabled(sounds && clipboard_.has_value());
        connect(paste_sounds, &QAction::triggered, this, [this, to_sounds] {
          if (clipboard_) to_sounds(*clipboard_);
        });
        QAction* reset_sounds = menu.addAction(QStringLiteral("Reset"));
        reset_sounds->setEnabled(sounds);
        connect(reset_sounds, &QAction::triggered, this, [this, members] {
          Edit(QJsonObject{
              {QStringLiteral("op"), QStringLiteral("resetChannels")},
              {QStringLiteral("ids"), members}});
        });
        menu.exec(menu_button->mapToGlobal(QPoint(0, menu_button->height())));
      });
  fold_row->addWidget(menu_button);
  column->addLayout(fold_row);
  const QJsonObject duck = settings.value(QStringLiteral("duck")).toObject();
  auto* duck_button = new QPushButton(duck.value(QStringLiteral("on")).toBool()
                                          ? QStringLiteral("Ducks")
                                          : QStringLiteral("No ducking"),
                                      shell);
  duck_button->setObjectName(QStringLiteral("StripToggle"));
  duck_button->setProperty("on", duck.value(QStringLiteral("on")).toBool());
  QJsonArray buses;
  for (const auto& one : view.buses) {
    buses.append(QJsonObject{{QStringLiteral("id"), ToQ(one.id)},
                             {QStringLiteral("name"), ToQ(one.name)}});
  }
  connect(duck_button, &QPushButton::clicked, this,
          [this, duck_button, id, buses, duck, patch] {
            OpenDuckPopover(
                duck_button, book_, id, buses, duck,
                [patch](const QJsonObject& next) {
                  patch(QJsonObject{{QStringLiteral("duck"), next}});
                });
          });
  column->addWidget(duck_button);
  const QString target = QStringLiteral("bus:") + id;
  column->addWidget(HighpassKnob(book_, settings, target, patch, shell), 0,
                    Qt::AlignHCenter);
  AddTone(column, book_, settings, target, false, patch, shell);
  AddLevel(column, book_, settings, ToQ(strip.name) + QStringLiteral(" fader"),
           target, false, true, patch, shell);
  return shell;
}

QWidget* MixerPanel::MasterStrip(const bridge::MixerView& view,
                                 QWidget* parent) {
  auto* shell = new QFrame(parent);
  shell->setObjectName(QStringLiteral("MasterStrip"));
  shell->setAttribute(Qt::WA_StyledBackground);
  shell->setFixedWidth(kMasterWidth);
  auto* row = new QHBoxLayout(shell);
  row->setContentsMargins(8, 6, 8, 6);
  row->setSpacing(8);
  auto* column = new QVBoxLayout();
  column->setSpacing(4);
  const QJsonObject settings = ObjectOf(view.master.settings_json);
  const Patch patch = [this](const QJsonObject& change) {
    Edit(QJsonObject{{QStringLiteral("op"), QStringLiteral("master")},
                     {QStringLiteral("patch"), change}});
  };
  auto* header = new QHBoxLayout();
  auto* title = MakeLabel(QStringLiteral("Master"), nullptr, shell);
  QFont bold = title->font();
  bold.setPixelSize(kNameFont);
  bold.setWeight(QFont::Medium);
  title->setFont(bold);
  header->addWidget(title, 1);
  auto* reset = MakeIconButton(QStringLiteral("ellipsis"),
                               QStringLiteral("Reset master"), shell, 14);
  const QJsonObject defaults =
      book_.value(QStringLiteral("defaultMaster")).toObject();
  connect(reset, &QToolButton::clicked, this,
          [patch, defaults] { patch(defaults); });
  header->addWidget(reset);
  column->addLayout(header);
  AddTone(column, book_, settings, QStringLiteral("master"), true, patch,
          shell);
  const QJsonObject limiter =
      settings.value(QStringLiteral("limiter")).toObject();
  auto* limiter_row = new QHBoxLayout();
  limiter_row->addWidget(Small(QStringLiteral("Limiter"), nullptr, shell));
  auto* limiter_on = new ToggleSwitch(shell);
  limiter_on->setAccessibleName(QStringLiteral("Limiter"));
  limiter_on->SetCheckedQuietly(limiter.value(QStringLiteral("on")).toBool());
  connect(limiter_on, &ToggleSwitch::toggled, this, [limiter, patch](bool on) {
    QJsonObject next = limiter;
    next.insert(QStringLiteral("on"), on);
    patch(QJsonObject{{QStringLiteral("limiter"), next}});
  });
  limiter_row->addWidget(limiter_on);
  limiter_row->addWidget(MakeKnob(
      SpecFrom(book_.value(QStringLiteral("limiterCeiling")).toObject(),
               QStringLiteral("Ceiling"), 0.1, QStringLiteral("db"),
               QStringLiteral("dB")),
      limiter.value(QStringLiteral("ceiling")).toDouble(), kSmallKnob,
      [limiter, patch](double ceiling) {
        QJsonObject next = limiter;
        next.insert(QStringLiteral("ceiling"), ceiling);
        patch(QJsonObject{{QStringLiteral("limiter"), next}});
      },
      !limiter.value(QStringLiteral("on")).toBool(), shell));
  column->addLayout(limiter_row);
  column->addWidget(Small(QStringLiteral("Loudness target"), "muted", shell));
  auto* loudness = new Combo(shell);
  loudness->setAccessibleName(QStringLiteral("Loudness target"));
  QString aim;
  const QString chosen = settings.value(QStringLiteral("loudness")).toString();
  for (const QJsonValue& entry :
       book_.value(QStringLiteral("loudness")).toArray()) {
    const QJsonObject choice = entry.toObject();
    const QJsonObject goal = choice.value(QStringLiteral("goal")).toObject();
    QString label = choice.value(QStringLiteral("label")).toString();
    if (!goal.isEmpty()) {
      const QString lufs =
          MixText(QStringLiteral("signed"),
                  goal.value(QStringLiteral("lufs")).toDouble());
      label += QStringLiteral(" (%1)").arg(lufs);
      if (choice.value(QStringLiteral("target")).toString() == chosen)
        aim = lufs;
    }
    loudness->addItem(label, choice.value(QStringLiteral("target")).toString());
  }
  loudness->setCurrentIndex(std::max(0, loudness->findData(chosen)));
  connect(loudness, &QComboBox::activated, this, [loudness, patch](int) {
    patch(QJsonObject{
        {QStringLiteral("loudness"), loudness->currentData().toString()}});
  });
  column->addWidget(loudness);
  const bridge::LoudnessView reading =
      App::Get().session().loudness_at(PlaybackClock::Instance()->frame());
  const auto readout = [&](const QString& label, double value,
                           const QString& unit, double floor) {
    auto* line = new QHBoxLayout();
    line->addWidget(Small(label, "muted", shell));
    line->addStretch(1);
    const QString text =
        std::isnan(value)
            ? QStringLiteral("—")
            : (value <= floor ? QStringLiteral("−∞")
                              : MixText(QStringLiteral("signed"), value)
                                    .remove(QLatin1Char('+')));
    line->addWidget(Small(text + QStringLiteral(" ") + unit, nullptr, shell));
    column->addLayout(line);
  };
  readout(QStringLiteral("Momentary"), reading.momentary,
          QStringLiteral("LUFS"), kQuietLufs);
  readout(QStringLiteral("Short-term"), reading.short_term,
          QStringLiteral("LUFS"), kQuietLufs);
  readout(QStringLiteral("Integrated"), reading.integrated,
          QStringLiteral("LUFS"), kQuietLufs);
  readout(QStringLiteral("True peak"), reading.true_peak,
          QStringLiteral("dBTP"), -1e9);
  readout(QStringLiteral("Range"), reading.range, QStringLiteral("LU"), -1e9);
  if (!aim.isEmpty()) {
    column->addWidget(
        Small(QStringLiteral("Aim: %1 LUFS").arg(aim), "muted", shell));
  }
  column->addStretch(1);
  row->addLayout(column, 1);
  auto* level = new QVBoxLayout();
  AddLevel(level, book_, settings, QStringLiteral("Master fader"),
           QStringLiteral("master"),
           settings.value(QStringLiteral("comp"))
               .toObject()
               .value(QStringLiteral("on"))
               .toBool(),
           false, patch, shell);
  row->addLayout(level);
  return shell;
}

QWidget* MixerPanel::Toolbar(const bridge::MixerView& view, QWidget* parent) {
  auto* bar = new QWidget(parent);
  bar->setObjectName(QStringLiteral("TimelineToolbar"));
  bar->setAttribute(Qt::WA_StyledBackground);
  auto* row = new QHBoxLayout(bar);
  row->setContentsMargins(8, 2, 8, 2);
  row->setSpacing(8);
  if (view.failed) {
    row->addWidget(MakeLabel(ToQ(view.error), "error", bar));
  } else if (view.stale) {
    row->addWidget(new Spinner(bar));
    row->addWidget(MakeLabel(QStringLiteral("Updating mix"), "muted", bar));
  } else {
    row->addWidget(MakeLabel(
        QStringLiteral("Levels show what the export plays."), "muted", bar));
  }
  if (view.solo) {
    auto* badge = MakeLabel(QStringLiteral("Solo on"), nullptr, bar);
    badge->setObjectName(QStringLiteral("SourceBadge"));
    row->addWidget(badge);
  }
  row->addStretch(1);
  auto* add = MakeButton(QStringLiteral("Add bus"), "ghost",
                         QStringLiteral("plus"), bar);
  connect(add, &QPushButton::clicked, this, [this] {
    Edit(QJsonObject{{QStringLiteral("op"), QStringLiteral("addBus")}});
  });
  row->addWidget(add);
  return bar;
}

MixerPanel::~MixerPanel() = default;

void MixerPanel::Rebuild() { Build(App::Get().session().mixer()); }

void MixerPanel::Refresh() {
  const bridge::MixerView view = App::Get().session().mixer();
  if (last_ != nullptr && *last_ == view) return;
  Build(view);
}

void MixerPanel::Build(const bridge::MixerView& view) {
  last_ = std::make_unique<bridge::MixerView>(view);
  builds_ += 1;
  book_ = ObjectOf(view.book_json);
  if (!view.shown) {
    stack_->setCurrentIndex(0);
    if (body_ != nullptr) {
      stack_->removeWidget(body_);
      body_->deleteLater();
      body_ = nullptr;
    }
    return;
  }
  auto* body = new QWidget(stack_);
  auto* column = new QVBoxLayout(body);
  column->setContentsMargins(0, 0, 0, 0);
  column->setSpacing(0);
  column->addWidget(Toolbar(view, body));
  auto* row = new QHBoxLayout();
  row->setSpacing(0);
  auto* scroll = new QScrollArea(body);
  scroll->setWidgetResizable(true);
  scroll->setFrameShape(QFrame::NoFrame);
  scroll->setVerticalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
  auto* strips = new QWidget(scroll);
  strips->setObjectName(QStringLiteral("MixerStrips"));
  auto* line = new QHBoxLayout(strips);
  line->setContentsMargins(0, 0, 0, 0);
  line->setSpacing(1);
  for (const auto& bus : view.buses) {
    if (!folded_.contains(ToQ(bus.id))) {
      for (const auto& channel : view.channels) {
        if (channel.bus == bus.id)
          line->addWidget(ChannelStrip(channel, view, strips));
      }
    }
    line->addWidget(BusStrip(bus, view, strips));
  }
  line->addStretch(1);
  scroll->setWidget(strips);
  row->addWidget(scroll, 1);
  row->addWidget(MasterStrip(view, body));
  column->addLayout(row, 1);
  const int scrolled =
      body_ != nullptr
          ? body_->findChild<QScrollArea*>()->horizontalScrollBar()->value()
          : 0;
  if (body_ != nullptr) {
    stack_->removeWidget(body_);
    body_->deleteLater();
  }
  body_ = body;
  stack_->addWidget(body);
  stack_->setCurrentWidget(body);
  scroll->horizontalScrollBar()->setValue(scrolled);
}

}
