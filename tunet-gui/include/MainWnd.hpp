#pragma once

#include <Model.hpp>
#include <QLabel>
#include <QMainWindow>

struct MainWnd : QMainWindow
{
    Q_OBJECT

private:
    Model m_model{};
    QLabel m_label{};

public:
    MainWnd();

    ~MainWnd() override;

public slots:
    void update_flux();
};
