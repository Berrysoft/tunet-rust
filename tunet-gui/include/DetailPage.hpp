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

    private:
        Model* m_pmodel{};

        QVBoxLayout m_details_layout{ this };
        QPushButton m_refresh_button{ this };
        QTableWidget m_details_table{ this };
    };
} // namespace TUNet
