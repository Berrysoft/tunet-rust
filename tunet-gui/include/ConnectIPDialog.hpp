#pragma once

#include <Model.hpp>
#include <QDialog>
#include <QHBoxLayout>
#include <QLineEdit>
#include <QPushButton>
#include <QVBoxLayout>

namespace TUNet
{
    struct ConnectIPDialog : QDialog
    {
    public:
        ConnectIPDialog(QWidget* parent = nullptr);
        ~ConnectIPDialog();

        void text_changed(const QString& str);

        Ipv4Addr ip() const;

    private:
        QVBoxLayout m_root_layout{ this };

        QLineEdit m_ip_edit{};

        QHBoxLayout m_command_layout{};
        QPushButton m_cancel_button{};
        QPushButton m_ok_button{};
    };
} // namespace TUNet
