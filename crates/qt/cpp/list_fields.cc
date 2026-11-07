#include "list_fields.h"

#include <QDoubleValidator>
#include <QHBoxLayout>
#include <QJsonDocument>
#include <QLineEdit>
#include <QLocale>
#include <QMouseEvent>
#include <QPushButton>
#include <QToolButton>
#include <cstdint>
#include <memory>

#include "app.h"
#include "color_picker.h"
#include "opsis-qt/src/bridge.rs.h"
#include "var_text.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr int kGripSize = 14;
constexpr int kSwatch = 28;
constexpr int kCollapseFrom = 4;
constexpr const char* kRowProperty = "opsisListRow";

QJsonArray ArrayOf(const bridge::FieldView& field) {
  return QJsonDocument::fromJson(ToQ(field.value_json).toUtf8()).array();
}

QJsonValue ItemOf(const bridge::FieldView& field) {
  const QByteArray wrapped =
      QByteArray("[") + ToQ(field.new_item_json).toUtf8() + "]";
  return QJsonDocument::fromJson(wrapped).array().at(0);
}

QString ItemName(const bridge::FieldView& field, int index) {
  return QStringLiteral("%1 %2").arg(ToQ(field.label).toLower()).arg(index + 1);
}

QWidget* Shell(const bridge::FieldView& field, QVBoxLayout** rows) {
  auto* box = new QWidget();
  auto* column = new QVBoxLayout(box);
  column->setContentsMargins(0, 0, 0, 0);
  column->setSpacing(6);
  *rows = column;
  Q_UNUSED(field)
  return box;
}

QToolButton* RemoveButton(const QString& name, bool enabled, QWidget* parent) {
  auto* remove = MakeIconButton(QStringLiteral("x"),
                                QStringLiteral("Remove %1").arg(name), parent);
  remove->setEnabled(enabled);
  return remove;
}

QPushButton* AddButton(const QString& text, bool enabled, QWidget* parent) {
  auto* add = MakeButton(text, nullptr, QStringLiteral("plus"), parent);
  add->setEnabled(enabled);
  return add;
}

QJsonArray Moved(QJsonArray array, int from, int to) {
  if (from < 0 || from >= array.size() || to < 0 || to >= array.size() ||
      from == to) {
    return array;
  }
  const QJsonValue moving = array.takeAt(from);
  array.insert(to, moving);
  return array;
}

QWidget* Row(QWidget* parent, int index) {
  auto* row = new QWidget(parent);
  row->setProperty(kRowProperty, index);
  auto* layout = new QHBoxLayout(row);
  layout->setContentsMargins(0, 0, 0, 0);
  layout->setSpacing(4);
  return row;
}

}

QString JsonText(const QJsonArray& array) {
  return QString::fromUtf8(QJsonDocument(array).toJson(QJsonDocument::Compact));
}

RowGrip::RowGrip(QWidget* row, std::function<void(int from, int to)> on_move)
    : QLabel(row), row_(row), on_move_(std::move(on_move)) {
  setPixmap(App::Get().Glyph(QStringLiteral("grip-vertical"),
                             App::Get().Token("muted-foreground"), kGripSize));
  setCursor(Qt::OpenHandCursor);
  setToolTip(QStringLiteral("Drag to reorder"));
}

void RowGrip::mousePressEvent(QMouseEvent* event) {
  setCursor(Qt::ClosedHandCursor);
  event->accept();
}

void RowGrip::mouseReleaseEvent(QMouseEvent* event) {
  setCursor(Qt::OpenHandCursor);
  QWidget* list = row_->parentWidget();
  if (list == nullptr || !on_move_) return;
  const int from = row_->property(kRowProperty).toInt();
  const int y = list->mapFromGlobal(event->globalPosition().toPoint()).y();
  int to = from;
  for (QWidget* other :
       list->findChildren<QWidget*>(QString(), Qt::FindDirectChildrenOnly)) {
    const QVariant index = other->property(kRowProperty);
    if (!index.isValid()) continue;
    if (y >= other->y() && y < other->y() + other->height()) to = index.toInt();
  }
  if (to != from) on_move_(from, to);
}

