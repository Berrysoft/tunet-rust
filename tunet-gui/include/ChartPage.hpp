#pragma once

#include <Model.hpp>
#include <QChart>
#include <QChartView>
#include <QGridLayout>
#include <QProgressIndicator.hpp>
#include <QPushButton>
#include <QVBoxLayout>
#include <QWidget>

namespace TUNet
{
#if QT_VERSION < QT_VERSION_CHECK(6, 0, 0)
    using namespace QtCharts;
#endif

    struct ChartPage : QWidget
    {
    public:
        ChartPage(QWidget* parent, Model* pmodel);
        ~ChartPage() override;

        void refresh_details();
        void update_details();
        void update_detail_busy();

    private:
        Model* m_pmodel{};

        QVBoxLayout m_root_layout{ this };

        QGridLayout m_chart_root_layout{};

        QVBoxLayout m_chart_layout{};

        QChartView m_daily_view{};
        QChart m_daily_chart{};

        QChartView m_time_view{};
        QChart m_time_chart{};

        QProgressIndicator m_detail_busy_indicator{};

        QPushButton m_refresh_button{};
    };
} // namespace TUNet
