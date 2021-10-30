#pragma once

#include <QLabel>
#include <QStandardItemModel>
#include <QTableView>
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

    QStandardItemModel m_libs{ this };
    QTableView m_lib_table{ this };
};
