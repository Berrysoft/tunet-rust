#include <MainWnd.hpp>
#include <Model.hpp>
#include <QApplication>

using namespace TUNet;

int main_impl(int argc, char** argv, Model* pmodel)
{
    QApplication app{ argc, argv };
    app.setApplicationDisplayName(u"清华校园网"_qs);

    MainWnd wnd{ pmodel };
    wnd.show();

    return app.exec();
}

int main(int argc, char** argv)
{
    return Model::start(4, main_impl, argc, argv);
}
