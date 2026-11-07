#include "project_panel.h"

#include <QComboBox>
#include <QGridLayout>
#include <QHBoxLayout>
#include <QIntValidator>
#include <QJsonArray>
#include <QJsonDocument>
#include <QLabel>
#include <QLineEdit>
#include <QPushButton>
#include <QToolButton>
#include <QVBoxLayout>
#include <array>
#include <utility>
#include <vector>

#include "app.h"
#include "event_hub.h"
#include "font_picker.h"
#include "opsis-qt/src/bridge.rs.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr int kMaxSize = 16384;
constexpr std::array<int, 2> kFrameRates = {30, 60};
constexpr int kNameShare = 2;
constexpr int kValueShare = 3;

struct FontRoleRow {
  const char* role;
  const char* label;
  const char* empty;
};

constexpr std::array<FontRoleRow, 3> kFontRoles = {{
    {"main", "Text", nullptr},
    {"heading", "Headings", "Same as text"},
    {"code", "Code", nullptr},
}};

QWidget* Legend(const QString& title, const QString& about, QWidget* parent) {
  auto* box = new QWidget(parent);
  auto* layout = new QVBoxLayout(box);
  layout->setContentsMargins(0, 0, 0, 0);
  layout->setSpacing(4);
  auto* heading = MakeLabel(title, nullptr, box);
  QFont font = heading->font();
  font.setWeight(QFont::Medium);
  heading->setFont(font);
  layout->addWidget(heading);
  if (!about.isEmpty()) {
    auto* text = MakeLabel(about, "small", box);
    text->setWordWrap(true);
    layout->addWidget(text);
  }
  return box;
}

QLineEdit* CommitLine(const QString& text, QWidget* parent,
                      const std::function<void(const QString&)>& commit) {
  auto* line = new QLineEdit(text, parent);
  line->setCursorPosition(0);
  QObject::connect(line, &QLineEdit::editingFinished, line,
                   [line, text, commit] {
                     if (line->text() != text) commit(line->text());
                   });
  return line;
}

QString JsonString(const QString& text) {
  const QByteArray array =
      QJsonDocument(QJsonArray{text}).toJson(QJsonDocument::Compact);
  return QString::fromUtf8(array.mid(1, array.size() - 2));
}

QWidget* Row(QWidget* parent) {
  auto* row = new QWidget(parent);
  auto* layout = new QHBoxLayout(row);
  layout->setContentsMargins(0, 0, 0, 0);
  layout->setSpacing(8);
  return row;
}

QString FirstFreeName(const std::vector<QString>& taken) {
  for (int n = 1;; ++n) {
    const QString name = QStringLiteral("Variable %1").arg(n);
    if (std::find(taken.begin(), taken.end(), name) == taken.end()) {
      return name;
    }
  }
}

}

ProjectPanel::ProjectPanel(QWidget* parent) : QWidget(parent) {
  const bridge::ProjectView view = App::Get().session().project();
  auto* layout = new QVBoxLayout(this);
  layout->setContentsMargins(16, 16, 16, 16);
  layout->setSpacing(20);
  if (view.selected > 1) AddSelection(layout, view);
  auto* name = CommitLine(ToQ(view.name), this, [](const QString& text) {
    Edit({{QStringLiteral("op"), QStringLiteral("rename")},
          {QStringLiteral("name"), text}});
  });
  auto* top = new QVBoxLayout();
  top->setSpacing(14);
  top->addWidget(
      MakeField(QStringLiteral("Project name"), name, QString(), this));
  layout->addLayout(top);
  AddCanvas(top, view);
  AddFonts(layout, view);
  AddVariables(layout, view);
  AddSavedResults(layout, view);
  AddProblems(layout, view);
  layout->addStretch(1);
}

void ProjectPanel::Edit(const QJsonObject& edit) {
  EditJson(
      QString::fromUtf8(QJsonDocument(edit).toJson(QJsonDocument::Compact)));
}

void ProjectPanel::EditJson(const QString& json) {
  try {
    App::Get().session().project_edit(ToRust(json));
  } catch (const rust::Error& error) {
    App::Get().ShowError(QString::fromUtf8(error.what()));
  }
}

void ProjectPanel::SetSettings(const QJsonObject& patch) {
  Edit({{QStringLiteral("op"), QStringLiteral("setSettings")},
        {QStringLiteral("patch"), patch}});
}

void ProjectPanel::AddSelection(QVBoxLayout* layout,
                                const bridge::ProjectView& view) {
  auto* alert = new QFrame(this);
  alert->setProperty("role", "alert");
  auto* box = new QVBoxLayout(alert);
  box->setContentsMargins(14, 12, 14, 12);
  box->setSpacing(6);
  auto* title = MakeLabel(
      QStringLiteral("%1 nodes selected").arg(view.selected), nullptr, alert);
  QFont font = title->font();
  font.setWeight(QFont::Medium);
  title->setFont(font);
  box->addWidget(title);
  box->addWidget(MakeLabel(
      QStringLiteral("Group them to keep a long flow tidy."), "muted", alert));
  auto* group =
      MakeButton(QStringLiteral("Group selection"), nullptr, QString(), alert);
  connect(group, &QPushButton::clicked, this, [] {
    try {
      App::Get().session().menu_command(ToRust(QStringLiteral("edit:group")));
    } catch (const rust::Error& error) {
      App::Get().ShowError(QString::fromUtf8(error.what()));
    }
  });
  box->addWidget(group, 0, Qt::AlignLeft);
  layout->addWidget(alert);
}

