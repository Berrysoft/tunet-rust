#pragma once

#include <QLabel>
#include <QTableWidget>
#include <QVBoxLayout>
#include <QWidget>

struct AboutPage : QWidget
{
public:
    AboutPage(QWidget* parent);
    ~AboutPage() override;

private:
    QVBoxLayout m_about_layout{ this };
    QLabel m_title_label{ this };
    QLabel m_copyright_label{ this };

    QLabel m_lib_label{ this };

    QTableWidget m_lib_table{ this };
};
