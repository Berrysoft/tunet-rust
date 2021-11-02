#include <Model.hpp>
#include <cmath>

extern "C"
{
    using TUNet::Action;
    using TUNet::NativeModel;
    using TUNet::State;
    using TUNet::StatusFlag;
    using TUNet::UpdateMsg;

    struct OnlineUser
    {
        std::uint32_t address;
        std::int64_t login_time;
        std::uint64_t flux;
        std::uint8_t mac_address[6];
        bool is_local;
    };

    struct Detail
    {
        std::int64_t login_time;
        std::int64_t logout_time;
        std::uint64_t flux;
    };

    struct DetailGroup
    {
        std::int64_t logout_date;
        std::uint64_t flux;
    };

    struct DetailGroupByTime
    {
        std::uint32_t logout_start_time;
        std::uint64_t flux;
    };

    using MainCallback = int (*)(NativeModel, void*);
    using UpdateCallback = void (*)(UpdateMsg, void*);
    using StringCallback = void (*)(const char16_t*, void*);
    using OnlinesForeachCallback = bool (*)(const OnlineUser*, void*);
    using DetailsForeachCallback = bool (*)(const Detail*, void*);
    using DetailsGroupedForeachCallback = bool (*)(const DetailGroup*, void*);
    using DetailsGroupedByTimeForeachCallback = bool (*)(const DetailGroupByTime*, void*);

    struct ThemeColor
    {
        std::uint8_t r;
        std::uint8_t g;
        std::uint8_t b;
    };

    ThemeColor tunet_color_accent();

    std::int32_t tunet_model_start(std::size_t val, MainCallback main, void* data);
    void tunet_model_set_update_callback(NativeModel m, UpdateCallback update, void* data);
    void tunet_model_queue(NativeModel m, Action a);
    bool tunet_model_queue_cred_load(NativeModel m);
    void tunet_model_queue_cred(NativeModel m, const char16_t* u, const char16_t* p);
    void tunet_model_queue_state(NativeModel m, State s);
    StatusFlag tunet_model_status(NativeModel m, StringCallback f, void* data);
    void tunet_model_cred_username(NativeModel m, StringCallback f, void* data);
    void tunet_model_cred_password(NativeModel m, StringCallback f, void* data);
    State tunet_model_state(NativeModel m);
    void tunet_model_log(NativeModel m, StringCallback f, void* data);
    void tunet_model_flux_username(NativeModel m, StringCallback f, void* data);
    std::uint64_t tunet_model_flux_flux(NativeModel m);
    std::int64_t tunet_model_flux_online_time(NativeModel m);
    double tunet_model_flux_balance(NativeModel m);
    void tunet_model_onlines_foreach(NativeModel m, OnlinesForeachCallback f, void* data);
    void tunet_model_details_foreach(NativeModel m, DetailsForeachCallback f, void* data);
    void tunet_model_details_grouped_foreach(NativeModel m, DetailsGroupedForeachCallback f, void* data);
    void tunet_model_details_grouped_by_time_foreach(NativeModel m, std::uint32_t groups, DetailsGroupedByTimeForeachCallback f, void* data);
}

namespace TUNet
{
    struct init_data
    {
        Model::StartCallback main;
        int argc;
        char** argv;
    };

    static int fn_init_callback(NativeModel handle, void* data)
    {
        auto d = reinterpret_cast<init_data*>(data);
        Model model{ handle };
        return (d->main)(d->argc, d->argv, &model);
    }

    QColor accent_color()
    {
        auto color = tunet_color_accent();
        return QColor::fromRgb(color.r, color.g, color.b);
    }

