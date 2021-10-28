#pragma once

#include <chrono>
#include <cstddef>
#include <cstdint>
#include <functional>

namespace tunet
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

    using main_callback = int (*)();

    using model_handle = void const*;

    bool start(std::size_t threads, main_callback main);

    struct net_flux
    {
        std::string_view username;
        std::uint64_t flux;
        std::chrono::seconds online_time;
        double balance;
    };

    struct model
    {
        using update_callback = std::function<void(update_msg)>;

        update_callback callback{};
        model_handle handle{};

        model(update_callback update);
        model(const model&) = delete;
        model(model&& m) noexcept : handle(m.handle) { m.handle = nullptr; }
        model& operator=(const model&) = delete;
        model& operator=(model&& m) noexcept
        {
            if (this != &m)
            {
                handle = m.handle;
                m.handle = nullptr;
            }
        }

        ~model();

        void queue(action a);

        void set_state(state s);

        net_flux get_flux();
    };
} // namespace tunet
