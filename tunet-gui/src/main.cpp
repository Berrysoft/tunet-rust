#include <MainWnd.hpp>
#include <Model.hpp>
#include <QApplication>

int main_impl(int argc, char** argv, TUNet::Model* pmodel)
{
    QApplication app{ argc, argv };
    app.setApplicationDisplayName(u"清华校园网"_qs);

    TUNet::MainWnd wnd{ pmodel };
    wnd.show();

    return app.exec();
}

int main(int argc, char** argv)
{
    return TUNet::start(4, main_impl, argc, argv);
}
