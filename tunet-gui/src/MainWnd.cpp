#include <MainWnd.hpp>
#include <QScreen>

namespace TUNet
{
    MainWnd::MainWnd(Model* pmodel) : QMainWindow(), m_pmodel(pmodel)
    {
        m_root_tab.addTab(&m_info_page, u"主页"_qs);
        m_root_tab.addTab(&m_chart_page, u"统计"_qs);
        m_root_tab.addTab(&m_detail_page, u"明细"_qs);
        m_root_tab.addTab(&m_settings_page, u"设置"_qs);
        m_root_tab.addTab(&m_about_page, u"关于"_qs);

        setCentralWidget(&m_root_tab);

        setMinimumSize(300, 300);
        resize(500, 500);
        move(screen()->geometry().center() - rect().center());

        QPalette pal = palette();
        QColor accent = accent_color();
        pal.setColor(QPalette::Highlight, accent);
        setPalette(pal);
    }

    MainWnd::~MainWnd() {}

    void MainWnd::showEvent(QShowEvent* event)
    {
        m_pmodel->queue_read_cred();
        m_pmodel->queue(Action::Timer);
        m_pmodel->queue_state(State::Auto);
        m_pmodel->queue(Action::Online);
        m_pmodel->queue(Action::Details);
    }
} // namespace TUNet
