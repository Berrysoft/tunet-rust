#include <ConnectIPDialog.hpp>
#include <CredDialog.hpp>
#include <QApplication>
#include <QHeaderView>
#include <QMessageBox>
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
        m_user_title_label.setText(QStringLiteral(u"当前凭据"));
        m_settings_layout.addWidget(&m_user_title_label);
        m_user_button.setText(QStringLiteral(u"设置"));
        QObject::connect(&m_user_button, &QPushButton::clicked, this, &SettingsPage::set_credential);
        m_del_exit_button.setText(QStringLiteral(u"删除并退出"));
        QObject::connect(&m_del_exit_button, &QPushButton::clicked, this, &SettingsPage::delete_cred_and_exit);
        m_user_layout.addStretch();
        m_user_layout.addWidget(&m_user_label);
        m_user_layout.addWidget(&m_user_button);
        m_user_layout.addWidget(&m_del_exit_button);
        m_user_layout.addStretch();
        m_settings_layout.addLayout(&m_user_layout);

        m_status_title_label.setFont(title_font);
        m_status_title_label.setAlignment(Qt::AlignHCenter);
        m_status_title_label.setText(QStringLiteral(u"网络状态"));
        m_settings_layout.addWidget(&m_status_title_label);
        m_status_label.setAlignment(Qt::AlignHCenter);
        m_status_label.setText(m_pmodel->status());
        m_settings_layout.addWidget(&m_status_label);

        m_online_label.setFont(title_font);
        m_online_label.setAlignment(Qt::AlignHCenter);
        m_online_label.setText(QStringLiteral(u"管理连接"));
        m_settings_layout.addWidget(&m_online_label);

        m_online_table.setColumnCount(5);
        m_online_table.setHorizontalHeaderLabels({ QStringLiteral(u"IP地址"), QStringLiteral(u"登录时间"), QStringLiteral(u"流量"), QStringLiteral(u"MAC地址"), QStringLiteral(u"设备") });
        m_online_table.horizontalHeader()->setSectionResizeMode(QHeaderView::Stretch);
        m_online_table.horizontalHeader()->setSectionResizeMode(4, QHeaderView::ResizeToContents);
        m_online_table.verticalHeader()->setVisible(false);
        m_online_table.setSelectionBehavior(QTableWidget::SelectRows);
        m_online_table.setSelectionMode(QTableWidget::SingleSelection);
        QObject::connect(&m_online_table, &QTableWidget::itemSelectionChanged, this, &SettingsPage::selection_changed);
        m_settings_layout.addWidget(&m_online_table);

        m_connect_button.setText(QStringLiteral(u"认证IP"));
        QObject::connect(&m_connect_button, &QPushButton::clicked, this, &SettingsPage::connect_ip);
        m_drop_button.setText(QStringLiteral(u"下线IP"));
        m_drop_button.setEnabled(false);
        QObject::connect(&m_drop_button, &QPushButton::clicked, this, &SettingsPage::drop_ip);
        m_refresh_button.setText(QStringLiteral(u"刷新"));
        QObject::connect(&m_refresh_button, &QPushButton::clicked, this, &SettingsPage::refresh_online);
        m_command_layout.addWidget(&m_connect_button);
        m_command_layout.addWidget(&m_drop_button);
        m_command_layout.addWidget(&m_refresh_button);
        m_settings_layout.addLayout(&m_command_layout);

        QObject::connect(m_pmodel, &Model::ask_cred, this, &SettingsPage::ask_credential);
        QObject::connect(m_pmodel, &Model::cred_changed, this, &SettingsPage::update_cred);
        QObject::connect(m_pmodel, &Model::onlines_changed, this, &SettingsPage::update_online);
        QObject::connect(m_pmodel, &Model::online_busy_changed, this, &SettingsPage::update_online_busy);
    }

    SettingsPage::~SettingsPage() {}

    void SettingsPage::ask_credential(const Credential& cred)
    {
        CredDialog dialog{ this };
        dialog.set_credential(cred);
        if (dialog.exec() == QDialog::Accepted)
        {
            m_pmodel->queue_cred(dialog.credential());
        }
    }

    void SettingsPage::set_credential()
    {
        ask_credential(m_pmodel->cred());
    }

    void SettingsPage::selection_changed()
    {
        m_drop_button.setEnabled(!m_online_table.selectedRanges().empty());
    }

    void SettingsPage::connect_ip()
    {
        ConnectIPDialog dialog{ this };
        if (dialog.exec() == QDialog::Accepted)
        {
            m_pmodel->queue_connect(dialog.ip());
        }
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
        auto cred = m_pmodel->cred();
        if (cred.username.isEmpty())
        {
            m_user_label.setText({});
        }
        else
        {
            m_user_label.setText(QStringLiteral(u"用户：%1").arg(cred.username));
        }
    }

    void SettingsPage::update_online()
    {
        auto users = m_pmodel->onlines();
        m_online_table.clearContents();
        m_online_table.setRowCount((int)users.size());
        int row = 0;
        for (auto& u : users)
        {
            auto address = new QTableWidgetItem(u.address.toString());
            address->setTextAlignment(Qt::AlignCenter);
            m_online_table.setItem(row, 0, address);

            auto login_time = new QTableWidgetItem(format_datetime(u.login_time));
            login_time->setTextAlignment(Qt::AlignCenter);
            m_online_table.setItem(row, 1, login_time);

            auto flux = new QTableWidgetItem(u.flux.toString());
            flux->setTextAlignment(Qt::AlignCenter);
            m_online_table.setItem(row, 2, flux);

            auto mac_address = new QTableWidgetItem(u.mac_address ? u.mac_address->toString() : QString{});
            mac_address->setTextAlignment(Qt::AlignCenter);
            m_online_table.setItem(row, 3, mac_address);

            auto device = new QTableWidgetItem(u.is_local ? QStringLiteral(u"本机") : QStringLiteral(u"未知"));
            device->setTextAlignment(Qt::AlignCenter);
            m_online_table.setItem(row, 4, device);

            row++;
        }
    }

    void SettingsPage::update_online_busy()
    {
        m_refresh_button.setEnabled(!m_pmodel->online_busy());
    }

    void SettingsPage::delete_cred_and_exit()
    {
        QMessageBox box{ QMessageBox::Warning, QStringLiteral(u"删除凭据并退出"), QStringLiteral(u"是否删除保存的设置文件与凭据？"), QMessageBox::NoButton, this };
        box.setInformativeText(QStringLiteral(u"删除后程序将会退出。"));
        box.addButton(QStringLiteral(u"是"), QMessageBox::AcceptRole);
        box.addButton(QStringLiteral(u"否"), QMessageBox::RejectRole);
        if (box.exec() == QMessageBox::AcceptRole)
        {
            m_pmodel->set_del_at_exit();
            QApplication::exit();
        }
    }
} // namespace TUNet
