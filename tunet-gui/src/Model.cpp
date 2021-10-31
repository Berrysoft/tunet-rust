#include <Model.hpp>
#include <cmath>

extern "C"
{
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

    using MainCallback = int (*)(void*);
    using UpdateCallback = void (*)(UpdateMsg, void*);
    using StringCallback = void (*)(const char8_t*, std::size_t, void*);
    using OnlinesForeachCallback = bool (*)(const OnlineUser*, void*);
    using DetailsForeachCallback = bool (*)(const Detail*, void*);
    using DetailsGroupedForeachCallback = bool (*)(const DetailGroup*, void*);
    using DetailsGroupedByTimeForeachCallback = bool (*)(const DetailGroupByTime*, void*);

    std::int32_t tunet_runtime_init(std::size_t val, MainCallback main, void* data);

    struct ThemeColor
    {
        std::uint8_t r;
        std::uint8_t g;
        std::uint8_t b;
    };

    ThemeColor tunet_color_accent();

    NativeModel tunet_model_new(UpdateCallback update, void* data);
    void tunet_model_unref(NativeModel m);
    void tunet_model_queue(NativeModel m, Action a);
    bool tunet_model_queue_read_cred(NativeModel m);
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

struct init_data
{
    int (*main)(int, char**);
    int argc;
    char** argv;
};

static int fn_init_callback(void* data)
{
    auto d = reinterpret_cast<init_data*>(data);
    return (d->main)(d->argc, d->argv);
}

std::int32_t tunet_start(std::size_t threads, int (*main)(int, char**), int argc, char** argv)
{
    init_data data{ main, argc, argv };
    return tunet_runtime_init(threads, fn_init_callback, &data);
}

QColor tunet_accent()
{
    auto color = tunet_color_accent();
    return QColor::fromRgb(color.r, color.g, color.b);
}

QString tunet_format_status(const Status& status)
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

QString tunet_format_flux(std::uint64_t f)
{
    double flux = f;
    if (flux < 1000.0)
    {
        return u"%1 B"_qs.arg(f);
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

QString tunet_format_duration(std::chrono::seconds s)
{
    auto total_sec = s.count();
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

static void fn_update_callback(UpdateMsg m, void* data)
{
    auto model = reinterpret_cast<Model*>(data);
    model->update(m);
}

Model::Model(QObject* parent) : QObject(parent) { m_handle = tunet_model_new(fn_update_callback, this); }

Model::~Model() { tunet_model_unref(m_handle); }

void Model::queue(Action a) const { tunet_model_queue(m_handle, a); }

bool Model::queue_read_cred() const { return tunet_model_queue_read_cred(m_handle); }

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
    case UpdateMsg::Details:
        emit details_changed();
        break;
    }
}

static void fn_string_callback(const char8_t* data, std::size_t size, void* d)
{
    QString* pstr = reinterpret_cast<QString*>(d);
    *pstr = QString::fromUtf8(data, size);
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

NetCredential Model::cred() const
{
    auto username = get_q_string(tunet_model_cred_username, m_handle);
    auto password = get_q_string(tunet_model_cred_password, m_handle);
    return { std::move(username), std::move(password) };
}

State Model::state() const
{
    return tunet_model_state(m_handle);
}

QString Model::log() const
{
    return get_q_string(tunet_model_log, m_handle);
}

NetFlux Model::flux() const
{
    auto username = get_q_string(tunet_model_flux_username, m_handle);
    auto f = tunet_model_flux_flux(m_handle);
    auto online = tunet_model_flux_online_time(m_handle);
    auto balance = tunet_model_flux_balance(m_handle);
    return NetFlux{ std::move(username), f, std::chrono::seconds{ online }, balance };
}

static bool fn_foreach_detail(const Detail* d, void* data)
{
    std::vector<NetDetail>& details = *reinterpret_cast<std::vector<NetDetail>*>(data);
    details.emplace_back(QDateTime::fromSecsSinceEpoch(d->login_time, Qt::UTC), QDateTime::fromSecsSinceEpoch(d->logout_time, Qt::UTC), d->flux);
    return true;
}

std::vector<NetDetail> Model::details() const
{
    std::vector<NetDetail> details{};
    tunet_model_details_foreach(m_handle, fn_foreach_detail, &details);
    return details;
}

static bool fn_foreach_detail_group(const DetailGroup* d, void* data)
{
    std::vector<NetDetailGroup>& details = *reinterpret_cast<std::vector<NetDetailGroup>*>(data);
    details.emplace_back(QDateTime::fromSecsSinceEpoch(d->logout_date, Qt::UTC).date(), d->flux);
    return true;
}

std::vector<NetDetailGroup> Model::details_grouped() const
{
    std::vector<NetDetailGroup> details{};
    tunet_model_details_grouped_foreach(m_handle, fn_foreach_detail_group, &details);
    return details;
}

static bool fn_foreach_detail_group_by_time(const DetailGroupByTime* d, void* data)
{
    std::map<std::uint32_t, std::uint64_t>& details = *reinterpret_cast<std::map<std::uint32_t, std::uint64_t>*>(data);
    details.emplace(d->logout_start_time, d->flux);
    return true;
}

std::map<std::uint32_t, std::uint64_t> Model::details_grouped_by_time(std::uint32_t groups) const
{
    std::map<std::uint32_t, std::uint64_t> details{};
    tunet_model_details_grouped_by_time_foreach(m_handle, groups, fn_foreach_detail_group_by_time, &details);
    return details;
}
