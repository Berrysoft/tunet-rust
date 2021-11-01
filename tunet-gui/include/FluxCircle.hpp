#pragma once

#include <Model.hpp>
#include <QWidget>
#include <cstdint>

struct FluxCircle : QWidget
{
public:
    FluxCircle(QWidget* parent = nullptr);
    ~FluxCircle() override;

    void update_flux(Flux flux, double balance);

protected:
    void paintEvent(QPaintEvent* event) override;

private:
    Flux m_flux{};
    double m_balance{};
};
