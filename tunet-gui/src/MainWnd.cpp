#include <MainWnd.hpp>
#include <QScreen>

MainWnd::MainWnd() : QMainWindow()
{
    m_root_tab.addTab(&m_info_page, u"主页"_qs);
    m_root_tab.addTab(&m_chart_page, u"统计"_qs);
    m_root_tab.addTab(&m_detail_page, u"明细"_qs);
    m_root_tab.addTab(&m_settings_page, u"设置"_qs);
    m_root_tab.addTab(&m_about_page, u"关于"_qs);

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

void MainWnd::showEvent(QShowEvent* event)
{
    m_model.queue_read_cred();
    m_model.queue_state(State::Auto);
    m_model.queue(Action::Timer);
}
