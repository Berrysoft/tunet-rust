#include <MainWnd.hpp>
#include <QScreen>

namespace TUNet
{
    MainWnd::MainWnd(Model* pmodel) : QMainWindow(), m_pmodel(pmodel)
    {
        setWindowTitle(QStringLiteral(u"清华校园网"));
        setWindowIcon(QIcon(QStringLiteral(u":logo.ico")));

        m_root_tab.addTab(&m_info_page, QStringLiteral(u"主页"));
        m_root_tab.addTab(&m_chart_page, QStringLiteral(u"统计"));
        m_root_tab.addTab(&m_detail_page, QStringLiteral(u"明细"));
        m_root_tab.addTab(&m_settings_page, QStringLiteral(u"设置"));
        m_root_tab.addTab(&m_about_page, QStringLiteral(u"关于"));

        setCentralWidget(&m_root_tab);

        setMinimumSize(300, 300);
        resize(500, 500);

#if QT_VERSION >= QT_VERSION_CHECK(5, 14, 0)
        move(screen()->geometry().center() - rect().center());
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
