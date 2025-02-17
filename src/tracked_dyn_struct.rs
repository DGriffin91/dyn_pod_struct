use std::sync::Arc;

#[cfg(feature = "bevy_reflect")]
use bevy_reflect::TypePath;

use bytemuck::{Pod, Zeroable};

use crate::{dyn_layout::DynLayout, dyn_struct::DynStruct, update_bitmask::UpdateBitmask};

/// Adds granular change detection tracking on top of DynStruct.
/// When `get_mut` or `get_mut_raw` are called the offset or path and size_of::<T>() are used to track what regions of
/// the data have been updated.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "bevy_reflect", derive(TypePath))]
pub struct TrackedDynStruct {
    pub dyn_struct: DynStruct,
    pub update_bitmask: UpdateBitmask,
    update_stride_exp: usize,
}

impl TrackedDynStruct {
    #[inline(always)]
    /// Copies data into new DynStruct using provided layout.
    /// Creating a layout can be slow, prefer creating a layout once and reusing.
    /// let layout = T::dyn_layout();
    /// `update_stride` is the granularity of update tracking. Must be a power of 2.
    /// Use 1 for tracking every byte. Use 4 for every 4 bytes, etc...
    /// Use `update_default` to set whether the update tracking indicates the data has been updated at creation or not.
    pub fn new<T: Pod>(
        data: &T,
        layout: &Arc<DynLayout>,
        update_stride: usize,
        update_default: bool,
    ) -> Self {
        let update_bitmask = UpdateBitmask::new(size_of::<T>() / update_stride, update_default);
        let dyn_struct = DynStruct::new(data, layout);
        TrackedDynStruct {
            dyn_struct,
            update_bitmask,
            update_stride_exp: update_stride.trailing_zeros() as usize,
        }
    }

    pub fn from_bytes(
        data: Vec<u8>,
        layout: Arc<DynLayout>,
        update_stride: usize,
        update_default: bool,
    ) -> Self {
        let update_bitmask = UpdateBitmask::new(data.len() / update_stride, update_default);
        let dyn_struct = DynStruct::from_bytes(data, layout);
        TrackedDynStruct {
            dyn_struct,
            update_bitmask,
            update_stride_exp: update_stride.trailing_zeros() as usize,
        }
    }

    #[inline(always)]
    pub fn get<T: Pod + Zeroable>(&self, path: &[&str]) -> Option<&T> {
        self.dyn_struct.get(path)
    }

    #[inline(always)]
    pub fn get_mut<T: Pod + Zeroable>(&mut self, path: &[&str]) -> Option<&mut T> {
        if let Some(field) = self.dyn_struct.layout.get_path(path) {
            // If this shouldn't be debug, bring back DynField size, field.ty.size_of() is too slow
            debug_assert_eq!(size_of::<T>(), field.ty.size_of());
            Some(self.get_mut_raw(field.offset as usize))
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn get_raw<T: Pod + Zeroable>(&self, offset: usize) -> &T {
        self.dyn_struct.get_raw(offset)
    }

    #[inline(always)]
    pub fn get_mut_raw<T: Pod + Zeroable>(&mut self, offset: usize) -> &mut T {
        self.mark_changed::<T>(offset);
        self.dyn_struct.get_mut_raw(offset)
    }

    #[inline(always)]
    /// For manually setting granular change detection. Not needed if using get_mut or get_mut_raw
    pub fn mark_changed<T: Pod + Zeroable>(&mut self, offset: usize) {
        let bitmask_start = offset >> self.update_stride_exp;
        let bitmask_end = (offset + size_of::<T>()) >> self.update_stride_exp;
        self.update_bitmask.set(bitmask_start..bitmask_end);
    }

    /// dyn_struct.retrieve_changes(|data_slice, start, end| {
    ///     data.extend_from_slice(data_slice);
    ///     indices.extend((dst_offset + start)..(dst_offset + end));
    /// });
    #[inline(always)]
    pub fn retrieve_changes<T: bytemuck::Pod>(&self, mut extend_fn: impl FnMut(&[T], u32, u32)) {
        if !self.update_bitmask.any_set() || self.dyn_struct.data.is_empty() {
            return;
        }

        let src_data: &[T] = bytemuck::cast_slice(&self.dyn_struct.data);
        let data_length = src_data.len();

        for (chunk_n, chunk) in self.update_bitmask.bits.iter().enumerate() {
            let chunk_index = chunk_n << 4;
            let mut chunk = *chunk;
            let mut total = 0;
            while chunk != 0 {
                // Start from LSB and chip away at chunk while making copies the size of contiguous ones
                let count = chunk.trailing_ones() as usize;
                let start = chunk_index + total;
                let end = (chunk_index + total + count).min(data_length);
                extend_fn(&src_data[start..end], start as u32, end as u32);
                total += count;
                chunk = chunk.checked_shr(count as u32).unwrap_or(0);
                let zeros_count = chunk.trailing_zeros();
                chunk = chunk.checked_shr(zeros_count).unwrap_or(0);
                total += zeros_count as usize;
            }
        }
    }

    #[inline(always)]
    pub fn retrieve_changes_and_reset<T: bytemuck::Pod>(
        &mut self,
        extend_fn: impl FnMut(&[T], u32, u32),
    ) {
        self.retrieve_changes(extend_fn);
        self.reset_change_detection();
    }

    #[inline(always)]
    pub fn changed(&self) -> bool {
        self.update_bitmask.any_set()
    }

    #[inline(always)]
    pub fn reset_change_detection(&mut self) {
        self.update_bitmask.reset();
    }
}
