#include <ConnectIPDialog.hpp>
#include <QRegularExpressionValidator>

namespace TUNet
{
    ConnectIPDialog::ConnectIPDialog(QWidget* parent) : QDialog(parent)
    {
        setWindowTitle(QStringLiteral(u"认证IP"));
        setWindowFlags(Qt::Dialog | Qt::MSWindowsFixedSizeDialogHint | Qt::WindowTitleHint | Qt::WindowSystemMenuHint | Qt::WindowCloseButtonHint);

        m_ip_edit.setPlaceholderText(QStringLiteral(u"IPv4地址"));
        QObject::connect(&m_ip_edit, &QLineEdit::textChanged, this, &ConnectIPDialog::text_changed);
        m_root_layout.addWidget(&m_ip_edit);

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

    ConnectIPDialog::~ConnectIPDialog() {}

    void ConnectIPDialog::text_changed(const QString& str)
    {
        static QRegularExpression reg{ QStringLiteral(u"^((2[0-4]\\d|25[0-5]|[01]?\\d\\d?)\\.){3}(2[0-4]\\d|25[0-5]|[01]?\\d\\d?)$") };
        m_ok_button.setEnabled(reg.match(str).hasMatch());
    }

    Ipv4Addr ConnectIPDialog::ip() const
    {
        return Ipv4Addr{ m_ip_edit.text() };
    }
} // namespace TUNet
