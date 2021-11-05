#include <CredDialog.hpp>

namespace TUNet
{
    CredDialog::CredDialog() : QDialog()
    {
        m_username_edit.setPlaceholderText(u"用户名"_qs);
        QObject::connect(&m_username_edit, &QLineEdit::textChanged, this, &CredDialog::text_changed);
        m_root_layout.addWidget(&m_username_edit);
        m_password_edit.setPlaceholderText(u"密码"_qs);
        m_password_edit.setEchoMode(QLineEdit::Password);
        QObject::connect(&m_password_edit, &QLineEdit::textChanged, this, &CredDialog::text_changed);
        m_root_layout.addWidget(&m_password_edit);

        m_cancel_button.setText(u"取消"_qs);
        QObject::connect(&m_cancel_button, &QPushButton::clicked, this, &QDialog::reject);
        m_command_layout.addWidget(&m_cancel_button);
        m_ok_button.setText(u"确定"_qs);
        m_ok_button.setDefault(true);
        m_ok_button.setEnabled(false);
        QObject::connect(&m_ok_button, &QPushButton::clicked, this, &QDialog::accept);
        m_command_layout.addWidget(&m_ok_button);
        m_root_layout.addLayout(&m_command_layout);
    }

    CredDialog::~CredDialog() {}

    void CredDialog::text_changed(const QString& str)
    {
        m_ok_button.setEnabled(!m_username_edit.text().isEmpty() && !m_password_edit.text().isEmpty());
    }

    void CredDialog::set_credential(const Credential& cred)
    {
        m_username_edit.setText(cred.username);
        m_password_edit.setText(cred.password);
    }

    Credential CredDialog::credential() const
    {
        return { m_username_edit.text(), m_password_edit.text() };
    }
} // namespace TUNet
