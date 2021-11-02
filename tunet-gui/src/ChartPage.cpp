#include <ChartPage.hpp>
#include <QBarCategoryAxis>
#include <QBarSeries>
#include <QBarSet>
#include <QDateTimeAxis>
#include <QLineSeries>
#include <QValueAxis>

namespace TUNet
{
    ChartPage::ChartPage(QWidget* parent, Model* pmodel) : QWidget(parent), m_pmodel(pmodel)
    {
        m_refresh_button.setText(u"刷新"_qs);
        QObject::connect(&m_refresh_button, &QPushButton::clicked, this, &ChartPage::refresh_details);
        m_chart_layout.addWidget(&m_refresh_button);

        m_daily_chart.setTitle(u"按日统计"_qs);
        m_daily_chart.legend()->setVisible(false);
        m_daily_view.setChart(&m_daily_chart);
        m_daily_view.setRenderHint(QPainter::Antialiasing);
        m_chart_layout.addWidget(&m_daily_view);

        m_time_chart.setTitle(u"按时段统计"_qs);
        m_time_chart.legend()->setVisible(false);
        m_time_view.setChart(&m_time_chart);
        m_time_view.setRenderHint(QPainter::Antialiasing);
        m_chart_layout.addWidget(&m_time_view);

        setLayout(&m_chart_layout);

        QObject::connect(m_pmodel, &Model::details_changed, this, &ChartPage::update_details);
    }

    ChartPage::~ChartPage() {}

    void ChartPage::refresh_details()
    {
        m_pmodel->queue(Action::Details);
    }

    void ChartPage::update_details()
    {
        auto accent = accent_color();
        {
            auto details = m_pmodel->details_grouped();
            m_daily_chart.removeAllSeries();
            for (auto axis : m_daily_chart.axes())
            {
                m_daily_chart.removeAxis(axis);
            }
            double total_flux = 0.0;
            auto series = new QLineSeries();
            for (auto& d : details)
            {
                total_flux += d.second.toGb();
                series->append(QDateTime{ d.first, QTime{} }.toMSecsSinceEpoch(), total_flux);
            }
            series->setColor(accent);
            m_daily_chart.addSeries(series);

            auto axis_x = new QDateTimeAxis();
            axis_x->setFormat(u"d日"_qs);
            m_daily_chart.addAxis(axis_x, Qt::AlignBottom);
            series->attachAxis(axis_x);

            auto axis_y = new QValueAxis();
            axis_y->setLabelFormat(u"%.2lf G"_qs);
            m_daily_chart.addAxis(axis_y, Qt::AlignLeft);
            series->attachAxis(axis_y);
        }
        {
            auto details = m_pmodel->details_grouped_by_time(4);
            m_time_chart.removeAllSeries();
            for (auto axis : m_time_chart.axes())
            {
                m_time_chart.removeAxis(axis);
            }
            auto series = new QBarSeries();
            auto set = new QBarSet({});
            auto axis_x = new QBarCategoryAxis();
            for (auto& d : details)
            {
                axis_x->append(u"%1~%2 时"_qs.arg(d.first).arg(d.first + 5));
                set->append(d.second.toGb());
            }
            set->setColor(accent);
            series->append(set);
            m_time_chart.addSeries(series);

            m_time_chart.addAxis(axis_x, Qt::AlignBottom);
            series->attachAxis(axis_x);

            auto axis_y = new QValueAxis();
            axis_y->setLabelFormat(u"%.2lf G"_qs);
            m_time_chart.addAxis(axis_y, Qt::AlignLeft);
            series->attachAxis(axis_y);
        }
    }
} // namespace TUNet
