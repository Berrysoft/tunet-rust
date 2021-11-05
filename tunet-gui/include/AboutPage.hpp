#pragma once

#include <QLabel>
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

    private:
        QVBoxLayout m_about_layout{ this };
        QLabel m_title_label{};
        QLabel m_source_label{};
        QLabel m_copyright_label{};

        QLabel m_lib_label{};
        QTableWidget m_lib_table{};
    };
} // namespace TUNet
