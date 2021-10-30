#include <DetailPage.hpp>
#include <QHeaderView>

DetailPage::DetailPage(QWidget* parent, Model* pmodel) : QWidget(parent), m_pmodel(pmodel)
{
    m_details.setHorizontalHeaderLabels(QStringList{ u8"登录时间", u8"注销时间", u8"流量" });

    m_details_table.setModel(&m_details);
    m_details_table.horizontalHeader()->setSectionResizeMode(QHeaderView::Stretch);
    m_details_table.verticalHeader()->setVisible(false);
    m_details_table.setSortingEnabled(true);
    m_details_layout.addWidget(&m_details_table);

    setLayout(&m_details_layout);

    QObject::connect(pmodel, &Model::details_changed, this, &DetailPage::update_details);
}

DetailPage::~DetailPage() {}

void DetailPage::update_details()
{
    static QStringView DATETIME_FORMAT = u"yyyy-MM-dd hh:mm:ss";

    m_details.clear();
    m_details.setHorizontalHeaderLabels(QStringList{ u8"登录时间", u8"注销时间", u8"流量" });

    auto ds = m_pmodel->details();
    for (auto& d : ds)
    {
        auto login_time = new QStandardItem(d.login_time.toString(DATETIME_FORMAT));
        login_time->setTextAlignment(Qt::AlignHCenter);
        auto logout_time = new QStandardItem(d.logout_time.toString(DATETIME_FORMAT));
        logout_time->setTextAlignment(Qt::AlignHCenter);
        auto flux = new QStandardItem(tunet_format_flux(d.flux));
        flux->setTextAlignment(Qt::AlignHCenter);
        m_details.appendRow({ login_time, logout_time, flux });
    }
}

void DetailPage::showEvent(QShowEvent* event)
{
    m_pmodel->queue(Action::Details);
}
