#include <MainWnd.hpp>
#include <Model.hpp>
#include <QApplication>

using namespace TUNet;

int main_impl(int argc, char** argv, Model* pmodel)
{
    QApplication app{ argc, argv };
    app.setApplicationDisplayName(u"清华校园网"_qs);

#if defined(Q_OS_WIN) && QT_VERSION < QT_VERSION_CHECK(6, 0, 0)
    QFont f = app.font();
    f.setFamily(u"Microsoft YaHei UI"_qs);
    app.setFont(f);
#endif

    MainWnd wnd{ pmodel };
    wnd.show();

    return app.exec();
}

int main(int argc, char** argv)
{
    return Model::start(4, main_impl, argc, argv);
}
