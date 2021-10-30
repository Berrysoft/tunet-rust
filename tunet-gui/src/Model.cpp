#include <Model.hpp>
#include <cmath>

extern "C"
{
    using MainCallback = int (*)(void*);
    using UpdateCallback = void (*)(UpdateMsg, void*);

    struct StringView
    {
        const char8_t* data;
        std::size_t size;

        operator QString() const
        {
            return QString::fromUtf8(data, size);
        }
    };

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
    StringView tunet_model_cred_username(NativeModel m);
    StringView tunet_model_cred_password(NativeModel m);
    State tunet_model_state(NativeModel m);
    StringView tunet_model_log(NativeModel m);
    StringView tunet_model_flux_username(NativeModel m);
    std::uint64_t tunet_model_flux_flux(NativeModel m);
    std::int64_t tunet_model_flux_online_time(NativeModel m);
    double tunet_model_flux_balance(NativeModel m);
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

QString tunet_format_flux(std::uint64_t f)
{
    double flux = f;
    if (flux < 1000.0)
    {
        return QString("%1 B").arg(f);
    }
    flux /= 1000.0;
    if (flux < 1000.0)
    {
        return QString("%1 K").arg(flux, 0, 'f', 2);
    }
    flux /= 1000.0;
    if (flux < 1000.0)
    {
        return QString("%1 M").arg(flux, 0, 'f', 2);
    }
    flux /= 1000.0;
    return QString("%1 G").arg(flux, 0, 'f', 2);
}

QString tunet_format_duration(std::chrono::seconds s)
{
    auto total_sec = s.count();
    auto [total_min, sec] = std::div(total_sec, 60ll);
    auto [total_h, min] = std::div(total_min, 60ll);
    auto [day, h] = std::div(total_h, 60ll);
    if (day)
    {
        return QString("%1.%2:%3:%4").arg(day).arg(h, 2, 10, QChar(u'0')).arg(min, 2, 10, QChar(u'0')).arg(sec, 2, 10, QChar(u'0'));
    }
    else
    {
        return QString("%1:%2:%3").arg(h, 2, 10, QChar(u'0')).arg(min, 2, 10, QChar(u'0')).arg(sec, 2, 10, QChar(u'0'));
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
    case UpdateMsg::State:
        emit state_changed();
        break;
    case UpdateMsg::Log:
        emit log_changed();
        break;
    case UpdateMsg::Flux:
        emit flux_changed();
        break;
    }
}

NetCredential Model::cred() const
{
    auto username = tunet_model_cred_username(m_handle);
    auto password = tunet_model_cred_password(m_handle);
    return { username, password };
}

State Model::state() const
{
    return tunet_model_state(m_handle);
}

QString Model::log() const
{
    return tunet_model_log(m_handle);
}

NetFlux Model::flux() const
{
    auto username = tunet_model_flux_username(m_handle);
    auto f = tunet_model_flux_flux(m_handle);
    auto online = tunet_model_flux_online_time(m_handle);
    auto balance = tunet_model_flux_balance(m_handle);
    return NetFlux{ username, f, std::chrono::seconds{ online }, balance };
}
