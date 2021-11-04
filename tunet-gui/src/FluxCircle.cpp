#include <FluxCircle.hpp>
#include <QPainter>
#include <QPainterPath>
#include <algorithm>
#include <cmath>
#include <numbers>

namespace TUNet
{
    FluxCircle::FluxCircle(QWidget* parent) : QWidget(parent)
    {
        setMouseTracking(true);
    }

    FluxCircle::~FluxCircle() {}

    void FluxCircle::update_flux(Flux flux, double balance)
    {
        m_flux = flux;
        m_balance = balance;
        repaint();
    }

    static void draw_arc(QPainter& painter, const QPen& pen, const QPointF& center, double radius, double startAngle, double endAngle)
    {
        using std::numbers::pi;

        auto width = pen.widthF();
        double large_radius = radius + width / 2.0;
        double small_radius = radius - width / 2.0;
        double arcLength = endAngle - startAngle;

        QRectF large_rect{ center.x() - large_radius, center.y() - large_radius, large_radius * 2.0, large_radius * 2.0 };
        QRectF small_rect{ center.x() - small_radius, center.y() - small_radius, small_radius * 2.0, small_radius * 2.0 };
        QPainterPath path{};
        path.moveTo(center.x() + large_radius * std::cos(startAngle / 180.0 * pi), center.y() + large_radius * std::sin(startAngle / 180.0 * pi));
        path.arcTo(large_rect, -startAngle, -arcLength);
        path.lineTo(center.x() + small_radius * std::cos(endAngle / 180.0 * pi), center.y() + small_radius * std::sin(endAngle / 180.0 * pi));
        path.arcTo(small_rect, -endAngle, arcLength);
        path.closeSubpath();

        painter.fillPath(path, pen.brush());
    }

    void FluxCircle::paintEvent(QPaintEvent* event)
    {
        auto s = size();
        double radius = (std::min)(s.width(), s.height()) * 0.4;
        QPointF center{ s.width() / 2.0, s.height() / 2.0 };
        double line_width = radius * 0.2;

        auto accent = accent_color();
        auto accent_t1 = accent;
        accent_t1.setAlphaF(0.75f);
        auto accent_t2 = accent;
        accent_t2.setAlphaF(0.55f);

        double max_flux = m_balance + 50.0;

        double flux_length = m_flux.toGb() / max_flux * 360.0;
        double free_length = 50.0 / max_flux * 360.0;

        QPainter painter{ this };
        painter.setRenderHint(QPainter::Antialiasing);

        draw_arc(painter, { accent_t2, line_width }, center, radius, 90.0 + free_length, 360.0 + 90.0);
        draw_arc(painter, { accent_t1, line_width }, center, radius, 90.0 + flux_length, 90.0 + free_length);
        draw_arc(painter, { accent, line_width }, center, radius, 90.0, 90.0 + flux_length);
    }
} // namespace TUNet
