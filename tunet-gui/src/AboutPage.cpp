#include <AboutPage.hpp>
#include <Model.hpp>
#include <QHeaderView>
#include <QStandardItem>

namespace TUNet
{
    static const QString LIBS[][2] = {
        { u"anyhow"_qs, u"MIT"_qs },
        { u"async-stream"_qs, u"MIT"_qs },
        { u"async-trait"_qs, u"MIT"_qs },
        { u"chrono"_qs, u"MIT"_qs },
        { u"crossterm"_qs, u"MIT"_qs },
        { u"data-encoding"_qs, u"MIT"_qs },
        { u"data-encoding-macro"_qs, u"MIT"_qs },
        { u"dirs"_qs, u"MIT"_qs },
        { u"futures-core"_qs, u"MIT"_qs },
        { u"futures-util"_qs, u"MIT"_qs },
        { u"hmac"_qs, u"MIT"_qs },
        { u"itertools"_qs, u"MIT"_qs },
        { u"keyutils"_qs, u"BSD-3-Clause"_qs },
        { u"lazy_static"_qs, u"MIT"_qs },
        { u"libc"_qs, u"MIT"_qs },
        { u"libloading"_qs, u"ISC"_qs },
        { u"mac_address"_qs, u"MIT"_qs },
        { u"md-5"_qs, u"MIT"_qs },
        { u"netlink_wi"_qs, u"MIT"_qs },
        { u"objc"_qs, u"MIT"_qs },
        { u"once_cell"_qs, u"MIT"_qs },
        { u"regex"_qs, u"MIT"_qs },
        { u"reqwest"_qs, u"MIT"_qs },
        { u"rpassword"_qs, u"Apache-2.0"_qs },
        { u"security-framework"_qs, u"MIT"_qs },
        { u"select"_qs, u"MIT"_qs },
        { u"serde"_qs, u"MIT"_qs },
        { u"serde_json"_qs, u"MIT"_qs },
        { u"sha-1"_qs, u"MIT"_qs },
        { u"structopt"_qs, u"MIT"_qs },
        { u"termcolor"_qs, u"MIT"_qs },
        { u"termcolor_output"_qs, u"MIT"_qs },
        { u"thiserror"_qs, u"MIT"_qs },
        { u"tokio"_qs, u"MIT"_qs },
        { u"trait_enum"_qs, u"MIT"_qs },
        { u"tui"_qs, u"MIT"_qs },
        { u"url"_qs, u"MIT"_qs },
        { u"wide-literials"_qs, u"Unlicense"_qs },
        { u"widestring"_qs, u"MIT"_qs },
        { u"windows"_qs, u"MIT"_qs },
        { u"Qt"_qs, u"LGPLv3"_qs }
    };

    AboutPage::AboutPage(QWidget* parent) : QWidget(parent)
    {
        QFont title_font = m_title_label.font();
        title_font.setBold(true);
        title_font.setPointSizeF(title_font.pointSizeF() * 1.5);

        m_title_label.setFont(title_font);
        m_title_label.setAlignment(Qt::AlignHCenter);
        m_title_label.setText(u"清华大学校园网客户端"_qs);
        m_about_layout.addWidget(&m_title_label);

        auto accent = accent_color();
        m_source_label.setAlignment(Qt::AlignHCenter);
        m_source_label.setText(u"版本 " TUNET_VERSION uR"#( <a href="https://github.com/Berrysoft/tunet-rust" style="color:#%1">项目地址</a>)#"_qs.arg(accent.rgb(), 6, 16, QChar(u'0')));
        m_source_label.setOpenExternalLinks(true);
        m_about_layout.addWidget(&m_source_label);

        m_copyright_label.setAlignment(Qt::AlignHCenter);
        m_copyright_label.setText(u"版权所有 © 2021 Berrysoft"_qs);
        m_about_layout.addWidget(&m_copyright_label);

        m_lib_label.setFont(title_font);
        m_lib_label.setAlignment(Qt::AlignHCenter);
        m_lib_label.setText(u"使用的库"_qs);
        m_about_layout.addWidget(&m_lib_label);

        m_lib_table.setColumnCount(2);
        m_lib_table.setHorizontalHeaderLabels({ u"名称"_qs, u"许可证"_qs });
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
    }

    AboutPage::~AboutPage() {}
} // namespace TUNet
