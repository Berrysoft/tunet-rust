#include <AboutPage.hpp>
#include <QHeaderView>
#include <QStandardItem>

static const QString LIBS[][2] = {
    { u8"anyhow", u8"MIT" },
    { u8"async-stream", u8"MIT" },
    { u8"async-trait", u8"MIT" },
    { u8"chrono", u8"MIT" },
    { u8"crossterm", u8"MIT" },
    { u8"data-encoding", u8"MIT" },
    { u8"data-encoding-macro", u8"MIT" },
    { u8"dirs", u8"MIT" },
    { u8"futures-core", u8"MIT" },
    { u8"futures-util", u8"MIT" },
    { u8"hmac", u8"MIT" },
    { u8"itertools", u8"MIT" },
    { u8"keyutils", u8"BSD-3-Clause" },
    { u8"lazy_static", u8"MIT" },
    { u8"libc", u8"MIT" },
    { u8"libloading", u8"ISC" },
    { u8"mac_address", u8"MIT" },
    { u8"md-5", u8"MIT" },
    { u8"netlink_wi", u8"MIT" },
    { u8"objc", u8"MIT" },
    { u8"once_cell", u8"MIT" },
    { u8"regex", u8"MIT" },
    { u8"reqwest", u8"MIT" },
    { u8"rpassword", u8"Apache-2.0" },
    { u8"security-framework", u8"MIT" },
    { u8"select", u8"MIT" },
    { u8"serde", u8"MIT" },
    { u8"serde_json", u8"MIT" },
    { u8"sha-1", u8"MIT" },
    { u8"structopt", u8"MIT" },
    { u8"termcolor", u8"MIT" },
    { u8"termcolor_output", u8"MIT" },
    { u8"thiserror", u8"MIT" },
    { u8"tokio", u8"MIT" },
    { u8"trait_enum", u8"MIT" },
    { u8"tui", u8"MIT" },
    { u8"url", u8"MIT" },
    { u8"wide-literials", u8"Unlicense" },
    { u8"widestring", u8"MIT" },
    { u8"windows", u8"MIT" },
    { u8"Qt", u8"LGPLv3" }
};

AboutPage::AboutPage(QWidget* parent) : QWidget(parent)
{
    QFont title_font = m_title_label.font();
    title_font.setBold(true);
    title_font.setPointSizeF(title_font.pointSizeF() * 1.5);

    m_title_label.setFont(title_font);
    m_title_label.setAlignment(Qt::AlignHCenter);
    m_title_label.setText(u8"清华大学校园网客户端");
    m_about_layout.addWidget(&m_title_label);

    m_copyright_label.setAlignment(Qt::AlignHCenter);
    m_copyright_label.setText(u8"版权所有 © 2021 Berrysoft");
    m_about_layout.addWidget(&m_copyright_label);

    m_lib_label.setFont(title_font);
    m_lib_label.setAlignment(Qt::AlignHCenter);
    m_lib_label.setText(u8"使用的库");
    m_about_layout.addWidget(&m_lib_label);

    m_libs.setHorizontalHeaderLabels(QStringList{ u8"名称", u8"许可证" });
    for (auto& lib : LIBS)
    {
        m_libs.appendRow({ new QStandardItem(lib[0]), new QStandardItem(lib[1]) });
    }

    m_lib_table.setModel(&m_libs);
    m_lib_table.horizontalHeader()->setSectionResizeMode(QHeaderView::Stretch);
    m_about_layout.addWidget(&m_lib_table);

    setLayout(&m_about_layout);
}

AboutPage::~AboutPage() {}
