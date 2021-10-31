#include <MainWnd.hpp>
#include <QScreen>

MainWnd::MainWnd() : QMainWindow()
{
    m_root_tab.addTab(&m_info_page, u8"主页");
    m_root_tab.addTab(&m_chart_page, u8"统计");
    m_root_tab.addTab(&m_detail_page, u8"明细");
    m_root_tab.addTab(&m_about_page, u8"关于");

    setCentralWidget(&m_root_tab);

    setMinimumSize(300, 300);
    resize(400, 400);
    move(screen()->geometry().center() - rect().center());

    QPalette pal = palette();
    QColor accent = tunet_accent();
    pal.setColor(QPalette::Highlight, accent);
    setPalette(pal);
}

MainWnd::~MainWnd() {}
