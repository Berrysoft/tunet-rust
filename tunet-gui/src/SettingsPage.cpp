#include <QHeaderView>
#include <SettingsPage.hpp>

namespace TUNet
{
    SettingsPage::SettingsPage(QWidget* parent, Model* pmodel) : QWidget(parent), m_pmodel(pmodel)
    {
        QFont title_font = m_user_title_label.font();
        title_font.setBold(true);
        title_font.setPointSizeF(title_font.pointSizeF() * 1.5);

        m_user_title_label.setFont(title_font);
        m_user_title_label.setAlignment(Qt::AlignHCenter);
        m_user_title_label.setText(u"当前用户"_qs);
        m_settings_layout.addWidget(&m_user_title_label);
        m_user_label.setAlignment(Qt::AlignHCenter);
        m_settings_layout.addWidget(&m_user_label);

        m_status_title_label.setFont(title_font);
        m_status_title_label.setAlignment(Qt::AlignHCenter);
        m_status_title_label.setText(u"网络状态"_qs);
        m_settings_layout.addWidget(&m_status_title_label);
        m_status_label.setAlignment(Qt::AlignHCenter);
        m_status_label.setText(m_pmodel->status());
        m_settings_layout.addWidget(&m_status_label);

        m_online_label.setFont(title_font);
        m_online_label.setAlignment(Qt::AlignHCenter);
        m_online_label.setText(u"管理连接"_qs);
        m_settings_layout.addWidget(&m_online_label);

        m_online_table.setColumnCount(5);
        m_online_table.setHorizontalHeaderLabels({ u"IP地址"_qs, u"登录时间"_qs, u"流量"_qs, u"MAC地址"_qs, u"设备"_qs });
        m_online_table.horizontalHeader()->setSectionResizeMode(QHeaderView::Stretch);
        m_online_table.horizontalHeader()->setSectionResizeMode(4, QHeaderView::ResizeToContents);
        m_online_table.verticalHeader()->setVisible(false);
        m_online_table.setSelectionBehavior(QTableWidget::SelectRows);
        m_online_table.setSelectionMode(QTableWidget::SingleSelection);
        QObject::connect(&m_online_table, &QTableWidget::itemSelectionChanged, this, &SettingsPage::selection_changed);
        m_settings_layout.addWidget(&m_online_table);

        m_connect_button.setText(u"连接IP"_qs);
        QObject::connect(&m_connect_button, &QPushButton::clicked, this, &SettingsPage::connect_ip);
        m_drop_button.setText(u"下线IP"_qs);
        m_drop_button.setEnabled(false);
        QObject::connect(&m_drop_button, &QPushButton::clicked, this, &SettingsPage::drop_ip);
        m_refresh_button.setText(u"刷新"_qs);
        QObject::connect(&m_refresh_button, &QPushButton::clicked, this, &SettingsPage::refresh_online);
        m_command_layout.addWidget(&m_connect_button);
        m_command_layout.addWidget(&m_drop_button);
        m_command_layout.addWidget(&m_refresh_button);
        m_settings_layout.addLayout(&m_command_layout);

        setLayout(&m_settings_layout);

        QObject::connect(m_pmodel, &Model::cred_changed, this, &SettingsPage::update_cred);
        QObject::connect(m_pmodel, &Model::onlines_changed, this, &SettingsPage::update_online);
    }

    SettingsPage::~SettingsPage() {}

    void SettingsPage::selection_changed()
    {
        m_drop_button.setEnabled(!m_online_table.selectedRanges().empty());
    }

    void SettingsPage::connect_ip()
    {
    }

    void SettingsPage::drop_ip()
    {
        auto users = m_pmodel->onlines();
        for (auto& range : m_online_table.selectedRanges())
        {
            for (int i = range.topRow(); i <= range.bottomRow(); i++)
            {
                m_pmodel->queue_drop(users[i].address);
            }
        }
    }

    void SettingsPage::refresh_online()
    {
        m_pmodel->queue(Action::Online);
    }

    void SettingsPage::update_cred()
    {
        m_user_label.setText(m_pmodel->cred().username);
    }

    void SettingsPage::update_online()
    {
        auto users = m_pmodel->onlines();
        m_online_table.clearContents();
        m_online_table.setRowCount((int)users.size());
        int row = 0;
        for (auto& u : users)
        {
            auto address = new QTableWidgetItem(format_ip(u.address));
            address->setTextAlignment(Qt::AlignCenter);
            m_online_table.setItem(row, 0, address);

            auto login_time = new QTableWidgetItem(format_datetime(u.login_time));
            login_time->setTextAlignment(Qt::AlignCenter);
            m_online_table.setItem(row, 1, login_time);

            auto flux = new QTableWidgetItem(u.flux.toString());
            flux->setTextAlignment(Qt::AlignCenter);
            m_online_table.setItem(row, 2, flux);

            auto mac_address = new QTableWidgetItem(u.mac_address ? format_mac_address(*u.mac_address) : QString{});
            mac_address->setTextAlignment(Qt::AlignCenter);
            m_online_table.setItem(row, 3, mac_address);

            auto device = new QTableWidgetItem(u.is_local ? u"本机"_qs : u"未知"_qs);
            device->setTextAlignment(Qt::AlignCenter);
            m_online_table.setItem(row, 4, device);

            row++;
        }
    }
} // namespace TUNet
