#include "window_chrome.h"

#include <QApplication>
#include <QEvent>
#include <QFrame>
#include <QHBoxLayout>
#include <QMouseEvent>
#include <QPainter>
#include <QStyle>
#include <QWindow>

#include "app.h"
#include "event_hub.h"
#include "opsis-qt/src/bridge.rs.h"
#include "widgets.h"

namespace opsis {

namespace {

constexpr int kBarHeight = 48;
constexpr int kDragStartPx = 3;
constexpr int kResizeMargin = 6;
constexpr int kLogoSize = 20;
constexpr int kLogoMargin = 4;
constexpr int kBarPadLeft = 8;
constexpr int kRuleHeight = 20;

class BorderOverlay : public QWidget {
 public:
  explicit BorderOverlay(QWidget* parent) : QWidget(parent) {
    setAttribute(Qt::WA_TransparentForMouseEvents);
    setAttribute(Qt::WA_NoSystemBackground);
  }

 protected:
  void paintEvent(QPaintEvent*) override {
    QPainter painter(this);
    painter.setPen(QPen(App::Get().Token("window-border"), 1));
    painter.drawRect(rect().adjusted(0, 0, -1, -1));
  }
};

bool Framed(const QWidget* window) {
  return !window->isMaximized() && !window->isFullScreen();
}

}

AppBar::AppBar(QWidget* window, bool resizable, bool menu)
    : QWidget(window), window_(window) {
  setObjectName(QStringLiteral("AppBar"));
  setAttribute(Qt::WA_StyledBackground);
  setFixedHeight(kBarHeight);
  auto* layout = new QHBoxLayout(this);
  layout->setContentsMargins(kBarPadLeft, 0, 0, 0);
  layout->setSpacing(8);
  auto* logo = new QLabel(this);
  logo->setPixmap(App::Get().Logo(kLogoSize));
  logo->setContentsMargins(kLogoMargin, 0, kLogoMargin, 0);
  layout->addWidget(logo);
  content_ = new QHBoxLayout();
  content_->setSpacing(8);
  if (menu) {
    menu_bar_ = new QMenuBar(this);
    menu_bar_->setNativeMenuBar(false);
    menu_bar_->setSizePolicy(QSizePolicy::Maximum, QSizePolicy::Preferred);
    layout->addWidget(menu_bar_, 0, Qt::AlignVCenter);
    auto* rule = new QFrame(this);
    rule->setProperty("role", "vrule");
    rule->setFixedSize(1, kRuleHeight);
    layout->addWidget(rule);
  }
  layout->addLayout(content_, 1);
  api_ = new QPushButton(QStringLiteral("API"), this);
  api_->setObjectName(QStringLiteral("ApiBadge"));
  api_->setFocusPolicy(Qt::NoFocus);
  api_->setCursor(Qt::PointingHandCursor);
  layout->addWidget(api_);
  connect(api_, &QPushButton::clicked, this, &AppBar::ApiBadgeClicked);
  connect(EventHub::Instance(), &EventHub::ApiChanged, this,
          &AppBar::SyncApiBadge);
  connect(EventHub::Instance(), &EventHub::SettingsChanged, this,
          &AppBar::SyncApiBadge);
  SyncApiBadge();
  auto* controls_rule = new QFrame(this);
  controls_rule->setProperty("role", "vrule");
  controls_rule->setFixedSize(1, kRuleHeight);
  layout->addSpacing(4);
  layout->addWidget(controls_rule);
  layout->addSpacing(4);
  const QColor ink = App::Get().Token("foreground");
  auto make = [&](const QString& icon, const QString& tip, const char* object) {
    auto* button = new QToolButton(this);
    button->setObjectName(QLatin1String(object));
    button->setIcon(App::Get().Icon(icon, ink, 16));
    button->setIconSize(QSize(16, 16));
    button->setToolTip(tip);
    button->setFixedHeight(kBarHeight);
    button->setFocusPolicy(Qt::NoFocus);
    layout->addWidget(button);
    return button;
  };
  if (resizable) {
    connect(make(QStringLiteral("minus"), QStringLiteral("Minimize"),
                 "WindowButton"),
            &QToolButton::clicked, window_, &QWidget::showMinimized);
    maximize_ = make(QStringLiteral("square"), QStringLiteral("Maximize"),
                     "WindowButton");
    connect(maximize_, &QToolButton::clicked, this, &AppBar::ToggleMaximized);
  }
  connect(make(QStringLiteral("x"), QStringLiteral("Close"), "CloseButton"),
          &QToolButton::clicked, window_, &QWidget::close);
  window_->installEventFilter(this);
}

void AppBar::SyncApiBadge() {
  const bridge::BarView bar = App::Get().session().bar();
  api_->setVisible(bar.api_enabled);
  api_->setProperty("tone", bar.api_listening ? "secondary" : "destructive");
  api_->setToolTip(
      bar.api_listening
          ? QStringLiteral(
                "Scripts and agents on this computer can use the open project.")
          : QStringLiteral("The API is switched on but isn't running. Open "
                           "its settings to see why."));
  api_->style()->unpolish(api_);
  api_->style()->polish(api_);
}

void AppBar::ToggleMaximized() {
  if (window_->isMaximized()) {
    window_->showNormal();
  } else {
    window_->showMaximized();
  }
}

void AppBar::SyncMaximizeIcon() {
  if (maximize_ == nullptr) return;
  const bool maximized = window_->isMaximized();
  maximize_->setIcon(App::Get().Icon(
      maximized ? QStringLiteral("copy") : QStringLiteral("square"),
      App::Get().Token("foreground"), 16));
  maximize_->setToolTip(maximized ? QStringLiteral("Restore")
                                  : QStringLiteral("Maximize"));
}

bool AppBar::eventFilter(QObject* watched, QEvent* event) {
  if (watched == window_ && event->type() == QEvent::WindowStateChange) {
    SyncMaximizeIcon();
  }
  return QWidget::eventFilter(watched, event);
}

void AppBar::mousePressEvent(QMouseEvent* event) {
  if (event->button() == Qt::LeftButton) {
    pressed_ = true;
    press_ = event->globalPosition().toPoint();
  }
  QWidget::mousePressEvent(event);
}

void AppBar::mouseMoveEvent(QMouseEvent* event) {
  if (pressed_ &&
      (event->globalPosition().toPoint() - press_).manhattanLength() >=
          kDragStartPx) {
    pressed_ = false;
    if (QWindow* handle = window_->windowHandle()) handle->startSystemMove();
  }
  QWidget::mouseMoveEvent(event);
}

void AppBar::mouseReleaseEvent(QMouseEvent* event) {
  pressed_ = false;
  QWidget::mouseReleaseEvent(event);
}

void AppBar::mouseDoubleClickEvent(QMouseEvent* event) {
  if (event->button() == Qt::LeftButton && maximize_ != nullptr) {
    ToggleMaximized();
  }
  QWidget::mouseDoubleClickEvent(event);
}

WindowChrome::WindowChrome(QWidget* window)
    : QObject(window), window_(window), border_(new BorderOverlay(window)) {
  window_->setWindowFlag(Qt::FramelessWindowHint);
  window_->setAttribute(Qt::WA_Hover);
  window_->installEventFilter(this);
  Sync();
}

void WindowChrome::Sync() {
  const bool framed = Framed(window_);
  const int margin = framed ? 1 : 0;
  window_->setContentsMargins(margin, margin, margin, margin);
  border_->setVisible(framed);
  border_->setGeometry(window_->rect());
  border_->raise();
  if (QWindow* handle = window_->windowHandle()) {
    handle->removeEventFilter(this);
    handle->installEventFilter(this);
  }
}

Qt::Edges WindowChrome::EdgesAt(const QPointF& position) const {
  Qt::Edges edges;
  if (!Framed(window_)) return edges;
  const QRect area = window_->rect();
  if (position.x() < kResizeMargin) edges |= Qt::LeftEdge;
  if (position.x() > area.width() - kResizeMargin) edges |= Qt::RightEdge;
  if (position.y() < kResizeMargin) edges |= Qt::TopEdge;
  if (position.y() > area.height() - kResizeMargin) edges |= Qt::BottomEdge;
  return edges;
}

bool WindowChrome::eventFilter(QObject* watched, QEvent* event) {
  if (watched == window_) {
    switch (event->type()) {
      case QEvent::Resize:
      case QEvent::WindowStateChange:
      case QEvent::Show:
      case QEvent::ChildAdded:
        Sync();
        break;
      default:
        break;
    }
    return false;
  }
  if (watched != window_->windowHandle()) return false;
  if (event->type() == QEvent::MouseMove) {
    auto* mouse = static_cast<QMouseEvent*>(event);
    if (mouse->buttons() != Qt::NoButton) return false;
    const Qt::Edges edges = EdgesAt(mouse->position());
    if (edges == (Qt::LeftEdge | Qt::TopEdge) ||
        edges == (Qt::RightEdge | Qt::BottomEdge)) {
      window_->setCursor(Qt::SizeFDiagCursor);
    } else if (edges == (Qt::RightEdge | Qt::TopEdge) ||
               edges == (Qt::LeftEdge | Qt::BottomEdge)) {
      window_->setCursor(Qt::SizeBDiagCursor);
    } else if (edges & (Qt::LeftEdge | Qt::RightEdge)) {
      window_->setCursor(Qt::SizeHorCursor);
    } else if (edges & (Qt::TopEdge | Qt::BottomEdge)) {
      window_->setCursor(Qt::SizeVerCursor);
    } else {
      window_->unsetCursor();
    }
    return false;
  }
  if (event->type() == QEvent::MouseButtonPress) {
    auto* mouse = static_cast<QMouseEvent*>(event);
    const Qt::Edges edges = EdgesAt(mouse->position());
    if (mouse->button() == Qt::LeftButton && edges != Qt::Edges()) {
      window_->windowHandle()->startSystemResize(edges);
      return true;
    }
  }
  return false;
}

}
