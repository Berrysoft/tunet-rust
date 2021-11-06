#include <ConnectIPDialog.hpp>
#include <QRegularExpressionValidator>

namespace TUNet
{
    ConnectIPDialog::ConnectIPDialog() : QDialog()
    {
        setWindowTitle(u"认证IP"_qs);
        setWindowFlags(Qt::Dialog | Qt::CustomizeWindowHint | Qt::WindowTitleHint);

        m_ip_edit.setPlaceholderText(u"IPv4地址"_qs);
        QObject::connect(&m_ip_edit, &QLineEdit::textChanged, this, &ConnectIPDialog::text_changed);
        m_root_layout.addWidget(&m_ip_edit);

        m_cancel_button.setText(u"取消"_qs);
        QObject::connect(&m_cancel_button, &QPushButton::clicked, this, &QDialog::reject);
        m_command_layout.addWidget(&m_cancel_button);
        m_ok_button.setText(u"确定"_qs);
        m_ok_button.setDefault(true);
        m_ok_button.setEnabled(false);
        QObject::connect(&m_ok_button, &QPushButton::clicked, this, &QDialog::accept);
        m_command_layout.addWidget(&m_ok_button);
        m_root_layout.addLayout(&m_command_layout);

        setFixedSize(sizeHint());
    }

    ConnectIPDialog::~ConnectIPDialog() {}

    void ConnectIPDialog::text_changed(const QString& str)
    {
        static QRegularExpression reg{ "^((2[0-4]\\d|25[0-5]|[01]?\\d\\d?)\\.){3}(2[0-4]\\d|25[0-5]|[01]?\\d\\d?)$" };
        m_ok_button.setEnabled(reg.match(str).hasMatch());
    }

    Ipv4Addr ConnectIPDialog::ip() const
    {
        return Ipv4Addr{ m_ip_edit.text() };
    }
} // namespace TUNet
