#pragma once

#include <QLabel>
#include <QPushButton>
#include <QTableWidget>
#include <QVBoxLayout>
#include <QWidget>

namespace TUNet
{
    struct AboutPage : QWidget
    {
    public:
        AboutPage(QWidget* parent);
        ~AboutPage() override;

        void about_qt();

    private:
        QVBoxLayout m_about_layout{ this };
        QLabel m_title_label{};
        QLabel m_source_label{};
        QLabel m_copyright_label{};
        QLabel m_dial_label{};

        QLabel m_lib_label{};
        QTableWidget m_lib_table{};

        QPushButton m_about_qt_button{};
    };
} // namespace TUNet
