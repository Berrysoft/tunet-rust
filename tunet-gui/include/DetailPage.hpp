#pragma once

#include <Model.hpp>
#include <QTableWidget>
#include <QVBoxLayout>
#include <QWidget>

struct DetailPage : QWidget
{
public:
    DetailPage(QWidget* parent, Model* pmodel);
    ~DetailPage() override;

    void update_details();

private:
    Model* m_pmodel{};

    QVBoxLayout m_details_layout{ this };
    QTableWidget m_details_table{ this };
};
