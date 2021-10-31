#include <ChartPage.hpp>
#include <QLineSeries>

ChartPage::ChartPage(QWidget* parent, Model* pmodel) : QWidget(parent), m_pmodel(pmodel)
{
    m_daily_view.setRenderHint(QPainter::Antialiasing);
    m_daily_view.setChart(&m_daily_chart);
    m_chart_layout.addWidget(&m_daily_view);

    setLayout(&m_chart_layout);

    QObject::connect(m_pmodel, &Model::details_changed, this, &ChartPage::update_details);
}

ChartPage::~ChartPage() {}

void ChartPage::update_details()
{
    auto details = m_pmodel->details_grouped();
    m_daily_chart.removeAllSeries();
    auto series = new QLineSeries();
    for (auto& d : details)
    {
        series->append(d.logout_date.day(), d.flux);
    }
    m_daily_chart.addSeries(series);
    m_daily_chart.createDefaultAxes();
}
