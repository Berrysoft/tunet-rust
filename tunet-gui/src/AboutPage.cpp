#include <AboutPage.hpp>
#include <Model.hpp>
#include <QHeaderView>
#include <QMessageBox>

namespace TUNet
{
    static const QString LIBS[][2] = {
        { QStringLiteral(u"anyhow"), QStringLiteral(u"MIT") },
        { QStringLiteral(u"async-stream"), QStringLiteral(u"MIT") },
        { QStringLiteral(u"async-trait"), QStringLiteral(u"MIT") },
        { QStringLiteral(u"chrono"), QStringLiteral(u"MIT") },
        { QStringLiteral(u"clap"), QStringLiteral(u"Apache-2.0") },
        { QStringLiteral(u"crossterm"), QStringLiteral(u"MIT") },
        { QStringLiteral(u"data-encoding"), QStringLiteral(u"MIT") },
        { QStringLiteral(u"data-encoding-macro"), QStringLiteral(u"MIT") },
        { QStringLiteral(u"dirs"), QStringLiteral(u"MIT") },
        { QStringLiteral(u"enum_dispatch"), QStringLiteral(u"MIT") },
        { QStringLiteral(u"futures-core"), QStringLiteral(u"MIT") },
        { QStringLiteral(u"futures-util"), QStringLiteral(u"MIT") },
        { QStringLiteral(u"hmac"), QStringLiteral(u"MIT") },
        { QStringLiteral(u"itertools"), QStringLiteral(u"MIT") },
        { QStringLiteral(u"keyutils"), QStringLiteral(u"BSD-3-Clause") },
        { QStringLiteral(u"lazy_static"), QStringLiteral(u"MIT") },
        { QStringLiteral(u"libc"), QStringLiteral(u"MIT") },
        { QStringLiteral(u"libloading"), QStringLiteral(u"ISC") },
        { QStringLiteral(u"mac_address"), QStringLiteral(u"MIT") },
        { QStringLiteral(u"md-5"), QStringLiteral(u"MIT") },
        { QStringLiteral(u"netlink_wi"), QStringLiteral(u"MIT") },
        { QStringLiteral(u"objc"), QStringLiteral(u"MIT") },
        { QStringLiteral(u"QProgressIndicator"), QStringLiteral(u"MIT") },
        { QStringLiteral(u"regex"), QStringLiteral(u"MIT") },
        { QStringLiteral(u"reqwest"), QStringLiteral(u"MIT") },
        { QStringLiteral(u"rpassword"), QStringLiteral(u"Apache-2.0") },
        { QStringLiteral(u"security-framework"), QStringLiteral(u"MIT") },
        { QStringLiteral(u"select"), QStringLiteral(u"MIT") },
        { QStringLiteral(u"serde"), QStringLiteral(u"MIT") },
        { QStringLiteral(u"serde_json"), QStringLiteral(u"MIT") },
        { QStringLiteral(u"sha-1"), QStringLiteral(u"MIT") },
        { QStringLiteral(u"termcolor"), QStringLiteral(u"MIT") },
        { QStringLiteral(u"termcolor_output"), QStringLiteral(u"MIT") },
        { QStringLiteral(u"thiserror"), QStringLiteral(u"MIT") },
        { QStringLiteral(u"tokio"), QStringLiteral(u"MIT") },
        { QStringLiteral(u"tui"), QStringLiteral(u"MIT") },
        { QStringLiteral(u"url"), QStringLiteral(u"MIT") },
        { QStringLiteral(u"widestring"), QStringLiteral(u"MIT") },
        { QStringLiteral(u"windows"), QStringLiteral(u"MIT") },
    };

    AboutPage::AboutPage(QWidget* parent) : QWidget(parent)
    {
        m_title_label.setAlignment(Qt::AlignHCenter);
        m_title_label.setText(QStringLiteral(u"<b><big>清华大学校园网客户端</big></b>"));
        m_about_layout.addWidget(&m_title_label);

        m_source_label.setAlignment(Qt::AlignHCenter);
        m_source_label.setText(QStringLiteral(u"版本 " TUNET_VERSION uR"( <a href="https://github.com/Berrysoft/tunet-rust">项目地址</a>)"));
        m_source_label.setOpenExternalLinks(true);
        m_about_layout.addWidget(&m_source_label);

        m_copyright_label.setAlignment(Qt::AlignHCenter);
        m_copyright_label.setText(QStringLiteral(u"版权所有 © 2021 Berrysoft"));
        m_about_layout.addWidget(&m_copyright_label);

        m_dial_label.setAlignment(Qt::AlignHCenter);
        m_dial_label.setText(QStringLiteral(u"服务热线（8:00~20:00）"
                                            uR"(<a href="tel:010-62784859">010-62784859</a>)"));
        m_dial_label.setOpenExternalLinks(true);
        m_about_layout.addWidget(&m_dial_label);

        m_lib_label.setAlignment(Qt::AlignHCenter);
        m_lib_label.setText(QStringLiteral(u"<b><big>使用的库</big></b>"));
        m_about_layout.addWidget(&m_lib_label);

        m_lib_table.setColumnCount(2);
        m_lib_table.setHorizontalHeaderLabels({ QStringLiteral(u"名称"), QStringLiteral(u"许可证") });
        m_lib_table.setRowCount((int)std::size(LIBS));
        int row = 0;
        for (auto& lib : LIBS)
        {
            auto name = new QTableWidgetItem(lib[0]);
            name->setTextAlignment(Qt::AlignCenter);
            m_lib_table.setItem(row, 0, name);

            auto license = new QTableWidgetItem(lib[1]);
            license->setTextAlignment(Qt::AlignCenter);
            m_lib_table.setItem(row, 1, license);

            row++;
        }

        m_lib_table.horizontalHeader()->setSectionResizeMode(QHeaderView::Stretch);
        m_lib_table.verticalHeader()->setVisible(false);
        m_lib_table.setSortingEnabled(true);
        m_lib_table.setSelectionBehavior(QTableWidget::SelectRows);
        m_about_layout.addWidget(&m_lib_table);

        m_about_qt_button.setText(QStringLiteral(u"关于Qt"));
        QObject::connect(&m_about_qt_button, &QPushButton::clicked, this, &AboutPage::about_qt);
        m_about_layout.addWidget(&m_about_qt_button);
    }

    AboutPage::~AboutPage() {}

    void AboutPage::about_qt()
    {
        QMessageBox::aboutQt(this);
    }
} // namespace TUNet
