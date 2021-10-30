#pragma once

#include <FluxCircle.hpp>
#include <Model.hpp>
#include <QComboBox>
#include <QGridLayout>
#include <QHBoxLayout>
#include <QLabel>
#include <QPushButton>
#include <QVBoxLayout>
#include <QWidget>

struct InfoPage : QWidget
{
public:
    InfoPage(QWidget* parent, Model* pmodel);
    ~InfoPage() override;

    void spawn_login();
    void spawn_logout();
    void spawn_flux();

    void update_state();
    void update_state_back(int index);
    void update_log();
    void update_flux();

private:
    Model* m_pmodel{};

    QVBoxLayout m_root_layout{ this };

    // basic info
    QGridLayout m_info_layout{};

    FluxCircle m_flux_circle{};

    QWidget m_flux_widget{};
    QVBoxLayout m_flux_layout{ &m_flux_widget };
    QLabel m_username_label{};
    QLabel m_flux_label{};
    QLabel m_online_time_label{};
    QLabel m_balance_label{};

    // state
    QWidget m_state_widget{};
    QHBoxLayout m_state_layout{ &m_state_widget };
    QLabel m_state_label{};
    QComboBox m_state_combo{};

    // log
    QLabel m_log_label{};

    // command
    QHBoxLayout m_command_layout{};
    QPushButton m_login_button{};
    QPushButton m_logout_button{};
    QPushButton m_flux_button{};
};
