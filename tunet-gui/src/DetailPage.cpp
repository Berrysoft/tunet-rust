#include <DetailPage.hpp>
#include <QHeaderView>
#include <QLabel>

namespace TUNet
{
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
        m_details_table.setHorizontalHeaderLabels({ QStringLiteral(u"登录时间"), QStringLiteral(u"注销时间"), QStringLiteral(u"流量") });
        m_details_table.horizontalHeader()->setSectionResizeMode(QHeaderView::Stretch);
        m_details_table.verticalHeader()->setVisible(false);
        m_details_table.setSortingEnabled(true);
        m_details_table.setSelectionBehavior(QTableWidget::SelectRows);
        m_details_layout.addWidget(&m_details_table);

        m_refresh_button.setText(QStringLiteral(u"刷新"));
        QObject::connect(&m_refresh_button, &QPushButton::clicked, this, &DetailPage::refresh_details);
        m_details_layout.addWidget(&m_refresh_button);

        QObject::connect(m_pmodel, &Model::details_changed, this, &DetailPage::update_details);
        QObject::connect(m_pmodel, &Model::detail_busy_changed, this, &DetailPage::update_detail_busy);
    }

    DetailPage::~DetailPage() {}

    void DetailPage::refresh_details()
    {
        m_pmodel->queue(Action::Details);
    }

    void DetailPage::update_details()
    {
        auto ds = m_pmodel->details();
        m_details_table.clearContents();
        m_details_table.setSortingEnabled(false);
        m_details_table.setRowCount((int)ds.size());
        int row = 0;
        for (auto& d : ds)
        {
            auto login_time = new QTableWidgetItem(format_datetime(d.login_time));
            login_time->setTextAlignment(Qt::AlignCenter);
            m_details_table.setItem(row, 0, login_time);

            auto logout_time = new QTableWidgetItem(format_datetime(d.logout_time));
            logout_time->setTextAlignment(Qt::AlignCenter);
            m_details_table.setItem(row, 1, logout_time);

            auto flux = new FluxItem(d.flux.toString());
            flux->setTextAlignment(Qt::AlignCenter);
            flux->setData(Qt::UserRole, static_cast<qulonglong>(d.flux));
            m_details_table.setItem(row, 2, flux);

            row++;
        }
        m_details_table.setSortingEnabled(true);
    }

    void DetailPage::update_detail_busy()
    {
        m_refresh_button.setEnabled(!m_pmodel->detail_busy());
    }
} // namespace TUNet
