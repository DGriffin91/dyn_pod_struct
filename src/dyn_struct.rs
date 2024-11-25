use std::{any::type_name, sync::Arc};

use bevy::reflect::TypePath;
use bytemuck::{bytes_of, Pod, Zeroable};

use crate::{base_type::BaseType, dyn_layout::DynLayout};

#[derive(Clone, Default, Debug, PartialEq, Hash)]
pub struct DynField {
    /// Absolute offset into top level parent in bytes
    pub offset: u32,
    // Spare 32 bits of padding here, could cache size here. Is faster than checking size of type with .size_of()
    pub ty: BaseType,
}

#[derive(Clone, Debug, TypePath)]
pub struct DynStruct {
    pub data: Vec<u8>,
    pub layout: Arc<DynLayout>,
}

impl DynStruct {
    #[inline(always)]
    /// Copies data into new DynStruct using provided layout.
    /// Creating a layout can be slow, prefer creating a layout once and reusing.
    /// let layout = T::dyn_layout();
    pub fn new<T: Pod>(data: &T, layout: &Arc<DynLayout>) -> Self {
        if layout.size != size_of::<T>() {
            panic!(
                "DynStruct layout does not match data length ({} != {}). Layout: {:?} T: {}",
                layout.size,
                size_of::<T>(),
                layout.name,
                type_name::<T>(),
            )
        }
        DynStruct {
            data: bytes_of(data).to_vec(),
            layout: layout.clone(),
        }
    }

    pub fn from_bytes(data: Vec<u8>, layout: Arc<DynLayout>) -> Self {
        let data_len = data.len();
        let layout_data_len = layout.size;
        if layout_data_len != data_len {
            panic!("DynStruct layout does not match data length ({layout_data_len} != {data_len}). Layout: {:?}", layout.name)
        }
        DynStruct { data, layout }
    }

    #[inline(always)]
    pub fn get<T: Pod + Zeroable>(&self, path: &[&str]) -> Option<&T> {
        if let Some(field) = self.layout.get_path(path) {
            // If this shouldn't be debug, bring back DynField size, field.ty.size_of() is too slow
            debug_assert_eq!(size_of::<T>(), field.ty.size_of());
            Some(self.get_raw(field.offset as usize))
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn get_mut<T: Pod + Zeroable>(&mut self, path: &[&str]) -> Option<&mut T> {
        if let Some(field) = self.layout.get_path(path) {
            // If this shouldn't be debug, bring back DynField size, field.ty.size_of() is too slow
            debug_assert_eq!(size_of::<T>(), field.ty.size_of());
            Some(self.get_mut_raw(field.offset as usize))
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn get_raw<T: Pod + Zeroable>(&self, offset: usize) -> &T {
        bytemuck::from_bytes(&self.data[offset..offset + size_of::<T>()])
    }

    #[inline(always)]
    pub fn get_mut_raw<T: Pod + Zeroable>(&mut self, offset: usize) -> &mut T {
        bytemuck::from_bytes_mut(&mut self.data[offset..offset + size_of::<T>()])
    }
}
