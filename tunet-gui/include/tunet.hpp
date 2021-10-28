#pragma once

#include <cstddef>
#include <cstdint>

namespace tunet
{
    namespace native
    {
        extern "C"
        {
            bool tunet_runtime_init(std::size_t val);

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

            using update_callback = void (*)(update_msg, void*);

            using model = void const*;

            model tunet_model_new(update_callback update, void* data);
        }
    } // namespace native
} // namespace tunet
