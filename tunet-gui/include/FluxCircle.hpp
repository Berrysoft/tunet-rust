#pragma once

#include <Model.hpp>
#include <QWidget>
#include <cstdint>

namespace TUNet
{
    struct FluxCircle : QWidget
    {
    public:
        FluxCircle(QWidget* parent = nullptr);
        ~FluxCircle() override;

        void set_color(const QColor& c);

        void update_flux(Flux flux, double balance);

    protected:
        void paintEvent(QPaintEvent* event) override;

    private:
        QColor m_color{};
        Flux m_flux{};
        double m_balance{};
    };
} // namespace TUNet