    QString Flux::toString() const
    {
        double flux = m_value;
        if (flux < 1000.0)
        {
            return u"%1 B"_qs.arg(m_value);
        }
        flux /= 1000.0;
        if (flux < 1000.0)
        {
            return u"%1 K"_qs.arg(flux, 0, 'f', 2);
        }
        flux /= 1000.0;
        if (flux < 1000.0)
        {
            return u"%1 M"_qs.arg(flux, 0, 'f', 2);
        }
        flux /= 1000.0;
        return u"%1 G"_qs.arg(flux, 0, 'f', 2);
    }

    QString format_status(const Status& status)
    {
        switch (status.flag)
        {
        case StatusFlag::Wwan:
            return u"移动流量"_qs;
        case StatusFlag::Wlan:
            return u"无线网络（%1）"_qs.arg(status.ssid);
        case StatusFlag::Lan:
            return u"有线网络"_qs;
        default:
            return u"未知"_qs;
        }
    }

    QString format_duration(std::chrono::seconds s)
    {
        long long total_sec = s.count();
        auto [total_min, sec] = std::div(total_sec, 60ll);
        auto [total_h, min] = std::div(total_min, 60ll);
        auto [day, h] = std::div(total_h, 60ll);
        if (day)
        {
            return u"%1.%2:%3:%4"_qs.arg(day).arg(h, 2, 10, QChar(u'0')).arg(min, 2, 10, QChar(u'0')).arg(sec, 2, 10, QChar(u'0'));
        }
        else
        {
            return u"%1:%2:%3"_qs.arg(h, 2, 10, QChar(u'0')).arg(min, 2, 10, QChar(u'0')).arg(sec, 2, 10, QChar(u'0'));
        }
    }

    QString format_datetime(const QDateTime& time)
    {
        static QStringView DATETIME_FORMAT = u"yyyy-MM-dd hh:mm:ss";
        return time.toString(DATETIME_FORMAT);
    }

    QString format_ip(std::uint32_t addr)
    {
        return u"%1.%2.%3.%4"_qs.arg((addr >> 24) & 0xFF).arg((addr >> 16) & 0xFF).arg((addr >> 8) & 0xFF).arg(addr & 0xFF);
    }

    QString format_mac_address(const std::array<std::uint8_t, 6>& maddr)
    {
        if (maddr == std::array<std::uint8_t, 6>{ 0, 0, 0, 0, 0, 0 })
        {
            return QString{};
        }
        else
        {
            QString fmt = u"%1:%2:%3:%4:%5:%6"_qs;
            for (auto& part : maddr)
            {
                fmt = fmt.arg(part, 2, 16, QChar(u'0'));
            }
            return fmt;
        }
    }

    static void fn_update_callback(UpdateMsg m, void* data)
    {
        auto model = reinterpret_cast<Model*>(data);
        model->update(m);
    }

    std::int32_t Model::start(std::size_t threads, StartCallback main, int argc, char** argv)
    {
        init_data data{ main, argc, argv };
        return tunet_model_start(threads, fn_init_callback, &data);
    }

    Model::Model(NativeModel handle) : QObject(), m_handle(handle)
    {
        tunet_model_set_update_callback(m_handle, fn_update_callback, this);
    }

    Model::~Model() { tunet_model_set_update_callback(m_handle, nullptr, nullptr); }

    void Model::queue(Action a) const { tunet_model_queue(m_handle, a); }

    bool Model::queue_cred_load() const { return tunet_model_queue_cred_load(m_handle); }

    void Model::queue_cred(const Credential& cred) const
    {
        tunet_model_queue_cred(m_handle, QStringView{ cred.username }.utf16(), QStringView{ cred.password }.utf16());
    }

    void Model::queue_state(State s) const { tunet_model_queue_state(m_handle, s); }

    void Model::update(UpdateMsg m) const
    {
        switch (m)
        {
        case UpdateMsg::Credential:
            emit cred_changed();
            break;
        case UpdateMsg::State:
            emit state_changed();
            break;
        case UpdateMsg::Log:
            emit log_changed();
            break;
        case UpdateMsg::Flux:
            emit flux_changed();
            break;
        case UpdateMsg::Online:
            emit onlines_changed();
            break;
        case UpdateMsg::Details:
            emit details_changed();
            break;
        }
    }

