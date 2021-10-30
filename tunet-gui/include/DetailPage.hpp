#pragma once

#include <Model.hpp>
#include <QStandardItemModel>
#include <QTableView>
#include <QVBoxLayout>
#include <QWidget>

struct DetailPage : QWidget
{
public:
    DetailPage(QWidget* parent, Model* pmodel);
    ~DetailPage() override;

    void update_details();

protected:
    void showEvent(QShowEvent* event) override;

private:
    Model* m_pmodel{};

    QVBoxLayout m_details_layout{ this };
    QStandardItemModel m_details{ this };
    QTableView m_details_table{ this };
};
