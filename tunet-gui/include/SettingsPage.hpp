#pragma once

#include <Model.hpp>
#include <QLabel>
#include <QVBoxLayout>
#include <QWidget>

struct SettingsPage : QWidget
{
public:
    SettingsPage(QWidget* parent, Model* pmodel);
    ~SettingsPage() override;

    void update_cred();

private:
    Model* m_pmodel{};

    QVBoxLayout m_settings_layout{ this };

    QLabel m_user_title_label{ this };
    QLabel m_user_label{ this };

    QLabel m_status_title_label{ this };
    QLabel m_status_label{ this };
};
