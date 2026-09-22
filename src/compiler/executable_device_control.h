/* SPDX-License-Identifier: MPL-2.0
 * Host/device transfer ABI. All persistent graph/frame links are scratch word
 * offsets. Device pointer bases are bound separately by initialization. */
#ifndef TB_DEVICE_CONTROL_DEFINED
#define TB_DEVICE_CONTROL_DEFINED
struct TBDeviceControl {
  u64 ready_head, ready_tail, completed_head, completed_tail;
  u64 adopt_head, adopt_tail, adopt_cursor, adopt_boundary;
  u64 table[1024];
  u64 root_words[255], root_owned[255];
  u64 dispatches, direct_calls, reused_calls, forks, launches;
  u64 lanes_seen[4];
  u64 helper_depths, helper_live, helper_calls;
  u32 helper_lanes, helper_limit, helper_peak;
  u32 ready_count, completed_count, live_runs, live_records, live_frames;
  u32 run_limit, task_limit, frame_limit;
  u32 adopt_phase, root_count, done, lock, active_lanes, peak_lanes;
};
#endif
