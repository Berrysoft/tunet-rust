#pragma once

#include <Model.hpp>
#include <QChart>
#include <QChartView>
#include <QVBoxLayout>
#include <QWidget>

struct ChartPage : QWidget
{
public:
    ChartPage(QWidget* parent, Model* pmodel);
    ~ChartPage() override;

    void update_details();

private:
    Model* m_pmodel{};

    QVBoxLayout m_chart_layout{ this };

    QChartView m_daily_view{ this };
    QChart m_daily_chart{};
};
