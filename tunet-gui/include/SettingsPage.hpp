#pragma once

#include <Model.hpp>
#include <QGridLayout>
#include <QHBoxLayout>
#include <QLabel>
#include <QProgressIndicator.hpp>
#include <QPushButton>
#include <QTableWidget>
#include <QVBoxLayout>
#include <QWidget>

namespace TUNet
{
    struct SettingsPage : QWidget
    {
    public:
        SettingsPage(QWidget* parent, Model* pmodel);
        ~SettingsPage() override;

        void ask_credential(const Credential& cred);
        void set_credential();

        void selection_changed();

        void connect_ip();
        void drop_ip();

        void refresh_online();

        void update_cred();
        void update_online();
        void update_online_busy();

        void delete_cred_and_exit();

    private:
        Model* m_pmodel{};

        QVBoxLayout m_settings_layout{ this };

        QLabel m_user_title_label{};
        QHBoxLayout m_user_layout{};
        QLabel m_user_label{};
        QPushButton m_user_button{};
        QPushButton m_del_exit_button{};

        QLabel m_status_title_label{};
        QLabel m_status_label{};

        QLabel m_online_label{};
        QGridLayout m_online_table_layout{};
        QTableWidget m_online_table{};
        QProgressIndicator m_online_busy_indicator{};

        QHBoxLayout m_command_layout{};
        QPushButton m_connect_button{};
        QPushButton m_drop_button{};
        QPushButton m_refresh_button{};
    };
} // namespace TUNet
