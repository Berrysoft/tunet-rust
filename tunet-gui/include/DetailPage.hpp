#pragma once

#include <Model.hpp>
#include <QPushButton>
#include <QTableWidget>
#include <QVBoxLayout>
#include <QWidget>

namespace TUNet
{
    struct DetailPage : QWidget
    {
    public:
        DetailPage(QWidget* parent, Model* pmodel);
        ~DetailPage() override;

        void refresh_details();
        void update_details();
        void update_detail_busy();

    private:
        Model* m_pmodel{};

        QVBoxLayout m_details_layout{ this };
        QTableWidget m_details_table{};
        QPushButton m_refresh_button{};
    };
} // namespace TUNet
