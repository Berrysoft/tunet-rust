#include <QHeaderView>
#include <SettingsPage.hpp>

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
    m_status_label.setText(tunet_format_status(m_pmodel->status()));
    m_settings_layout.addWidget(&m_status_label);

    m_online_label.setFont(title_font);
    m_online_label.setAlignment(Qt::AlignHCenter);
    m_online_label.setText(u"管理连接"_qs);
    m_settings_layout.addWidget(&m_online_label);

    m_online_table.setColumnCount(5);
    m_online_table.setHorizontalHeaderLabels({ u"IP地址"_qs, u"登录时间"_qs, u"流量"_qs, u"MAC地址"_qs, u"设备"_qs });
    m_online_table.horizontalHeader()->setSectionResizeMode(QHeaderView::Stretch);
    m_online_table.verticalHeader()->setVisible(false);
    m_settings_layout.addWidget(&m_online_table);

    setLayout(&m_settings_layout);

    QObject::connect(m_pmodel, &Model::cred_changed, this, &SettingsPage::update_cred);
}

SettingsPage::~SettingsPage() {}

void SettingsPage::update_cred()
{
    m_user_label.setText(m_pmodel->cred().username);
}
