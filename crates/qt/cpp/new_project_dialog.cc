#include "new_project_dialog.h"

#include <QFileDialog>
#include <QHBoxLayout>
#include <QVBoxLayout>

#include "app.h"
#include "opsis-qt/src/bridge.rs.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr int kWidth = 480;

QString DefaultName() { return QStringLiteral("My launch video"); }

QString DefaultTemplate() { return QStringLiteral("todomvc"); }

}

NewProjectDialog::NewProjectDialog(const QString& template_id, bool fixed,
                                   QWidget* parent)
    : QDialog(parent),
      name_(new QLineEdit(DefaultName(), this)),
      location_(
          new QLineEdit(ToQ(App::Get().session().project_location()), this)),
      folder_(MakeLabel(QString(), "muted", this)),
      problem_(MakeLabel(QString(), "error", this)),
      create_(MakeButton(QStringLiteral("Create"), "primary", QString(), this)),
      choices_(new QWidget(this)),
      chosen_(new QWidget(this)) {
  setObjectName(QStringLiteral("NewProjectDialog"));
  setWindowTitle(QStringLiteral("New project"));
  setMinimumWidth(kWidth);
  auto* layout = new QVBoxLayout(this);
  layout->setContentsMargins(24, 22, 24, 20);
  layout->setSpacing(16);
  layout->addWidget(MakeLabel(QStringLiteral("New project"), "title", this));
  layout->addWidget(MakeField(QStringLiteral("Name"), name_, QString(), this));

  auto* where = new QWidget(this);
  auto* line = new QHBoxLayout(where);
  line->setContentsMargins(0, 0, 0, 0);
  line->setSpacing(8);
  auto* choose =
      MakeButton(QStringLiteral("Choose…"), nullptr, QString(), where);
  line->addWidget(location_, 1);
  line->addWidget(choose);
  folder_->setWordWrap(true);
  auto* location_field =
      MakeField(QStringLiteral("Location"), where, QString(), this);
  location_field->layout()->addWidget(folder_);
  layout->addWidget(location_field);

  const QString start = template_id.isEmpty() ? DefaultTemplate() : template_id;
  auto* choice_layout = new QVBoxLayout(choices_);
  choice_layout->setContentsMargins(0, 0, 0, 0);
  choice_layout->setSpacing(4);
  auto* chosen_layout = new QHBoxLayout(chosen_);
  chosen_layout->setContentsMargins(0, 0, 0, 0);
  QString chosen_title;
  int index = 0;
  for (const auto& entry : App::Get().session().templates()) {
    const QString id = ToQ(entry.id);
    auto* option = new QPushButton(ToQ(entry.title), choices_);
    option->setProperty("variant", "option");
    option->setCheckable(true);
    option->setToolTip(ToQ(entry.description));
    option->setChecked(id == start);
    templates_.addButton(option, index++);
    choice_layout->addWidget(option);
    ids_.append(id);
    if (id == start) chosen_title = ToQ(entry.title);
  }
  auto* chosen_label = MakeLabel(chosen_title, nullptr, chosen_);
  auto* change =
      MakeButton(QStringLiteral("Change"), "ghost", QString(), chosen_);
  chosen_layout->addWidget(chosen_label, 1);
  chosen_layout->addWidget(change);
  auto* template_field = new QWidget(this);
  auto* template_layout = new QVBoxLayout(template_field);
  template_layout->setContentsMargins(0, 0, 0, 0);
  template_layout->setSpacing(6);
  template_layout->addWidget(
      MakeLabel(QStringLiteral("Template"), nullptr, template_field));
  template_layout->addWidget(chosen_);
  template_layout->addWidget(choices_);
  choices_->setVisible(!fixed);
  chosen_->setVisible(fixed);
  layout->addWidget(template_field);

  problem_->setWordWrap(true);
  problem_->hide();
  layout->addWidget(problem_);
  auto* buttons = new QHBoxLayout();
  buttons->addStretch(1);
  auto* cancel = MakeButton(QStringLiteral("Cancel"), nullptr, QString(), this);
  buttons->addWidget(cancel);
  buttons->addWidget(create_);
  layout->addLayout(buttons);
  create_->setDefault(true);

  connect(change, &QPushButton::clicked, this, [this] {
    chosen_->hide();
    choices_->show();
  });
  connect(choose, &QPushButton::clicked, this, [this] {
    const QString dir = QFileDialog::getExistingDirectory(
        this, QStringLiteral("Choose where to create the project"),
        location_->text());
    if (!dir.isEmpty()) location_->setText(dir);
  });
  connect(name_, &QLineEdit::textChanged, this, &NewProjectDialog::SyncFolder);
  connect(location_, &QLineEdit::textChanged, this,
          &NewProjectDialog::SyncFolder);
  connect(cancel, &QPushButton::clicked, this, &QDialog::reject);
  connect(create_, &QPushButton::clicked, this, &NewProjectDialog::Create);
  name_->selectAll();
  name_->setFocus();
  SyncFolder();
}

QString NewProjectDialog::ChosenTemplate() const {
  const int id = templates_.checkedId();
  return id < 0 ? DefaultTemplate() : ids_.value(id, DefaultTemplate());
}

void NewProjectDialog::SyncFolder() {
  const QString location = location_->text().trimmed();
  const bool ready = !location.isEmpty() && !name_->text().trimmed().isEmpty();
  create_->setEnabled(ready);
  folder_->setText(
      location.isEmpty()
          ? QStringLiteral("Choose where the project folder goes.")
          : QStringLiteral("Creates ") +
                ToQ(App::Get().session().project_folder(
                    ToRust(location), ToRust(name_->text().trimmed()))));
}

void NewProjectDialog::Create() {
  try {
    App::Get().session().create_project(ToRust(name_->text().trimmed()),
                                        ToRust(ChosenTemplate()),
                                        ToRust(location_->text().trimmed()));
    accept();
  } catch (const rust::Error& error) {
    problem_->setText(QStringLiteral("Couldn't create the project: ") +
                      QString::fromUtf8(error.what()));
    problem_->show();
    FitHeightToWidth(this);
  }
}

}
