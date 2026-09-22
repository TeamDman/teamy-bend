/* SPDX-License-Identifier: MPL-2.0
 * Shared host/device corpus transfer record. u64/u32 are supplied by the
 * including backend. Every graph/storage reference is an offset, not a host
 * address; device buffer bases are separate kernel arguments. */
#ifndef TEAMY_BEND_DEVICE_STATE_H
#define TEAMY_BEND_DEVICE_STATE_H
typedef struct TBDeviceState {
  u64 capacity, bump, live_words, live_blocks, steps, step_limit;
  u64 free_lists[32];
  u64 scratch_capacity, scratch_bump, scratch_live, scratch_peak;
  u64 scratch_free[32];
  u32 lock, error, active_lanes, reserved;
  char error_text[192];
} TBDeviceState;
#ifdef __cplusplus
static_assert(sizeof(u64) == 8 && sizeof(u32) == 4 && sizeof(TBDeviceState) == 800,
  "unsupported Bend device-state representation");
#else
_Static_assert(sizeof(u64) == 8 && sizeof(u32) == 4 && sizeof(TBDeviceState) == 800,
  "unsupported Bend device-state representation");
#endif
#endif
