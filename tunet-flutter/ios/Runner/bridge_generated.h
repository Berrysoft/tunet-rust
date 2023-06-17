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

typedef struct wire_MutexNetStatus {
  const void *ptr;
} wire_MutexNetStatus;

typedef struct wire_Runtime {
  struct wire_MutexOptionMpscReceiverAction rx;
  struct wire_MutexModel model;
  struct wire_MutexOptionHandle handle;
  struct wire_MutexNetStatus init_status;
} wire_Runtime;

typedef struct wire_uint_8_list {
  uint8_t *ptr;
  int32_t len;
} wire_uint_8_list;

typedef struct wire_NetStateWrap {
  int32_t field0;
} wire_NetStateWrap;

typedef struct DartCObject *WireSyncReturn;

void store_dart_post_cobject(DartPostCObjectFnType ptr);

Dart_Handle get_dart_object(uintptr_t ptr);

void drop_dart_object(uintptr_t ptr);

uintptr_t new_dart_opaque(Dart_Handle handle);

intptr_t init_frb_dart_api_dl(void *obj);

void wire_flux_to_string(int64_t port_, uint64_t f);

void wire_new__static_method__Runtime(int64_t port_);

void wire_initialize_status__method__Runtime(int64_t port_,
                                             struct wire_Runtime *that,
                                             int32_t t,
                                             struct wire_uint_8_list *ssid);

void wire_start__method__Runtime(int64_t port_, struct wire_Runtime *that);

void wire_queue_credential__method__Runtime(int64_t port_,
                                            struct wire_Runtime *that,
                                            struct wire_uint_8_list *u,
                                            struct wire_uint_8_list *p);

void wire_queue_login__method__Runtime(int64_t port_, struct wire_Runtime *that);

void wire_queue_logout__method__Runtime(int64_t port_, struct wire_Runtime *that);

void wire_queue_flux__method__Runtime(int64_t port_, struct wire_Runtime *that);

void wire_queue_state__method__Runtime(int64_t port_,
                                       struct wire_Runtime *that,
                                       struct wire_NetStateWrap *s);

void wire_queue_details__method__Runtime(int64_t port_, struct wire_Runtime *that);

void wire_log_busy__method__Runtime(int64_t port_, struct wire_Runtime *that);

void wire_flux__method__Runtime(int64_t port_, struct wire_Runtime *that);

void wire_state__method__Runtime(int64_t port_, struct wire_Runtime *that);

void wire_status__method__Runtime(int64_t port_, struct wire_Runtime *that);

void wire_details__method__Runtime(int64_t port_, struct wire_Runtime *that);

void wire_detail_daily__method__Runtime(int64_t port_, struct wire_Runtime *that);

struct wire_MutexModel new_MutexModel(void);

struct wire_MutexNetStatus new_MutexNetStatus(void);

struct wire_MutexOptionHandle new_MutexOptionHandle(void);

struct wire_MutexOptionMpscReceiverAction new_MutexOptionMpscReceiverAction(void);

struct wire_NetStateWrap *new_box_autoadd_net_state_wrap_0(void);

struct wire_Runtime *new_box_autoadd_runtime_0(void);

struct wire_uint_8_list *new_uint_8_list_0(int32_t len);

void drop_opaque_MutexModel(const void *ptr);

const void *share_opaque_MutexModel(const void *ptr);

void drop_opaque_MutexNetStatus(const void *ptr);

const void *share_opaque_MutexNetStatus(const void *ptr);

void drop_opaque_MutexOptionHandle(const void *ptr);

const void *share_opaque_MutexOptionHandle(const void *ptr);

void drop_opaque_MutexOptionMpscReceiverAction(const void *ptr);

const void *share_opaque_MutexOptionMpscReceiverAction(const void *ptr);

void free_WireSyncReturn(WireSyncReturn ptr);

static int64_t dummy_method_to_enforce_bundling(void) {
    int64_t dummy_var = 0;
    dummy_var ^= ((int64_t) (void*) wire_flux_to_string);
    dummy_var ^= ((int64_t) (void*) wire_new__static_method__Runtime);
    dummy_var ^= ((int64_t) (void*) wire_initialize_status__method__Runtime);
    dummy_var ^= ((int64_t) (void*) wire_start__method__Runtime);
    dummy_var ^= ((int64_t) (void*) wire_queue_credential__method__Runtime);
    dummy_var ^= ((int64_t) (void*) wire_queue_login__method__Runtime);
    dummy_var ^= ((int64_t) (void*) wire_queue_logout__method__Runtime);
    dummy_var ^= ((int64_t) (void*) wire_queue_flux__method__Runtime);
    dummy_var ^= ((int64_t) (void*) wire_queue_state__method__Runtime);
    dummy_var ^= ((int64_t) (void*) wire_queue_details__method__Runtime);
    dummy_var ^= ((int64_t) (void*) wire_log_busy__method__Runtime);
    dummy_var ^= ((int64_t) (void*) wire_flux__method__Runtime);
    dummy_var ^= ((int64_t) (void*) wire_state__method__Runtime);
    dummy_var ^= ((int64_t) (void*) wire_status__method__Runtime);
    dummy_var ^= ((int64_t) (void*) wire_details__method__Runtime);
    dummy_var ^= ((int64_t) (void*) wire_detail_daily__method__Runtime);
    dummy_var ^= ((int64_t) (void*) new_MutexModel);
    dummy_var ^= ((int64_t) (void*) new_MutexNetStatus);
    dummy_var ^= ((int64_t) (void*) new_MutexOptionHandle);
    dummy_var ^= ((int64_t) (void*) new_MutexOptionMpscReceiverAction);
    dummy_var ^= ((int64_t) (void*) new_box_autoadd_net_state_wrap_0);
    dummy_var ^= ((int64_t) (void*) new_box_autoadd_runtime_0);
    dummy_var ^= ((int64_t) (void*) new_uint_8_list_0);
    dummy_var ^= ((int64_t) (void*) drop_opaque_MutexModel);
    dummy_var ^= ((int64_t) (void*) share_opaque_MutexModel);
    dummy_var ^= ((int64_t) (void*) drop_opaque_MutexNetStatus);
    dummy_var ^= ((int64_t) (void*) share_opaque_MutexNetStatus);
    dummy_var ^= ((int64_t) (void*) drop_opaque_MutexOptionHandle);
    dummy_var ^= ((int64_t) (void*) share_opaque_MutexOptionHandle);
    dummy_var ^= ((int64_t) (void*) drop_opaque_MutexOptionMpscReceiverAction);
    dummy_var ^= ((int64_t) (void*) share_opaque_MutexOptionMpscReceiverAction);
    dummy_var ^= ((int64_t) (void*) free_WireSyncReturn);
    dummy_var ^= ((int64_t) (void*) store_dart_post_cobject);
    dummy_var ^= ((int64_t) (void*) get_dart_object);
    dummy_var ^= ((int64_t) (void*) drop_dart_object);
    dummy_var ^= ((int64_t) (void*) new_dart_opaque);
    return dummy_var;
}
