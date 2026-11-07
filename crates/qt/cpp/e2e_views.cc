#include <QApplication>
#include <QGraphicsItem>
#include <QGraphicsView>
#include <QImage>
#include <QKeyEvent>
#include <QPushButton>
#include <QSet>
#include <QtTest/QTest>
#include <cmath>
#include <memory>

#include "app.h"
#include "color_picker.h"
#include "gradient_field.h"
#include "graph_canvas.h"
#include "main_window.h"
#include "opsis-qt/src/bridge.rs.h"
#include "scenario.h"
#include "start_screen.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr int kProjectMs = 20'000;
constexpr int kSwatches = 14;
constexpr int kBarWidth = 300;

QString Expect(bool ok, const QString& what) { return ok ? QString() : what; }

QColor IconInk(const QAbstractButton* button) {
  const QImage icon = button->icon()
                          .pixmap(button->iconSize())
                          .toImage()
                          .convertToFormat(QImage::Format_ARGB32);
  for (int y = 0; y < icon.height(); ++y) {
    for (int x = 0; x < icon.width(); ++x) {
      const QColor pixel = icon.pixelColor(x, y);
      if (pixel.alpha() == 255) return pixel;
    }
  }
  return {};
}

QSet<QString> SelectedCards(GraphCanvas* canvas) {
  QSet<QString> ids;
  for (QGraphicsItem* item : canvas->scene()->selectedItems()) {
    if (auto* card = dynamic_cast<CardItem*>(item)) ids.insert(card->id());
  }
  return ids;
}

void Select(const QStringList& ids) {
  rust::Vec<rust::String> chosen;
  for (const QString& id : ids) chosen.push_back(ToRust(id));
  App::Get().session().select(std::move(chosen));
}

QString Overlaps(const QWidget* parent) {
  QList<QWidget*> parts;
  for (QObject* child : parent->children()) {
    auto* part = qobject_cast<QWidget*>(child);
    if (part != nullptr && part->isVisible()) parts << part;
  }
  for (int i = 0; i < parts.size(); ++i) {
    for (int j = i + 1; j < parts.size(); ++j) {
      if (parts.at(i)->geometry().intersects(parts.at(j)->geometry())) {
        return QStringLiteral("%1 covers %2")
            .arg(parts.at(j)->accessibleName(), parts.at(i)->accessibleName());
      }
    }
  }
  return {};
}

}

void Scenario::BuildViews() {
  const QString projects = qEnvironmentVariable("OPSIS_QT_PROJECTS");
  GraphCanvas* canvas = window_->canvas();
  Add(
      QStringLiteral("views-skip-setup"),
      [this] { Click(window_, QStringLiteral("Skip for now")); }, 500,
      [this] {
        return Expect(window_->start_screen()->isVisible(),
                      QStringLiteral("skipping setup didn't show the start "
                                     "screen"));
      });
  Until(
      QStringLiteral("views-create-a-project"),
      [projects] {
        App::Get().session().create_project(ToRust(QStringLiteral("Views")),
                                            ToRust(QStringLiteral("todomvc")),
                                            ToRust(projects));
      },
      kProjectMs,
      [] {
        return App::Get().session().has_project() &&
               !App::Get().session().graph().nodes.empty();
      },
      [canvas] {
        const auto cards = App::Get().session().graph().nodes.size();
        return Expect(cards > 0 && canvas->isVisible(),
                      QStringLiteral("the canvas shows %1 cards").arg(cards));
      });
  Add(
      QStringLiteral("views-canvas-toggles-show-their-state"), [] {}, 100,
      [this, canvas] {
        QAbstractButton* grid = Find(canvas, QStringLiteral("Hide the grid"));
        QAbstractButton* map = Find(canvas, QStringLiteral("Hide the minimap"));
        if (map == nullptr) {
          map = Find(canvas, QStringLiteral("Show the minimap"));
        }
        const bool carded = grid != nullptr && map != nullptr &&
                            grid->parentWidget()->objectName() ==
                                QStringLiteral("CanvasToolbar") &&
                            map->parentWidget()->objectName() ==
                                QStringLiteral("CanvasToolbar");
        return Expect(carded, QStringLiteral("the grid and minimap buttons "
                                             "aren't in toolbar cards")) +
               Expect(grid != nullptr &&
                          grid->isChecked() == canvas->GridShown() &&
                          map != nullptr &&
                          map->isChecked() == canvas->minimap_visible(),
                      QStringLiteral(" a toggle doesn't show its state"));
      });
  Add(
      QStringLiteral("views-canvas-mirrors-the-selection"),
      [] {
        Select({QStringLiteral("open"), QStringLiteral("todo1")});
        QTest::qWait(100);
        Select({QStringLiteral("todo2")});
      },
      300,
      [canvas] {
        const QSet<QString> shown = SelectedCards(canvas);
        return Expect(shown == QSet<QString>{QStringLiteral("todo2")},
                      QStringLiteral("the canvas shows %1 selected")
                          .arg(QStringList(shown.values()).join(", ")));
      });
  Add(
      QStringLiteral("views-inspector-follows-the-selection"), [] {}, 300,
      [] {
        return Expect(
            ToQ(App::Get().session().inspector().id) == QStringLiteral("todo2"),
            QStringLiteral("the Inspector shows '%1'")
                .arg(ToQ(App::Get().session().inspector().id)));
      });
  Add(
      QStringLiteral("views-colour-picker-fits-its-swatches"),
      [this] {
        QStringList swatches;
        for (int i = 0; i < kSwatches; ++i) {
          swatches << QColor::fromHsv(i * 360 / kSwatches, 200, 220).name();
        }
        ColorPicker::Open(window_, QStringLiteral("#3b82f6"),
                          QStringLiteral("#000000"), swatches,
                          [](const QString&) {});
      },
      400,
      [] {
        auto* picker =
            qobject_cast<ColorPicker*>(QApplication::activePopupWidget());
        if (picker == nullptr) return QStringLiteral("no colour picker opened");
        const QString overlap = Overlaps(picker);
        picker->close();
        return Expect(overlap.isEmpty(),
                      QStringLiteral("in the picker, %1").arg(overlap));
      });
  Add(
      QStringLiteral("views-gradient-stops-sit-on-the-bar"), [] {}, 100,
      [] {
        auto bar = std::make_unique<StopBar>();
        bar->resize(kBarWidth, bar->sizeHint().height());
        bar->SetStops({{QStringLiteral("#000000"), 0.0},
                       {QStringLiteral("#ffffff"), 100.0}},
                      true, 0);
        const double middle = bar->Track().center().y();
        return Expect(std::abs(bar->HandleOf(0).y() - middle) < 0.5 &&
                          std::abs(bar->HandleOf(1).y() - middle) < 0.5,
                      QStringLiteral("the stops sit at %1, the bar's middle "
                                     "at %2")
                          .arg(bar->HandleOf(0).y())
                          .arg(middle));
      });
  Add(
      QStringLiteral("views-button-icon-follows-its-style"), [] {}, 100,
      [] {
        std::unique_ptr<QPushButton> button(MakeButton(
            QStringLiteral("Download"), "primary", QStringLiteral("download")));
        const QColor primary = IconInk(button.get());
        SetButtonVariant(button.get(), "outline", QStringLiteral("download"));
        const QColor outline = IconInk(button.get());
        App& app = App::Get();
        return Expect(
            primary == app.Token(QStringLiteral("primary-foreground")) &&
                outline == app.Token(QStringLiteral("foreground")),
            QStringLiteral("the icon is %1 then %2")
                .arg(primary.name(), outline.name()));
      });
}

}