QWidget* MakeStringList(const bridge::FieldView& field,
                        const CommitJson& commit) {
  QVBoxLayout* rows = nullptr;
  QWidget* box = Shell(field, &rows);
  const QJsonArray items = ArrayOf(field);
  const int count = static_cast<int>(items.size());
  for (int i = 0; i < count; ++i) {
    QWidget* row = Row(box, i);
    auto* layout = static_cast<QHBoxLayout*>(row->layout());
    layout->addWidget(new RowGrip(row, [items, commit](int from, int to) {
      commit(JsonText(Moved(items, from, to)));
    }));
    auto* text = new VarText(items.at(i).toString(), false, field.pauses,
                             QString(), row);
    QObject::connect(text, &VarText::Committed, row,
                     [items, i, commit](const QString& value) {
                       QJsonArray next = items;
                       next.replace(i, value);
                       commit(JsonText(next));
                     });
    layout->addWidget(text, 1);
    auto* remove =
        RemoveButton(ItemName(field, i),
                     static_cast<std::uint32_t>(count) > field.min_items, row);
    QObject::connect(remove, &QToolButton::clicked, row, [items, i, commit] {
      QJsonArray next = items;
      next.removeAt(i);
      commit(JsonText(next));
    });
    layout->addWidget(remove, 0, Qt::AlignTop);
    rows->addWidget(row);
  }
  auto* add =
      AddButton(QStringLiteral("Add"),
                static_cast<std::uint32_t>(count) < field.max_items, box);
  const QJsonValue fresh = ItemOf(field);
  QObject::connect(add, &QPushButton::clicked, box, [items, fresh, commit] {
    QJsonArray next = items;
    next.append(fresh);
    commit(JsonText(next));
  });
  rows->addWidget(add, 0, Qt::AlignLeft);
  return box;
}

QWidget* MakeNumberList(const bridge::FieldView& field,
                        const CommitJson& commit) {
  QVBoxLayout* rows = nullptr;
  QWidget* box = Shell(field, &rows);
  const QJsonArray items = ArrayOf(field);
  const int count = static_cast<int>(items.size());
  for (int i = 0; i < count; ++i) {
    QWidget* row = Row(box, i);
    auto* layout = static_cast<QHBoxLayout*>(row->layout());
    layout->addWidget(new RowGrip(row, [items, commit](int from, int to) {
      commit(JsonText(Moved(items, from, to)));
    }));
    auto* line =
        new QLineEdit(QLocale::c().toString(items.at(i).toDouble()), row);
    auto* validator = new QDoubleValidator(line);
    validator->setLocale(QLocale::c());
    line->setValidator(validator);
    line->setToolTip(ItemName(field, i));
    const QString initial = line->text();
    QObject::connect(line, &QLineEdit::editingFinished, row,
                     [items, i, commit, line, initial] {
                       if (line->text() == initial) return;
                       bool ok = false;
                       const double value =
                           QLocale::c().toDouble(line->text(), &ok);
                       if (!ok) return;
                       QJsonArray next = items;
                       next.replace(i, value);
                       commit(JsonText(next));
                     });
    layout->addWidget(line, 1);
    auto* remove =
        RemoveButton(ItemName(field, i),
                     static_cast<std::uint32_t>(count) > field.min_items, row);
    QObject::connect(remove, &QToolButton::clicked, row, [items, i, commit] {
      QJsonArray next = items;
      next.removeAt(i);
      commit(JsonText(next));
    });
    layout->addWidget(remove);
    rows->addWidget(row);
  }
  auto* add =
      AddButton(QStringLiteral("Add"),
                static_cast<std::uint32_t>(count) < field.max_items, box);
  const QJsonValue fresh = ItemOf(field);
  QObject::connect(add, &QPushButton::clicked, box, [items, fresh, commit] {
    QJsonArray next = items;
    next.append(fresh);
    commit(JsonText(next));
  });
  rows->addWidget(add, 0, Qt::AlignLeft);
  return box;
}

QWidget* MakeColorList(const bridge::FieldView& field,
                       const QStringList& swatches, const CommitJson& commit) {
  auto* box = new QWidget();
  auto* flow = new FlowLayout(box, 6);
  const QJsonArray items = ArrayOf(field);
  const int count = static_cast<int>(items.size());
  for (int i = 0; i < count; ++i) {
    auto* swatch = new ColorChip(items.at(i).toString(), kSwatch, box);
    swatch->setObjectName(QStringLiteral("Swatch"));
    swatch->setToolTip(
        QStringLiteral("Colour %1: %2").arg(i + 1).arg(items.at(i).toString()));
    QObject::connect(
        swatch, &QToolButton::clicked, box,
        [items, i, commit, swatch, swatches] {
          ColorPicker::Open(
              swatch, items.at(i).toString(), QString(), swatches,
              [items, i, commit,
               chip = QPointer<ColorChip>(swatch)](const QString& picked) {
                if (chip != nullptr) chip->SetValue(picked);
                QJsonArray next = items;
                next.replace(i, picked);
                commit(JsonText(next));
              });
        });
    swatch->setContextMenuPolicy(Qt::ActionsContextMenu);
    if (static_cast<std::uint32_t>(count) > field.min_items) {
      auto* remove =
          new QAction(QStringLiteral("Remove colour %1").arg(i + 1), swatch);
      QObject::connect(remove, &QAction::triggered, box, [items, i, commit] {
        QJsonArray next = items;
        next.removeAt(i);
        commit(JsonText(next));
      });
      swatch->addAction(remove);
    }
    flow->addWidget(swatch);
  }
  auto* add =
      MakeIconButton(QStringLiteral("plus"), QStringLiteral("Add colour"), box);
  add->setEnabled(static_cast<std::uint32_t>(count) < field.max_items);
  const QJsonValue fresh = ItemOf(field);
  QObject::connect(add, &QToolButton::clicked, box, [items, fresh, commit] {
    QJsonArray next = items;
    next.append(fresh);
    commit(JsonText(next));
  });
  flow->addWidget(add);
  return box;
}

