#ifndef TUNET_H
#define TUNET_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

enum tunet_state
{
    tunet_unknown,
    tunet_net,
    tunet_auth4,
    tunet_auth6,
    tunet_auto
};

typedef struct tunet_ac_id_hints
{
    const int32_t* data;
    size_t size;
} tunet_ac_id_hints;

typedef struct tunet_credential
{
    const char* username;
    const char* password;
    tunet_state state;
    bool use_proxy;
    tunet_ac_id_hints ac_id_hints;
} tunet_credential;

typedef struct tunet_flux
{
    const char* username;
    uint64_t flux;
    uint64_t online_time;
    double balance;
} tunet_flux;

typedef struct tunet_user
{
    uint32_t address;
    int64_t login_time;
    uint8_t mac_address[6];
} tunet_user;

typedef bool (*tunet_usereg_users_callback)(const tunet_user* const user, void* const data);

typedef struct tunet_detail
{
    int64_t login_time;
    int64_t logout_time;
    uint64_t flux;
} tunet_detail;

typedef bool (*tunet_usereg_details_callback)(const tunet_detail* const detail, void* const data);

enum tunet_detail_order
{
    tunet_detail_login_time,
    tunet_detail_logout_time,
    tunet_detail_flux
};

#ifdef __cplusplus
extern "C"
{
#endif // __cplusplus

    const char* tunet_last_err(void);

    int32_t tunet_suggest(bool proxy);
    tunet_ac_id_hints tunet_hints(void);
    int32_t tunet_login(const tunet_credential* const cred);
    int32_t tunet_logout(const tunet_credential* const cred);
    int32_t tunet_status(const tunet_credential* const cred, tunet_flux* const flux);

    int32_t tunet_usereg_login(const tunet_credential* const cred);
    int32_t tunet_usereg_logout(const tunet_credential* const cred);
    int32_t tunet_usereg_drop(const tunet_credential* const cred, const uint32_t addr);
    int32_t tunet_usereg_users(const tunet_credential* const cred, const tunet_usereg_users_callback callback, void* const data);
    int32_t tunet_usereg_details(const tunet_credential* const cred, const tunet_detail_order order, const bool desc, const tunet_usereg_details_callback callback, void* const data);

#ifdef __cplusplus
}
#endif // __cplusplus

#endif // !TUNET_H