    static void fn_string_callback(const char16_t* data, void* d)
    {
        QString* pstr = reinterpret_cast<QString*>(d);
        *pstr = QString::fromUtf16(data);
    }

    template <typename F, typename... Args>
    QString get_q_string(F&& f, Args... args)
    {
        QString str;
        f(std::move(args)..., fn_string_callback, &str);
        return str;
    }

    Status Model::status() const
    {
        QString ssid{};
        StatusFlag flag = tunet_model_status(m_handle, fn_string_callback, &ssid);
        return { flag, std::move(ssid) };
    }

    Credential Model::cred() const
    {
        auto username = get_q_string(tunet_model_cred_username, m_handle);
        auto password = get_q_string(tunet_model_cred_password, m_handle);
        return { std::move(username), std::move(password) };
    }

    State Model::state() const { return tunet_model_state(m_handle); }

    QString Model::log() const { return get_q_string(tunet_model_log, m_handle); }

    Info Model::flux() const
    {
        auto username = get_q_string(tunet_model_flux_username, m_handle);
        auto f = tunet_model_flux_flux(m_handle);
        auto online = tunet_model_flux_online_time(m_handle);
        auto balance = tunet_model_flux_balance(m_handle);
        return Info{ std::move(username), f, std::chrono::seconds{ online }, balance };
    }

    static bool fn_foreach_online(const OnlineUser* u, void* data)
    {
        auto& users = *reinterpret_cast<std::vector<Online>*>(data);
        std::array<std::uint8_t, 6> mac{};
        std::copy(std::begin(u->mac_address), std::end(u->mac_address), mac.begin());
        users.emplace_back(Online{ u->address, QDateTime::fromSecsSinceEpoch(u->login_time, Qt::UTC), u->flux, std::move(mac), u->is_local });
        return true;
    }

    std::vector<Online> Model::onlines() const
    {
        std::vector<Online> users;
        tunet_model_onlines_foreach(m_handle, fn_foreach_online, &users);
        return users;
    }

    static bool fn_foreach_detail(const ::Detail* d, void* data)
    {
        auto& details = *reinterpret_cast<std::vector<Detail>*>(data);
        details.emplace_back(Detail{ QDateTime::fromSecsSinceEpoch(d->login_time, Qt::UTC), QDateTime::fromSecsSinceEpoch(d->logout_time, Qt::UTC), d->flux });
        return true;
    }

    std::vector<Detail> Model::details() const
    {
        std::vector<Detail> details{};
        tunet_model_details_foreach(m_handle, fn_foreach_detail, &details);
        return details;
    }

    static bool fn_foreach_detail_group(const DetailGroup* d, void* data)
    {
        auto& details = *reinterpret_cast<std::map<QDate, Flux>*>(data);
        details.emplace(QDateTime::fromSecsSinceEpoch(d->logout_date, Qt::UTC).date(), d->flux);
        return true;
    }

    std::map<QDate, Flux> Model::details_grouped() const
    {
        std::map<QDate, Flux> details{};
        tunet_model_details_grouped_foreach(m_handle, fn_foreach_detail_group, &details);
        return details;
    }

    static bool fn_foreach_detail_group_by_time(const DetailGroupByTime* d, void* data)
    {
        auto& details = *reinterpret_cast<std::map<std::uint32_t, Flux>*>(data);
        details.emplace(d->logout_start_time, d->flux);
        return true;
    }

    std::map<std::uint32_t, Flux> Model::details_grouped_by_time(std::uint32_t groups) const
    {
        std::map<std::uint32_t, Flux> details{};
        tunet_model_details_grouped_by_time_foreach(m_handle, groups, fn_foreach_detail_group_by_time, &details);
        return details;
    }
} // namespace TUNet
