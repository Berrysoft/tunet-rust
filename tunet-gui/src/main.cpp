#include <MainWnd.hpp>
#include <Model.hpp>
#include <QApplication>

int main_impl(int argc, char** argv)
{
    QApplication app{ argc, argv };
    app.setApplicationDisplayName(u8"清华校园网");

    MainWnd wnd{};
    wnd.show();

    return app.exec();
}

int main(int argc, char** argv)
{
    return tunet_start(4, main_impl, argc, argv);
}
