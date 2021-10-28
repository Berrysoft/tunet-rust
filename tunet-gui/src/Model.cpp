#include <Model.hpp>

extern "C"
{
    using MainCallback = int (*)(void*);
    using UpdateCallback = void (*)(UpdateMsg, void*);

    struct StringView
    {
        const char* data;
        std::size_t size;
    };

    bool tunet_runtime_init(std::size_t val, MainCallback main, void* data);

    NativeModel tunet_model_new(UpdateCallback update, void* data);
    void tunet_model_unref(NativeModel m);
    void tunet_model_queue(NativeModel m, Action a);
    void tunet_model_queue_state(NativeModel m, State s);
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

bool tunet_start(std::size_t threads, int (*main)(int, char**), int argc, char** argv)
{
    init_data data{ main, argc, argv };
    return tunet_runtime_init(threads, fn_init_callback, &data);
}

static void fn_update_callback(UpdateMsg m, void* data)
{
    auto model = reinterpret_cast<Model*>(data);
    model->update(m);
}

Model::Model() { m_handle = tunet_model_new(fn_update_callback, this); }

Model::~Model() { tunet_model_unref(m_handle); }

void Model::queue(Action a) const { tunet_model_queue(m_handle, a); }

void Model::update(UpdateMsg m) const
{
    switch (m)
    {
    case UpdateMsg::Flux:
    {
        emit flux_changed();
        break;
    }
    }
}

void Model::queue_state(State s) const { tunet_model_queue_state(m_handle, s); }

NetFlux Model::flux() const
{
    auto username = tunet_model_flux_username(m_handle);
    auto f = tunet_model_flux_flux(m_handle);
    auto online = tunet_model_flux_online_time(m_handle);
    auto balance = tunet_model_flux_balance(m_handle);
    return NetFlux{ std::string_view{ username.data, username.size }, f, std::chrono::seconds{ online }, balance };
}
