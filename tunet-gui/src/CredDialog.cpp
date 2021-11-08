#include <CredDialog.hpp>

namespace TUNet
{
    CredDialog::CredDialog(QWidget* parent) : QDialog(parent)
    {
        setWindowTitle(QStringLiteral(u"设置凭据"));
        setWindowFlags(Qt::Dialog | Qt::MSWindowsFixedSizeDialogHint | Qt::WindowTitleHint | Qt::WindowSystemMenuHint | Qt::WindowCloseButtonHint);

        m_username_edit.setPlaceholderText(QStringLiteral(u"用户名"));
        QObject::connect(&m_username_edit, &QLineEdit::textChanged, this, &CredDialog::text_changed);
        m_root_layout.addWidget(&m_username_edit);
        m_password_edit.setPlaceholderText(QStringLiteral(u"密码"));
        m_password_edit.setEchoMode(QLineEdit::Password);
        QObject::connect(&m_password_edit, &QLineEdit::textChanged, this, &CredDialog::text_changed);
        m_root_layout.addWidget(&m_password_edit);

        m_ok_button.setText(QStringLiteral(u"确定"));
        m_ok_button.setEnabled(false);
        QObject::connect(&m_ok_button, &QPushButton::clicked, this, &QDialog::accept);
        m_command_layout.addWidget(&m_ok_button);
        m_cancel_button.setText(QStringLiteral(u"取消"));
        QObject::connect(&m_cancel_button, &QPushButton::clicked, this, &QDialog::reject);
        m_command_layout.addWidget(&m_cancel_button);
        m_root_layout.addLayout(&m_command_layout);

        setFixedSize(sizeHint());
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
