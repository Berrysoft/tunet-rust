#include <MainWnd.hpp>
#include <QScreen>

MainWnd::MainWnd() : QMainWindow()
{
    m_flux_layout.addWidget(&m_username_label, Qt::AlignLeft);
    m_flux_layout.addWidget(&m_flux_label, Qt::AlignLeft);
    m_flux_layout.addWidget(&m_online_time_label, Qt::AlignLeft);
    m_flux_layout.addWidget(&m_balance_label, Qt::AlignLeft);

    m_info_layout.addWidget(&m_flux_circle, 0, 0);
    m_info_layout.addWidget(&m_flux_widget, 0, 0, Qt::AlignCenter);
    m_root_layout.addLayout(&m_info_layout, 1);

    m_state_layout.addStretch();
    m_state_label.setText(u8"连接方式：");
    m_state_layout.addWidget(&m_state_label);
    m_state_combo.addItem("Net");
    m_state_combo.addItem("Auth4");
    m_state_combo.addItem("Auth6");
    QObject::connect(&m_state_combo, &QComboBox::currentIndexChanged, this, &MainWnd::update_state_back);
    m_state_layout.addWidget(&m_state_combo);
    m_state_layout.addStretch();
    m_root_layout.addWidget(&m_state_widget);

    m_log_label.setTextInteractionFlags(Qt::TextSelectableByMouse);
    m_log_label.setWordWrap(true);
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

    setCentralWidget(&m_root_widget);

    resize(400, 400);
    move(screen()->geometry().center() - rect().center());

    QObject::connect(&m_model, &Model::state_changed, this, &MainWnd::update_state);
    QObject::connect(&m_model, &Model::log_changed, this, &MainWnd::update_log);
    QObject::connect(&m_model, &Model::flux_changed, this, &MainWnd::update_flux);
}

MainWnd::~MainWnd() {}

void MainWnd::showEvent(QShowEvent* event)
{
    m_model.queue_read_cred();
    m_model.queue_state(State::Auto);
    m_model.queue(Action::Timer);
}

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

void MainWnd::update_state()
{
    auto state = m_model.state();
    m_state_combo.setCurrentIndex(static_cast<int>(state) - 1);
    m_model.queue(Action::Flux);
}

void MainWnd::update_state_back(int index)
{
    m_model.queue_state(static_cast<State>(index + 1));
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
