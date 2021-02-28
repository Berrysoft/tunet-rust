#ifndef TUNET_H
#define TUNET_H

#ifndef TUNET_API
    #ifdef _MSC_VER
        #define TUNET_API __cdecl
    #else
        #define TUNET_API
    #endif
#endif // !TUNET_API

#ifndef TUNET_RESTRICT
    #if defined(_MSC_VER) || defined(__GNUC__)
        #define TUNET_RESTRICT __restrict
    #else
        #define TUNET_RESTRICT
    #endif
#endif // !TUNET_RESTRICT

#ifdef __cplusplus
extern "C"
{
#endif // __cplusplus

#include <stdbool.h>
#include <stdint.h>

    enum tunet_state
    {
        tunet_unknown,
        tunet_net,
        tunet_auth4,
        tunet_auth6
    };

    typedef struct tunet_credential
    {
        const char* username;
        const char* password;
        tunet_state state;
        bool use_proxy;
    } tunet_credential;

    typedef struct tunet_flux
    {
        char* username;
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

    typedef int(TUNET_API* tunet_usereg_users_callback)(const tunet_user* user, void* data);

    typedef struct tunet_detail
    {
        int64_t login_time;
        int64_t logout_time;
        uint64_t flux;
    } tunet_detail;

    typedef int(TUNET_API* tunet_usereg_details_callback)(const tunet_detail* detail, void* data);

    enum tunet_detail_order
    {
        tunet_detail_login_time,
        tunet_detail_logout_time,
        tunet_detail_flux
    };

    char* TUNET_API tunet_last_err(void);
    void TUNET_API tunet_string_free(char* message);

    int32_t TUNET_API tunet_login(const tunet_credential* cred);
    int32_t TUNET_API tunet_logout(const tunet_credential* cred);
    int32_t TUNET_API tunet_status(const tunet_credential* TUNET_RESTRICT cred, tunet_flux* TUNET_RESTRICT flux);

    int32_t TUNET_API tunet_usereg_login(const tunet_credential* cred);
    int32_t TUNET_API tunet_usereg_logout(const tunet_credential* cred);
    int32_t TUNET_API tunet_usereg_drop(const tunet_credential* cred, uint32_t addr);

    int32_t TUNET_API tunet_usereg_users(const tunet_credential* cred, tunet_usereg_users_callback callback, void* data);

    int32_t TUNET_API tunet_usereg_details(const tunet_credential* cred, tunet_detail_order order, int descending, tunet_usereg_details_callback callback, void* data);

#ifdef __cplusplus
}
#endif // __cplusplus

#endif // !TUNET_H