void ProjectPanel::AddCanvas(QVBoxLayout* layout,
                             const bridge::ProjectView& view) {
  auto* shapes = new Segmented(this);
  std::vector<std::pair<QString, QString>> choices;
  QString current;
  for (const auto& shape : view.shapes) {
    const QString label = ToQ(shape.label);
    choices.emplace_back(label, label);
    if (shape.width == view.width && shape.height == view.height) {
      current = label;
    }
  }
  shapes->SetChoices(choices);
  shapes->SetOutline();
  shapes->SetValue(current);
  std::vector<std::pair<QString, std::pair<int, int>>> sizes;
  for (const auto& shape : view.shapes) {
    sizes.emplace_back(ToQ(shape.label),
                       std::make_pair(static_cast<int>(shape.width),
                                      static_cast<int>(shape.height)));
  }
  connect(shapes, &Segmented::Changed, this, [sizes](const QString& label) {
    for (const auto& [name, size] : sizes) {
      if (name != label) continue;
      SetSettings({{QStringLiteral("width"), size.first},
                   {QStringLiteral("height"), size.second}});
    }
  });
  auto* shape_row = Row(this);
  shape_row->layout()->addWidget(shapes);
  static_cast<QHBoxLayout*>(shape_row->layout())->addStretch(1);
  layout->addWidget(
      MakeField(QStringLiteral("Canvas"), shape_row, QString(), this));

  auto* sizes_row = Row(this);
  const auto size_field = [this](const QString& label, int value,
                                 const QString& key) {
    auto* line =
        CommitLine(QString::number(value), this, [key](const QString& text) {
          bool ok = false;
          const int size = text.toInt(&ok);
          if (ok && size > 0) {
            SetSettings({{key, size}});
          }
        });
    line->setValidator(new QIntValidator(1, kMaxSize, line));
    return MakeField(label, line, QString(), this);
  };
  sizes_row->layout()->addWidget(size_field(QStringLiteral("Width"),
                                            static_cast<int>(view.width),
                                            QStringLiteral("width")));
  sizes_row->layout()->addWidget(size_field(QStringLiteral("Height"),
                                            static_cast<int>(view.height),
                                            QStringLiteral("height")));
  layout->addWidget(sizes_row);

  auto* rates = new Segmented(this);
  std::vector<std::pair<QString, QString>> rate_choices;
  for (const int rate : kFrameRates) {
    rate_choices.emplace_back(QString::number(rate),
                              QStringLiteral("%1 fps").arg(rate));
  }
  rates->SetChoices(rate_choices);
  rates->SetOutline();
  rates->SetValue(QString::number(view.fps));
  connect(rates, &Segmented::Changed, this, [](const QString& value) {
    SetSettings({{QStringLiteral("fps"), value.toInt()}});
  });
  auto* rate_row = Row(this);
  rate_row->layout()->addWidget(rates);
  static_cast<QHBoxLayout*>(rate_row->layout())->addStretch(1);
  layout->addWidget(
      MakeField(QStringLiteral("Frame rate"), rate_row, QString(), this));

  auto* background =
      CommitLine(ToQ(view.background), this, [](const QString& text) {
        SetSettings({{QStringLiteral("background"), text.trimmed()}});
      });
  background->setPlaceholderText(ToQ(view.background_hint));
  layout->addWidget(
      MakeField(QStringLiteral("Background"), background, QString(), this));
}

void ProjectPanel::AddFonts(QVBoxLayout* layout,
                            const bridge::ProjectView& view) {
  auto* box = new QWidget(this);
  auto* column = new QVBoxLayout(box);
  column->setContentsMargins(0, 0, 0, 0);
  column->setSpacing(12);
  column->addWidget(Legend(
      QStringLiteral("Fonts"),
      QStringLiteral("Every text in the video follows these unless a node "
                     "picks its own font."),
      box));
  const std::array<QString, kFontRoles.size()> values = {
      ToQ(view.main_font), ToQ(view.heading_font), ToQ(view.code_font)};
  std::size_t index = 0;
  for (const auto& role : kFontRoles) {
    const QString name = QString::fromUtf8(role.role);
    auto* picker = new FontPicker(
        values.at(index++),
        role.empty != nullptr ? QString::fromUtf8(role.empty) : QString(),
        [name](const QString& family) {
          Edit({{QStringLiteral("op"), QStringLiteral("setFontRole")},
                {QStringLiteral("role"), name},
                {QStringLiteral("family"), family}});
        },
        box);
    column->addWidget(
        MakeField(QString::fromUtf8(role.label), picker, QString(), box));
  }
  layout->addWidget(box);
}

