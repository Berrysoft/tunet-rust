#pragma once

#include <QWidget>
#include <cstdint>

struct FluxCircle : QWidget
{
public:
    FluxCircle(QWidget* parent = nullptr);
    ~FluxCircle() override;

    void update_flux(std::uint64_t flux, double balance);

private:
    void paintEvent(QPaintEvent* event) override;

private:
    std::uint64_t m_flux{};
    double m_balance{};
};
