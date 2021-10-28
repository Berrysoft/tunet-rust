#pragma once

#include <cstddef>
#include <cstdint>
#include <functional>

namespace tunet
{
    namespace native
    {
        enum class action : std::int32_t
        {
            timer,
            tick,
            login,
            logout,
            flux,
            online,
            details,
        };

        enum class update_msg : std::int32_t
        {
            log,
            flux,
            online,
            details,
        };

        enum class state : std::int32_t
        {
            suggest,
            net,
            auth4,
            auth6
        };

        using main_callback = void (*)();
        using update_callback = void (*)(update_msg, void*);

        using model = void const*;

        extern "C"
        {
            bool tunet_runtime_init(std::size_t val, main_callback main);

            model tunet_model_new(update_callback update, void* data);
            void tunet_model_unref(model m);
            void tunet_model_queue(model m, action a);
            void tunet_model_set_state(model m, state s);
            std::uint64_t tunet_model_flux_flux(model m);
        }
    } // namespace native

    using native::action;
    using native::state;
    using native::update_msg;

    inline bool init(std::size_t threads, native::main_callback main) { return native::tunet_runtime_init(threads, main); }

    struct model
    {
        using update_callback = std::function<void(update_msg)>;

        update_callback callback{};
        native::model handle{};

        static void function_callback(update_msg msg, void* data)
        {
            auto func = reinterpret_cast<update_callback*>(data);
            (*func)(msg);
        }

        model(update_callback update) : callback(update) { handle = native::tunet_model_new(function_callback, &callback); }
        model(const model&) = delete;
        model(model&&) = delete;
        model& operator=(const model&) = delete;
        model& operator=(model&&) = delete;

        ~model() { native::tunet_model_unref(handle); }

        void queue(action a) { native::tunet_model_queue(handle, a); }

        void state(state s) { native::tunet_model_set_state(handle, s); }

        std::uint64_t flux_flux() { return native::tunet_model_flux_flux(handle); }
    };
} // namespace tunet
