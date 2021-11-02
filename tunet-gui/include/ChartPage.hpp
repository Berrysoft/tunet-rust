#pragma once

#include <Model.hpp>
#include <QChart>
#include <QChartView>
#include <QPushButton>
#include <QVBoxLayout>
#include <QWidget>

namespace TUNet
{
    struct ChartPage : QWidget
    {
    public:
        ChartPage(QWidget* parent, Model* pmodel);
        ~ChartPage() override;

        void refresh_details();
        void update_details();

    private:
        Model* m_pmodel{};

        QVBoxLayout m_chart_layout{};

        QChartView m_daily_view{ this };
        QChart m_daily_chart{};

        QChartView m_time_view{ this };
        QChart m_time_chart{};

        QPushButton m_refresh_button{ this };
    };
} // namespace TUNet
