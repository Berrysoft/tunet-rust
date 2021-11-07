#include <MainWnd.hpp>
#include <Model.hpp>
#include <QApplication>

using namespace TUNet;

int main_impl(int argc, char** argv, Model* pmodel)
{
    QApplication::setApplicationDisplayName(u"清华校园网"_qs);
    QApplication app{ argc, argv };

#if defined(Q_OS_WIN) && QT_VERSION < QT_VERSION_CHECK(6, 0, 0)
    QFont f = app.font();
    f.setFamily(u"Microsoft YaHei UI"_qs);
    app.setFont(f);
#endif

#ifdef Q_OS_WIN
    QPalette pal = QApplication::palette();
    QColor accent = pmodel->accent_color();
    pal.setColor(QPalette::Highlight, accent);
    pal.setColor(QPalette::Link, accent);
    pal.setColor(QPalette::LinkVisited, accent);
    QApplication::setPalette(pal);
#endif

#if defined(Q_OS_MACOS) && QT_VERSION >= QT_VERSION_CHECK(6, 0, 0)
    // Fix for wrong button height when text contains CJK chars,
    // set to a small height and let cocoa to determine the default.
    // Ref commit: https://github.com/qt/qtbase/commit/c6379e34993370e7e2208b51be384b738ce35817
    app.setStyleSheet(u"QPushButton{height:1;}"_qs);
#endif

    MainWnd wnd{ pmodel };
    wnd.show();

    return QApplication::exec();
}

int main(int argc, char** argv)
{
    return Model::start(4, main_impl, argc, argv);
}
