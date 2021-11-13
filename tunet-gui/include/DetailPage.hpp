#pragma once

#include <Model.hpp>
#include <QGridLayout>
#include <QProgressIndicator.hpp>
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
        QGridLayout m_details_table_layout{};
        QTableWidget m_details_table{};
        QProgressIndicator m_detail_busy_indicator{};
        QPushButton m_refresh_button{};
    };
} // namespace TUNet
