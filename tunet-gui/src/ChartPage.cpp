#include <ChartPage.hpp>
#include <QBarCategoryAxis>
#include <QBarSeries>
#include <QBarSet>
#include <QDateTimeAxis>
#include <QLineSeries>
#include <QValueAxis>

ChartPage::ChartPage(QWidget* parent, Model* pmodel) : QWidget(parent), m_pmodel(pmodel)
{
    m_daily_chart.setTitle(u8"按日统计");
    m_daily_chart.legend()->setVisible(false);
    m_daily_view.setChart(&m_daily_chart);
    m_daily_view.setRenderHint(QPainter::Antialiasing);
    m_chart_layout.addWidget(&m_daily_view);

    m_time_chart.setTitle(u8"按时段统计");
    m_time_chart.legend()->setVisible(false);
    m_time_view.setChart(&m_time_chart);
    m_time_view.setRenderHint(QPainter::Antialiasing);
    m_chart_layout.addWidget(&m_time_view);

    setLayout(&m_chart_layout);

    QObject::connect(m_pmodel, &Model::details_changed, this, &ChartPage::update_details);
}

ChartPage::~ChartPage() {}

void ChartPage::update_details()
{
    {
        auto details = m_pmodel->details_grouped();
        m_daily_chart.removeAllSeries();
        double total_flux = 0.0;
        auto series = new QLineSeries();
        for (auto& d : details)
        {
            total_flux += d.flux / 1000000000.0;
            series->append(QDateTime{ d.logout_date, QTime{} }.toMSecsSinceEpoch(), total_flux);
        }
        m_daily_chart.addSeries(series);

        auto axis_x = new QDateTimeAxis();
        axis_x->setFormat(u8"d日");
        m_daily_chart.addAxis(axis_x, Qt::AlignBottom);
        series->attachAxis(axis_x);

        auto axis_y = new QValueAxis();
        axis_y->setLabelFormat("%.2lf G");
        m_daily_chart.addAxis(axis_y, Qt::AlignLeft);
        series->attachAxis(axis_y);
    }
    {
        auto details = m_pmodel->details_grouped_by_time(4);
        m_time_chart.removeAllSeries();
        auto series = new QBarSeries();
        auto set = new QBarSet("");
        auto axis_x = new QBarCategoryAxis();
        for (auto& d : details)
        {
            axis_x->append(QString(u8"%1~%2 时").arg(d.first).arg(d.first + 5));
            set->append(d.second / 1000000000.0);
        }
        series->append(set);
        m_time_chart.addSeries(series);

        m_time_chart.addAxis(axis_x, Qt::AlignBottom);
        series->attachAxis(axis_x);

        auto axis_y = new QValueAxis();
        axis_y->setLabelFormat("%.2lf G");
        m_time_chart.addAxis(axis_y, Qt::AlignLeft);
        series->attachAxis(axis_y);
    }
}