QWidget* MakeObjectList(const bridge::FieldView& field,
                        const CommitJson& commit, QSet<QString>* closed,
                        QHash<QString, QVBoxLayout*>* bodies) {
  QVBoxLayout* rows = nullptr;
  QWidget* box = Shell(field, &rows);
  const QJsonArray items = ArrayOf(field);
  const int count = static_cast<int>(items.size());
  const QString path = ToQ(field.key);
  int index = 0;
  for (const auto& summary : field.choices) {
    const QString group = QStringLiteral("%1/%2").arg(path).arg(index);
    auto* item = new QFrame(box);
    item->setProperty(kRowProperty, index);
    item->setProperty("role", "list-item");
    auto* column = new QVBoxLayout(item);
    column->setContentsMargins(4, 4, 8, 8);
    column->setSpacing(6);
    auto* head = new QHBoxLayout();
    head->setSpacing(4);
    head->addWidget(new RowGrip(item, [items, commit](int from, int to) {
      commit(JsonText(Moved(items, from, to)));
    }));
    const bool start_closed = count >= kCollapseFrom;
    const QString closed_key =
        group + (start_closed ? QStringLiteral("+") : QString());
    const bool open = start_closed ? closed->contains(closed_key)
                                   : !closed->contains(closed_key);
    auto* title = MakeButton(
        ToQ(summary.label), "row",
        open ? QStringLiteral("chevron-down") : QStringLiteral("chevron-right"),
        item);
    head->addWidget(title, 1);
    const int i = index;
    auto* copy = MakeIconButton(
        QStringLiteral("copy"),
        QStringLiteral("Duplicate %1").arg(ToQ(summary.label)), item);
    copy->setEnabled(static_cast<std::uint32_t>(count) < field.max_items);
    QObject::connect(copy, &QToolButton::clicked, item, [items, i, commit] {
      QJsonArray next = items;
      next.insert(i + 1, items.at(i));
      commit(JsonText(next));
    });
    head->addWidget(copy);
    auto* remove =
        RemoveButton(ToQ(summary.label),
                     static_cast<std::uint32_t>(count) > field.min_items, item);
    QObject::connect(remove, &QToolButton::clicked, item, [items, i, commit] {
      QJsonArray next = items;
      next.removeAt(i);
      commit(JsonText(next));
    });
    head->addWidget(remove);
    column->addLayout(head);
    auto* body = new QWidget(item);
    auto* body_layout = new QVBoxLayout(body);
    body_layout->setContentsMargins(20, 0, 0, 0);
    body_layout->setSpacing(12);
    body->setVisible(open);
    column->addWidget(body);
    bodies->insert(group, body_layout);
    QObject::connect(
        title, &QPushButton::clicked, item, [closed, closed_key, body, title] {
          if (!closed->remove(closed_key)) closed->insert(closed_key);
          body->setVisible(!body->isVisible());
          title->setIcon(App::Get().Icon(body->isVisible()
                                             ? QStringLiteral("chevron-down")
                                             : QStringLiteral("chevron-right"),
                                         App::Get().Token("foreground")));
        });
    rows->addWidget(item);
    ++index;
  }
  auto* add =
      AddButton(QStringLiteral("Add"),
                static_cast<std::uint32_t>(count) < field.max_items, box);
  const QJsonValue fresh = ItemOf(field);
  QObject::connect(add, &QPushButton::clicked, box, [items, fresh, commit] {
    QJsonArray next = items;
    next.append(fresh);
    commit(JsonText(next));
  });
  rows->addWidget(add, 0, Qt::AlignLeft);
  return box;
}

}
