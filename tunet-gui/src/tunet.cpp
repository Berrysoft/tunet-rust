#include <tunet.hpp>

namespace tunet
{
    namespace native
    {
        extern "C"
        {
            using update_callback = void (*)(update_msg, void*);

            struct string_view
            {
                const char* data;
                std::size_t size;
            };

            bool tunet_runtime_init(std::size_t val, main_callback main);

            model_handle tunet_model_new(update_callback update, void* data);
            void tunet_model_unref(model_handle m);
            void tunet_model_queue(model_handle m, action a);
            void tunet_model_set_state(model_handle m, state s);
            string_view tunet_model_flux_username(model_handle m);
            std::uint64_t tunet_model_flux_flux(model_handle m);
            std::int64_t tunet_model_flux_online_time(model_handle m);
            double tunet_model_flux_balance(model_handle m);
        }
    } // namespace native

    bool start(std::size_t threads, main_callback main) { return native::tunet_runtime_init(threads, main); }

    static void function_callback(update_msg msg, void* data)
    {
        auto func = reinterpret_cast<model::update_callback*>(data);
        (*func)(msg);
    }

    model::model(update_callback update) : callback(update) { handle = native::tunet_model_new(function_callback, &callback); }

    model::~model() { native::tunet_model_unref(handle); }

    void model::queue(action a) { native::tunet_model_queue(handle, a); }

    void model::set_state(state s) { native::tunet_model_set_state(handle, s); }

    net_flux model::get_flux()
    {
        auto username = native::tunet_model_flux_username(handle);
        auto f = native::tunet_model_flux_flux(handle);
        auto online = native::tunet_model_flux_online_time(handle);
        auto balance = native::tunet_model_flux_balance(handle);
        return net_flux{ std::string_view{ username.data, username.size }, f, std::chrono::seconds{ online }, balance };
    }
} // namespace tunet
