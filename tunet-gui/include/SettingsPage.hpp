#pragma once

#include <Model.hpp>
#include <QLabel>
#include <QTableWidget>
#include <QVBoxLayout>
#include <QWidget>

struct SettingsPage : QWidget
{
public:
    SettingsPage(QWidget* parent, Model* pmodel);
    ~SettingsPage() override;

    void update_cred();
    void update_online();

private:
    Model* m_pmodel{};

    QVBoxLayout m_settings_layout{ this };

    QLabel m_user_title_label{ this };
    QLabel m_user_label{ this };

    QLabel m_status_title_label{ this };
    QLabel m_status_label{ this };

    QLabel m_online_label{ this };
    QTableWidget m_online_table{ this };
};
