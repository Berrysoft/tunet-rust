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

    setLayout(&m_settings_layout);

    QObject::connect(m_pmodel, &Model::cred_changed, this, &SettingsPage::update_cred);
}

SettingsPage::~SettingsPage() {}

void SettingsPage::update_cred()
{
    m_user_label.setText(m_pmodel->cred().username);
}
