#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
typedef struct _Dart_Handle* Dart_Handle;

typedef struct DartCObject DartCObject;

typedef int64_t DartPort;

typedef bool (*DartPostCObjectFnType)(DartPort port_id, void *message);

typedef struct wire_MutexOptionMpscReceiverAction {
  const void *ptr;
} wire_MutexOptionMpscReceiverAction;

typedef struct wire_MutexModel {
  const void *ptr;
} wire_MutexModel;

typedef struct wire_MutexOptionHandle {
  const void *ptr;
} wire_MutexOptionHandle;

typedef struct wire_Runtime {
  struct wire_MutexOptionMpscReceiverAction rx;
  struct wire_MutexModel model;
  struct wire_MutexOptionHandle handle;
} wire_Runtime;

typedef struct wire_NetStatus_Unknown {

} wire_NetStatus_Unknown;

typedef struct wire_NetStatus_Wwan {

} wire_NetStatus_Wwan;

typedef struct wire_uint_8_list {
  uint8_t *ptr;
  int32_t len;
} wire_uint_8_list;

typedef struct wire_NetStatus_Wlan {
  struct wire_uint_8_list *field0;
} wire_NetStatus_Wlan;

typedef struct wire_NetStatus_Lan {

} wire_NetStatus_Lan;

typedef union NetStatusKind {
  struct wire_NetStatus_Unknown *Unknown;
  struct wire_NetStatus_Wwan *Wwan;
  struct wire_NetStatus_Wlan *Wlan;
  struct wire_NetStatus_Lan *Lan;
} NetStatusKind;

typedef struct wire_NetStatus {
  int32_t tag;
  union NetStatusKind *kind;
} wire_NetStatus;

typedef struct wire_RuntimeStartConfig {
  struct wire_NetStatus status;
  struct wire_uint_8_list *username;
  struct wire_uint_8_list *password;
} wire_RuntimeStartConfig;

typedef struct wire_Ipv4AddrWrap {
  struct wire_uint_8_list *octets;
} wire_Ipv4AddrWrap;

typedef struct wire_list_ipv_4_addr_wrap {
  struct wire_Ipv4AddrWrap *ptr;
  int32_t len;
} wire_list_ipv_4_addr_wrap;

typedef struct DartCObject *WireSyncReturn;

void store_dart_post_cobject(DartPostCObjectFnType ptr);

Dart_Handle get_dart_object(uintptr_t ptr);

void drop_dart_object(uintptr_t ptr);

uintptr_t new_dart_opaque(Dart_Handle handle);

intptr_t init_frb_dart_api_dl(void *obj);

void wire_new__static_method__Runtime(int64_t port_);

void wire_start__method__Runtime(int64_t port_,
                                 struct wire_Runtime *that,
                                 struct wire_RuntimeStartConfig *config);

void wire_queue_credential__method__Runtime(int64_t port_,
                                            struct wire_Runtime *that,
                                            struct wire_uint_8_list *u,
                                            struct wire_uint_8_list *p);

void wire_queue_login__method__Runtime(int64_t port_, struct wire_Runtime *that);

void wire_queue_logout__method__Runtime(int64_t port_, struct wire_Runtime *that);

void wire_queue_flux__method__Runtime(int64_t port_, struct wire_Runtime *that);

void wire_queue_state__method__Runtime(int64_t port_, struct wire_Runtime *that, int32_t *s);

void wire_queue_details__method__Runtime(int64_t port_, struct wire_Runtime *that);

void wire_queue_onlines__method__Runtime(int64_t port_, struct wire_Runtime *that);

void wire_queue_connect__method__Runtime(int64_t port_,
                                         struct wire_Runtime *that,
                                         struct wire_Ipv4AddrWrap *ip);

void wire_queue_drop__method__Runtime(int64_t port_,
                                      struct wire_Runtime *that,
                                      struct wire_list_ipv_4_addr_wrap *ips);

struct wire_MutexModel new_MutexModel(void);

struct wire_MutexOptionHandle new_MutexOptionHandle(void);

struct wire_MutexOptionMpscReceiverAction new_MutexOptionMpscReceiverAction(void);

