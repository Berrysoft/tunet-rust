#include <MainWnd.hpp>
#include <QScreen>

namespace TUNet
{
    MainWnd::MainWnd(Model* pmodel) : QMainWindow(), m_pmodel(pmodel)
    {
        setWindowIcon(QIcon(u":logo.ico"_qs));

        m_root_tab.addTab(&m_info_page, u"主页"_qs);
        m_root_tab.addTab(&m_chart_page, u"统计"_qs);
        m_root_tab.addTab(&m_detail_page, u"明细"_qs);
        m_root_tab.addTab(&m_settings_page, u"设置"_qs);
        m_root_tab.addTab(&m_about_page, u"关于"_qs);

        setCentralWidget(&m_root_tab);

        setMinimumSize(300, 300);
        resize(500, 500);

#if QT_VERSION >= QT_VERSION_CHECK(5, 14, 0)
        move(screen()->geometry().center() - rect().center());
#endif

#ifdef Q_OS_WIN
        QPalette pal = palette();
        QColor accent = accent_color();
        pal.setColor(QPalette::Highlight, accent);
        setPalette(pal);
#endif

        QObject::connect(m_pmodel, &Model::cred_changed, this, &MainWnd::update_cred);
    }

    MainWnd::~MainWnd() {}

    void MainWnd::showEvent(QShowEvent* event)
    {
        m_pmodel->queue_cred_load();
        m_pmodel->queue(Action::Timer);
    }

    void MainWnd::update_cred()
    {
        m_pmodel->queue_state(State::Auto);
        m_pmodel->queue(Action::Online);
        m_pmodel->queue(Action::Details);
    }
} // namespace TUNet
