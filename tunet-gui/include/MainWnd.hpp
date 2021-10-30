#pragma once

#include <FluxCircle.hpp>
#include <Model.hpp>
#include <QComboBox>
#include <QGridLayout>
#include <QLabel>
#include <QMainWindow>
#include <QPushButton>
#include <QVBoxLayout>

struct MainWnd : QMainWindow
{
    Q_OBJECT

public:
    MainWnd();
    ~MainWnd() override;

protected:
    void showEvent(QShowEvent* event) override;

public slots:
    void spawn_login();
    void spawn_logout();
    void spawn_flux();

    void update_state();
    void update_state_back(int index);
    void update_log();
    void update_flux();

private:
    Model m_model{};
    QWidget m_root_widget{};
    QVBoxLayout m_root_layout{};
    QGridLayout m_info_layout{};

    FluxCircle m_flux_circle{};

    QWidget m_flux_widget{};
    QVBoxLayout m_flux_layout{};
    QLabel m_username_label{};
    QLabel m_flux_label{};
    QLabel m_online_time_label{};
    QLabel m_balance_label{};

    QComboBox m_state_combo{};

    QLabel m_log_label{};

    QHBoxLayout m_command_layout{};
    QPushButton m_login_button{};
    QPushButton m_logout_button{};
    QPushButton m_flux_button{};
};