struct wire_Ipv4AddrWrap *new_box_autoadd_ipv_4_addr_wrap_0(void);

int32_t *new_box_autoadd_net_state_0(int32_t value);

struct wire_Runtime *new_box_autoadd_runtime_0(void);

struct wire_RuntimeStartConfig *new_box_autoadd_runtime_start_config_0(void);

struct wire_list_ipv_4_addr_wrap *new_list_ipv_4_addr_wrap_0(int32_t len);

struct wire_uint_8_list *new_uint_8_list_0(int32_t len);

void drop_opaque_MutexModel(const void *ptr);

const void *share_opaque_MutexModel(const void *ptr);

void drop_opaque_MutexOptionHandle(const void *ptr);

const void *share_opaque_MutexOptionHandle(const void *ptr);

void drop_opaque_MutexOptionMpscReceiverAction(const void *ptr);

const void *share_opaque_MutexOptionMpscReceiverAction(const void *ptr);

union NetStatusKind *inflate_NetStatus_Wlan(void);

void free_WireSyncReturn(WireSyncReturn ptr);

static int64_t dummy_method_to_enforce_bundling(void) {
    int64_t dummy_var = 0;
    dummy_var ^= ((int64_t) (void*) wire_new__static_method__Runtime);
    dummy_var ^= ((int64_t) (void*) wire_start__method__Runtime);
    dummy_var ^= ((int64_t) (void*) wire_queue_credential__method__Runtime);
    dummy_var ^= ((int64_t) (void*) wire_queue_login__method__Runtime);
    dummy_var ^= ((int64_t) (void*) wire_queue_logout__method__Runtime);
    dummy_var ^= ((int64_t) (void*) wire_queue_flux__method__Runtime);
    dummy_var ^= ((int64_t) (void*) wire_queue_state__method__Runtime);
    dummy_var ^= ((int64_t) (void*) wire_queue_details__method__Runtime);
    dummy_var ^= ((int64_t) (void*) wire_queue_onlines__method__Runtime);
    dummy_var ^= ((int64_t) (void*) wire_queue_connect__method__Runtime);
    dummy_var ^= ((int64_t) (void*) wire_queue_drop__method__Runtime);
    dummy_var ^= ((int64_t) (void*) new_MutexModel);
    dummy_var ^= ((int64_t) (void*) new_MutexOptionHandle);
    dummy_var ^= ((int64_t) (void*) new_MutexOptionMpscReceiverAction);
    dummy_var ^= ((int64_t) (void*) new_box_autoadd_ipv_4_addr_wrap_0);
    dummy_var ^= ((int64_t) (void*) new_box_autoadd_net_state_0);
    dummy_var ^= ((int64_t) (void*) new_box_autoadd_runtime_0);
    dummy_var ^= ((int64_t) (void*) new_box_autoadd_runtime_start_config_0);
    dummy_var ^= ((int64_t) (void*) new_list_ipv_4_addr_wrap_0);
    dummy_var ^= ((int64_t) (void*) new_uint_8_list_0);
    dummy_var ^= ((int64_t) (void*) drop_opaque_MutexModel);
    dummy_var ^= ((int64_t) (void*) share_opaque_MutexModel);
    dummy_var ^= ((int64_t) (void*) drop_opaque_MutexOptionHandle);
    dummy_var ^= ((int64_t) (void*) share_opaque_MutexOptionHandle);
    dummy_var ^= ((int64_t) (void*) drop_opaque_MutexOptionMpscReceiverAction);
    dummy_var ^= ((int64_t) (void*) share_opaque_MutexOptionMpscReceiverAction);
    dummy_var ^= ((int64_t) (void*) inflate_NetStatus_Wlan);
    dummy_var ^= ((int64_t) (void*) free_WireSyncReturn);
    dummy_var ^= ((int64_t) (void*) store_dart_post_cobject);
    dummy_var ^= ((int64_t) (void*) get_dart_object);
    dummy_var ^= ((int64_t) (void*) drop_dart_object);
    dummy_var ^= ((int64_t) (void*) new_dart_opaque);
    return dummy_var;
}
