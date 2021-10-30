#include <DetailPage.hpp>
#include <QHeaderView>
#include <QLabel>

struct FluxItem : QTableWidgetItem
{
    FluxItem(const QString& text) : QTableWidgetItem(text) {}
    ~FluxItem() override {}

    bool operator<(const QTableWidgetItem& other) const override
    {
        return data(Qt::UserRole).toULongLong() < other.data(Qt::UserRole).toULongLong();
    }
};

DetailPage::DetailPage(QWidget* parent, Model* pmodel) : QWidget(parent), m_pmodel(pmodel)
{
    m_details_table.setColumnCount(3);
    m_details_table.setHorizontalHeaderLabels(QStringList{ u8"登录时间", u8"注销时间", u8"流量" });
    m_details_table.horizontalHeader()->setSectionResizeMode(QHeaderView::Stretch);
    m_details_table.verticalHeader()->setVisible(false);
    m_details_table.setSortingEnabled(true);
    m_details_layout.addWidget(&m_details_table);

    setLayout(&m_details_layout);

    QObject::connect(pmodel, &Model::details_changed, this, &DetailPage::update_details);

    m_pmodel->queue(Action::Details);
}

DetailPage::~DetailPage() {}

void DetailPage::update_details()
{
    static QStringView DATETIME_FORMAT = u"yyyy-MM-dd hh:mm:ss";

    auto ds = m_pmodel->details();
    m_details_table.clearContents();
    m_details_table.setSortingEnabled(false);
    m_details_table.setRowCount((int)ds.size());
    int row = 0;
    for (auto& d : ds)
    {
        auto login_time = new QTableWidgetItem(d.login_time.toString(DATETIME_FORMAT));
        login_time->setTextAlignment(Qt::AlignCenter);
        m_details_table.setItem(row, 0, login_time);

        auto logout_time = new QTableWidgetItem(d.logout_time.toString(DATETIME_FORMAT));
        logout_time->setTextAlignment(Qt::AlignCenter);
        m_details_table.setItem(row, 1, logout_time);

        auto flux = new FluxItem(tunet_format_flux(d.flux));
        flux->setTextAlignment(Qt::AlignCenter);
        flux->setData(Qt::UserRole, d.flux);
        m_details_table.setItem(row, 2, flux);

        row++;
    }
    m_details_table.setSortingEnabled(true);
}
