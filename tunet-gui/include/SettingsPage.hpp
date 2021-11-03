#pragma once

#include <Model.hpp>
#include <QHBoxLayout>
#include <QLabel>
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

        void set_credential();

        void selection_changed();

        void connect_ip();
        void drop_ip();

        void refresh_online();

        void update_cred();
        void update_online();

    private:
        Model* m_pmodel{};

        QVBoxLayout m_settings_layout{ this };

        QLabel m_user_title_label{ this };
        QHBoxLayout m_user_layout{};
        QPushButton m_user_button{ this };

        QLabel m_status_title_label{ this };
        QLabel m_status_label{ this };

        QLabel m_online_label{ this };
        QTableWidget m_online_table{ this };

        QHBoxLayout m_command_layout{};
        QPushButton m_connect_button{ this };
        QPushButton m_drop_button{ this };
        QPushButton m_refresh_button{ this };
    };
} // namespace TUNet
