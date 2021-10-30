#include <MainWnd.hpp>
#include <QScreen>

MainWnd::MainWnd() : QMainWindow()
{
    setCentralWidget(&m_info_page);

    setMinimumSize(300, 300);
    resize(400, 400);
    move(screen()->geometry().center() - rect().center());
}

MainWnd::~MainWnd() {}
