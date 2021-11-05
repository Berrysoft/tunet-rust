#pragma once

#include <AboutPage.hpp>
#include <ChartPage.hpp>
#include <DetailPage.hpp>
#include <InfoPage.hpp>
#include <Model.hpp>
#include <QMainWindow>
#include <QTabWidget>
#include <SettingsPage.hpp>

namespace TUNet
{
    struct MainWnd : QMainWindow
    {
    public:
        MainWnd(Model* pmodel);
        ~MainWnd() override;

        void update_cred();

    protected:
        void showEvent(QShowEvent* event) override;

    private:
        Model* m_pmodel{};

        QTabWidget m_root_tab{};
        InfoPage m_info_page{ &m_root_tab, m_pmodel };
        ChartPage m_chart_page{ &m_root_tab, m_pmodel };
        DetailPage m_detail_page{ &m_root_tab, m_pmodel };
        SettingsPage m_settings_page{ &m_root_tab, m_pmodel };
        AboutPage m_about_page{ &m_root_tab };
    };
} // namespace TUNet
