#include <Model.hpp>

#if QT_VERSION < QT_VERSION_CHECK(5, 10, 0)
    #define QSTRING_UTF16(str) (reinterpret_cast<const char16_t*>((str).utf16()))
#elif QT_VERSION < QT_VERSION_CHECK(7, 0, 0)
    #define QSTRING_UTF16(str) (QStringView{ (str) }.utf16())
#else
    #define QSTRING_UTF16(str) ((str).utf16())
#endif

extern "C"
{
    using TUNet::Action;
    using TUNet::NativeModel;
    using TUNet::State;
    using TUNet::UpdateMsg;

    struct OnlineUser
    {
        std::uint32_t address;
        std::int64_t login_time;
        std::uint64_t flux;
        std::uint8_t mac_address[6];
        bool has_mac;
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

    void tunet_format_flux(std::uint64_t flux, StringCallback f, void* data);
    void tunet_format_duration(std::int64_t sec, StringCallback f, void* data);
    void tunet_format_ip(std::uint32_t addr, StringCallback f, void* data);
    std::uint32_t tunet_parse_ip(const char16_t* str);
    void tunet_format_mac_address(const std::uint8_t* addr, StringCallback f, void* data);

    std::int32_t tunet_model_start(std::size_t val, MainCallback main, void* data);
    void tunet_model_set_update_callback(NativeModel m, UpdateCallback update, void* data);
    void tunet_model_queue(NativeModel m, Action a);
    bool tunet_model_queue_cred_load(NativeModel m);
    void tunet_model_queue_cred(NativeModel m, const char16_t* u, const char16_t* p);
    void tunet_model_queue_state(NativeModel m, State s);
    void tunet_model_queue_connect(NativeModel m, std::uint32_t addr);
    void tunet_model_queue_drop(NativeModel m, std::uint32_t addr);
    void tunet_model_status(NativeModel m, StringCallback f, void* data);
    ThemeColor tunet_model_accent_color(NativeModel m);
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
    bool tunet_model_log_busy(NativeModel m);
    bool tunet_model_online_busy(NativeModel m);
    bool tunet_model_detail_busy(NativeModel m);
    void tunet_model_set_del_at_exit(NativeModel m, bool v);
}

namespace TUNet
{
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

    Ipv4Addr::Ipv4Addr(const QString& str) { m_value = tunet_parse_ip(QSTRING_UTF16(str)); }

    QString Flux::toString() const { return get_q_string(tunet_format_flux, m_value); }

    QString Ipv4Addr::toString() const { return get_q_string(tunet_format_ip, m_value); }

    QString MacAddress::toString() const { return get_q_string(tunet_format_mac_address, m_values.data()); }

    QString format_duration(std::chrono::seconds s) { return get_q_string(tunet_format_duration, s.count()); }

    QString format_datetime(const QDateTime& time)
    {
#if QT_VERSION < QT_VERSION_CHECK(5, 10, 0)
        return time.toString(QStringLiteral(u"yyyy-MM-dd hh:mm:ss"));
#else
        constexpr QStringView DATETIME_FORMAT{ u"yyyy-MM-dd hh:mm:ss" };
        return time.toString(DATETIME_FORMAT);
#endif
    }

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

    std::int32_t Model::start(std::size_t threads, StartCallback main, int argc, char** argv)
    {
        init_data data{ main, argc, argv };
        return tunet_model_start(threads, fn_init_callback, &data);
    }

    static void fn_update_callback(UpdateMsg m, void* data)
    {
        auto model = reinterpret_cast<Model*>(data);
        model->update(m);
    }

    Model::Model(NativeModel handle) : QObject(), m_handle(handle)
    {
        tunet_model_set_update_callback(m_handle, fn_update_callback, this);
    }

    Model::~Model() { tunet_model_set_update_callback(m_handle, nullptr, nullptr); }

    void Model::queue(Action a) const { tunet_model_queue(m_handle, a); }

    void Model::queue_cred_load() const
    {
        bool loaded = tunet_model_queue_cred_load(m_handle);
        if (!loaded)
        {
            emit ask_cred({});
        }
    }

    void Model::queue_cred(const Credential& cred) const
    {
        tunet_model_queue_cred(m_handle, QSTRING_UTF16(cred.username), QSTRING_UTF16(cred.password));
    }

    void Model::queue_state(State s) const { tunet_model_queue_state(m_handle, s); }

    void Model::queue_connect(Ipv4Addr addr) const { tunet_model_queue_connect(m_handle, addr); }

    void Model::queue_drop(Ipv4Addr addr) const { tunet_model_queue_drop(m_handle, addr); }

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
        case UpdateMsg::LogBusy:
            emit log_busy_changed();
            break;
        case UpdateMsg::OnlineBusy:
            emit online_busy_changed();
            break;
        case UpdateMsg::DetailBusy:
            emit detail_busy_changed();
            break;
        }
    }

    QString Model::status() const
    {
        return get_q_string(tunet_model_status, m_handle);
    }

    QColor Model::accent_color() const
    {
        auto color = tunet_model_accent_color(m_handle);
        return QColor::fromRgb(color.r, color.g, color.b);
    }

    Credential Model::cred() const
    {
        auto username = get_q_string(tunet_model_cred_username, m_handle);
        auto password = get_q_string(tunet_model_cred_password, m_handle);
        Credential cred{ std::move(username), std::move(password) };
        if (cred.username.isEmpty() || cred.password.isEmpty())
        {
            emit ask_cred(cred);
        }
        return cred;
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
        users.emplace_back(Online{
            u->address,
            QDateTime::fromSecsSinceEpoch(u->login_time, Qt::UTC),
            u->flux,
            u->has_mac ? std::make_optional(MacAddress{ u->mac_address }) : std::nullopt,
            u->is_local });
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
        details.emplace_back(Detail{
            QDateTime::fromSecsSinceEpoch(d->login_time, Qt::UTC),
            QDateTime::fromSecsSinceEpoch(d->logout_time, Qt::UTC),
            d->flux });
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

    bool Model::log_busy() const { return tunet_model_log_busy(m_handle); }

    bool Model::online_busy() const { return tunet_model_online_busy(m_handle); }

    bool Model::detail_busy() const { return tunet_model_detail_busy(m_handle); }

    void Model::set_del_at_exit(bool v) const { return tunet_model_set_del_at_exit(m_handle, v); }
} // namespace TUNet
