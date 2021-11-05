#pragma once

#include <Model.hpp>
#include <QDialog>
#include <QHBoxLayout>
#include <QLineEdit>
#include <QPushButton>
#include <QVBoxLayout>

namespace TUNet
{
    struct CredDialog : QDialog
    {
    public:
        CredDialog();
        ~CredDialog();

        void text_changed(const QString& str);

        void set_credential(const Credential& cred);
        Credential credential() const;

    private:
        QVBoxLayout m_root_layout{ this };

        QLineEdit m_username_edit{};
        QLineEdit m_password_edit{};

        QHBoxLayout m_command_layout{};
        QPushButton m_cancel_button{};
        QPushButton m_ok_button{};
    };
} // namespace TUNet
