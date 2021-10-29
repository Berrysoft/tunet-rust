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

    m_info_layout.addWidget(&m_flux_circle, 0, 0);
    m_info_layout.addWidget(&m_flux_widget, 0, 0, Qt::AlignCenter);

    m_root_layout.addLayout(&m_info_layout, 1);

    m_log_label.setAlignment(Qt::AlignCenter);
    m_root_layout.addWidget(&m_log_label);

    m_login_button.setText(u8"登录");
    QObject::connect(&m_login_button, &QPushButton::clicked, this, &MainWnd::spawn_login);
    m_logout_button.setText(u8"注销");
    QObject::connect(&m_logout_button, &QPushButton::clicked, this, &MainWnd::spawn_logout);
    m_flux_button.setText(u8"刷新");
    QObject::connect(&m_flux_button, &QPushButton::clicked, this, &MainWnd::spawn_flux);

    m_command_layout.addWidget(&m_login_button);
    m_command_layout.addWidget(&m_logout_button);
    m_command_layout.addWidget(&m_flux_button);
    m_root_layout.addLayout(&m_command_layout, 1);

    m_root_widget.setLayout(&m_root_layout);
    setCentralWidget(&m_root_widget);

    m_model.queue_state(State::Net);
    QObject::connect(&m_model, &Model::log_changed, this, &MainWnd::update_log);
    QObject::connect(&m_model, &Model::flux_changed, this, &MainWnd::update_flux);
    m_model.queue(Action::Timer);
    m_model.queue(Action::Flux);
}

MainWnd::~MainWnd() {}

void MainWnd::spawn_login()
{
    m_model.queue(Action::Login);
}

void MainWnd::spawn_logout()
{
    m_model.queue(Action::Logout);
}

void MainWnd::spawn_flux()
{
    m_model.queue(Action::Flux);
}

void MainWnd::update_log()
{
    m_log_label.setText(m_model.log());
}

void MainWnd::update_flux()
{
    auto flux = m_model.flux();
    m_username_label.setText(QString(u8"用户：%1").arg(flux.username));
    m_flux_label.setText(QString(u8"流量：%1").arg(tunet_format_flux(flux.flux)));
    m_online_time_label.setText(QString(u8"时长：%1").arg(tunet_format_duration(flux.online_time)));
    m_balance_label.setText(QString(u8"余额：￥%1").arg(flux.balance));
    m_flux_circle.update_flux(flux.flux, flux.balance);
}