void ProjectPanel::AddVariables(QVBoxLayout* layout,
                                const bridge::ProjectView& view) {
  auto* box = new QWidget(this);
  auto* column = new QVBoxLayout(box);
  column->setContentsMargins(0, 0, 0, 0);
  column->setSpacing(8);
  column->addWidget(Legend(
      QStringLiteral("Variables"),
      QStringLiteral("Reuse values like your site's address. Insert them into "
                     "any text with Insert variable."),
      box));
  std::vector<std::pair<QString, QString>> entries;
  for (const auto& variable : view.variables) {
    entries.emplace_back(ToQ(variable.name), ToQ(variable.value));
  }
  const auto set_all =
      [](const std::vector<std::pair<QString, QString>>& list) {
        QStringList pairs;
        for (const auto& [key, value] : list) {
          pairs.append(JsonString(key) + QLatin1Char(':') + JsonString(value));
        }
        EditJson(QStringLiteral("{\"op\":\"setVariables\",\"variables\":{") +
                 pairs.join(QLatin1Char(',')) + QStringLiteral("}}"));
      };
  for (std::size_t i = 0; i < entries.size(); ++i) {
    const QString key = entries.at(i).first;
    auto* row = Row(box);
    auto* name = CommitLine(key, row, [entries, key](const QString& raw) {
      QString to = raw;
      to.remove(QLatin1Char('{')).remove(QLatin1Char('}'));
      to = to.trimmed();
      if (to.isEmpty() || to == key) return;
      for (const auto& entry : entries) {
        if (entry.first == to) {
          App::Get().ShowError(
              QStringLiteral("There's already a variable called “%1”.")
                  .arg(to));
          return;
        }
      }
      Edit({{QStringLiteral("op"), QStringLiteral("renameVariable")},
            {QStringLiteral("from"), key},
            {QStringLiteral("to"), to}});
    });
    name->setToolTip(QStringLiteral("Variable name"));
    auto* value = CommitLine(entries.at(i).second, row,
                             [entries, i, set_all](const QString& text) {
                               auto next = entries;
                               next.at(i).second = text;
                               set_all(next);
                             });
    value->setToolTip(QStringLiteral("Variable value"));
    auto* remove = MakeIconButton(QStringLiteral("trash-2"),
                                  QStringLiteral("Remove variable"), row);
    connect(remove, &QToolButton::clicked, this, [entries, i, set_all] {
      auto next = entries;
      next.erase(next.begin() + static_cast<std::ptrdiff_t>(i));
      set_all(next);
    });
    auto* line = static_cast<QHBoxLayout*>(row->layout());
    line->addWidget(name, kNameShare);
    line->addWidget(value, kValueShare);
    line->addWidget(remove);
    column->addWidget(row);
  }
  auto* add = MakeButton(QStringLiteral("Add variable"), nullptr,
                         QStringLiteral("plus"), box);
  connect(add, &QPushButton::clicked, this, [entries, set_all] {
    std::vector<QString> taken;
    for (const auto& entry : entries) taken.push_back(entry.first);
    auto next = entries;
    next.emplace_back(FirstFreeName(taken), QString());
    set_all(next);
  });
  column->addWidget(add);
  layout->addWidget(box);
}

void ProjectPanel::AddSavedResults(QVBoxLayout* layout,
                                   const bridge::ProjectView& view) {
  auto* box = new QWidget(this);
  auto* column = new QVBoxLayout(box);
  column->setContentsMargins(0, 0, 0, 0);
  column->setSpacing(8);
  column->addWidget(Legend(
      QStringLiteral("Saved results"),
      QStringLiteral("Opsis keeps the result of every node so unchanged parts "
                     "don't run again. Clear them to record and export "
                     "everything again on the next run."),
      box));
  auto* clear = MakeButton(QStringLiteral("Clear saved results"), nullptr,
                           QStringLiteral("trash-2"), box);
  clear->setEnabled(!view.running);
  connect(clear, &QPushButton::clicked, this, [] {
    try {
      App::Get().session().clear_saved_results();
      emit EventHub::Instance() -> Notice(
          QStringLiteral(
              "Cleared. The next run records and exports everything again."),
          QString());
    } catch (const rust::Error& error) {
      App::Get().ShowError(
          QStringLiteral("Couldn't clear the saved results: ") +
          QString::fromUtf8(error.what()));
    }
  });
  column->addWidget(clear);
  layout->addWidget(box);
}

void ProjectPanel::AddProblems(QVBoxLayout* layout,
                               const bridge::ProjectView& view) {
  if (view.issues.empty()) return;
  auto* box = new QWidget(this);
  auto* column = new QVBoxLayout(box);
  column->setContentsMargins(0, 0, 0, 0);
  column->setSpacing(4);
  column->addWidget(Legend(QStringLiteral("Problems"), QString(), box));
  for (const auto& issue : view.issues) {
    auto* line =
        MakeLabel(ToQ(issue.message), issue.error ? "error" : "small", box);
    line->setWordWrap(true);
    column->addWidget(line);
  }
  layout->addWidget(box);
}

}
