#pragma once

#include <InfoPage.hpp>
#include <Model.hpp>
#include <QMainWindow>

struct MainWnd : QMainWindow
{
public:
    MainWnd();
    ~MainWnd() override;

private:
    Model m_model{};

    InfoPage m_info_page{ this, &m_model };
};
