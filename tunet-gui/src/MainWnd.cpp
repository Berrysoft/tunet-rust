#include <MainWnd.hpp>

MainWnd::MainWnd() : QMainWindow()
{
    m_flux_layout.setAlignment(Qt::AlignCenter);
    m_flux_layout.addStretch();
    m_flux_layout.addWidget(&m_username_label, Qt::AlignLeft);
    m_flux_layout.addWidget(&m_flux_label, Qt::AlignLeft);
    m_flux_layout.addWidget(&m_online_time_label, Qt::AlignLeft);
    m_flux_layout.addWidget(&m_balance_label, Qt::AlignLeft);
    m_flux_layout.addStretch();
    m_flux_widget.setLayout(&m_flux_layout);

    m_info_layout.addWidget(&m_flux_widget, 0, 0, Qt::AlignCenter);

    m_root_layout.addLayout(&m_info_layout);

    m_login_button.setText(u8"登录");
    m_logout_button.setText(u8"注销");
    m_flux_button.setText(u8"刷新");

    m_command_layout.addWidget(&m_login_button);
    m_command_layout.addWidget(&m_logout_button);
    m_command_layout.addWidget(&m_flux_button);
    m_root_layout.addLayout(&m_command_layout);

    m_root_widget.setLayout(&m_root_layout);
    setCentralWidget(&m_root_widget);

    m_model.queue_state(State::Net);
    QObject::connect(&m_model, &Model::flux_changed, this, &MainWnd::update_flux);
    m_model.queue(Action::Timer);
    m_model.queue(Action::Flux);
}

MainWnd::~MainWnd() {}

void MainWnd::update_flux()
{
    auto flux = m_model.flux();
    m_username_label.setText(QString(u8"用户：%1").arg(tunet_format(flux.username)));
    m_flux_label.setText(QString(u8"流量：%1").arg(tunet_format_flux(flux.flux)));
    m_online_time_label.setText(QString(u8"时长：%1").arg(tunet_format_duration(flux.online_time)));
    m_balance_label.setText(QString(u8"余额：￥%1").arg(flux.balance));
}
