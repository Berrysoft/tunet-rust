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
        static QString CHART_VIEW_TRANSPARENT{ QStringLiteral(u"background: transparent") };

        auto& pal = palette();
        auto& base = pal.color(QPalette::Base);
        bool dark = ((base.redF() + base.greenF() + base.blueF()) / 3.0) < 0.5;

        m_daily_chart.setTitle(QStringLiteral(u"按日统计"));
        m_daily_chart.legend()->setVisible(false);
        m_daily_chart.setBackgroundVisible(false);
        m_daily_chart.setTheme(dark ? QChart::ChartThemeDark : QChart::ChartThemeLight);
        m_daily_view.setChart(&m_daily_chart);
        m_daily_view.setRenderHint(QPainter::Antialiasing);
        m_daily_view.setStyleSheet(CHART_VIEW_TRANSPARENT);
        m_chart_layout.addWidget(&m_daily_view);

        m_time_chart.setTitle(QStringLiteral(u"按时段统计"));
        m_time_chart.legend()->setVisible(false);
        m_time_chart.setBackgroundVisible(false);
        m_time_chart.setTheme(dark ? QChart::ChartThemeDark : QChart::ChartThemeLight);
        m_time_view.setChart(&m_time_chart);
        m_time_view.setRenderHint(QPainter::Antialiasing);
        m_time_view.setStyleSheet(CHART_VIEW_TRANSPARENT);
        m_chart_layout.addWidget(&m_time_view);

        m_chart_root_layout.addLayout(&m_chart_layout, 0, 0);
        m_detail_busy_indicator.setColor(m_pmodel->accent_color());
        m_chart_root_layout.addWidget(&m_detail_busy_indicator, 0, 0, Qt::AlignCenter);
        m_root_layout.addLayout(&m_chart_root_layout);

        m_refresh_button.setText(QStringLiteral(u"刷新"));
        QObject::connect(&m_refresh_button, &QPushButton::clicked, this, &ChartPage::refresh_details);
        m_root_layout.addWidget(&m_refresh_button);

        QObject::connect(m_pmodel, &Model::details_changed, this, &ChartPage::update_details);
        QObject::connect(m_pmodel, &Model::detail_busy_changed, this, &ChartPage::update_detail_busy);
    }

    ChartPage::~ChartPage() {}

    void ChartPage::refresh_details()
    {
        m_pmodel->queue(Action::Details);
    }

    void ChartPage::update_details()
    {
        auto accent = m_pmodel->accent_color();
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
            axis_x->setFormat(QStringLiteral(u"d日"));
            m_daily_chart.addAxis(axis_x, Qt::AlignBottom);
            series->attachAxis(axis_x);

            auto axis_y = new QValueAxis();
            axis_y->setLabelFormat(QStringLiteral(u"%.2lf G"));
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
                axis_x->append(QStringLiteral(u"%1~%2 时").arg(d.first).arg(d.first + 5));
                set->append(d.second.toGb());
            }
            set->setColor(accent);
            series->append(set);
            m_time_chart.addSeries(series);

            m_time_chart.addAxis(axis_x, Qt::AlignBottom);
            series->attachAxis(axis_x);

            auto axis_y = new QValueAxis();
            axis_y->setLabelFormat(QStringLiteral(u"%.2lf G"));
            m_time_chart.addAxis(axis_y, Qt::AlignLeft);
            series->attachAxis(axis_y);
        }
    }

    void ChartPage::update_detail_busy()
    {
        bool free = !m_pmodel->detail_busy();
        m_refresh_button.setEnabled(free);
        if (!free)
        {
            m_detail_busy_indicator.startAnimation();
        }
        else
        {
            m_detail_busy_indicator.stopAnimation();
        }
    }
} // namespace TUNet
