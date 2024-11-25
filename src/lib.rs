pub mod bevy_reflect_for_tracked_dyn;
pub mod spirv;
use base_type::BaseType;
use dyn_layout::DynLayout;
pub mod base_type;
pub mod dyn_layout;
pub mod dyn_struct;
pub mod tracked_dyn_struct;

pub mod update_bitmask;

/// Usage
/// T: data type of slice
/// dyn_struct: The struct we are retrieving changes from
///
/// data_slice: The current slice of data we can extend onto our larger slice of update_data
/// start: start of slice units
/// end: end of slice units
///
/// update_data: all the collected changes we need to update the retained buffer with
/// indices: tracks where each u32 in update_data should go in the retained buffer
/// dst_offset: the start index of this dyn_struct in the retained buffer
///
/// retrieve_changes!(u32, dyn_struct, |data_slice, start, end| {
///     update_data.extend_from_slice(data_slice);
///     indices.extend((dst_offset + start)..(dst_offset + end));
/// });
// TODO figure out if closure version is slow or not. Also consider generic version.
#[macro_export]
macro_rules! retrieve_changes {
    ($T:ty, $dyn_struct:expr, $extend_fn:expr) => {{
        if !$dyn_struct.update_bitmask.any_set() || $dyn_struct.dyn_struct.data.is_empty() {
            return;
        }

        let src_data: &[$T] = bytemuck::cast_slice(&$dyn_struct.dyn_struct.data);
        let data_length = src_data.len();

        for (chunk_n, chunk) in $dyn_struct.update_bitmask.bits.iter().enumerate() {
            let chunk_index = chunk_n << 4;
            let mut chunk = *chunk;
            let mut total = 0;
            while chunk != 0 {
                // Start from LSB and chip away at chunk while making copies the size of contiguous ones
                let count = chunk.trailing_ones() as usize;
                let start = chunk_index + total;
                let end = (chunk_index + total + count).min(data_length);
                $extend_fn(&src_data[start..end], start as u32, end as u32);
                total += count;
                chunk = chunk.checked_shr(count as u32).unwrap_or(0);
                let zeros_count = chunk.trailing_zeros();
                chunk = chunk.checked_shr(zeros_count).unwrap_or(0);
                total += zeros_count as usize;
            }
        }
    }};
}

#[macro_export]
macro_rules! retrieve_changes_and_reset {
    ($T:ty, $dyn_struct:expr, $extend_fn:expr) => {{
        $crate::retrieve_changes!($T, $dyn_struct, $extend_fn);
        $dyn_struct.reset_change_detection();
    }};
}

/*
Example shader to update retained buffer

struct Config
{
    uint count;
    uint spare1;
    uint spare2;
    uint spare3;
};

struct DataCopy
{
    uint data[UPDATE_STRIDE];
};

#include "shader_util.hlsl"
PUSH_CONSTANT(Config, conf)

[[vk::binding(0, 0)]]
StructuredBuffer<DataCopy> src_buffer : register(t0, space0);
[[vk::binding(1, 0)]]
StructuredBuffer<uint> update_indices_buffer : register(t0, space0);
[[vk::binding(2, 0)]]
RWStructuredBuffer<DataCopy> dst_buffer : register(u0, space0);

[numthreads(64, 1, 1)]
void main(uint3 dispatchThreadID: SV_DispatchThreadID, uint3 groupID: SV_GroupID, uint3 groupThreadID: SV_GroupThreadID)
{
    int update_index = dispatchThreadID.x;
    if (update_index >= conf.count)
    {
        return;
    }
    uint destination_index = update_indices_buffer[update_index];
    dst_buffer[destination_index] = src_buffer[update_index];
}
*/
