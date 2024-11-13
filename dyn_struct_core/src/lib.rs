use std::{borrow::Cow, sync::Arc};

use bytemuck::{Pod, Zeroable};

use indexmap::IndexMap;

// Looking up by index in IndexMap should be similar perf to indexing into a Vec
// Looking up by string should be similar perf to HashMap.
// Consider using a runtime PHF map since this will be immutable after building.
// https://crates.io/crates/ph ?

#[derive(Clone, Default, Debug, PartialEq)]
pub struct DynField {
    pub offset: u32, // In bytes
    pub size: u32,   // In bytes
    pub struct_: Option<Arc<DynStructLayout>>,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct DynStructLayout {
    pub fields: IndexMap<String, DynField>,
}

pub struct DynStruct {
    pub data: Vec<u8>,
    pub layout: Arc<DynStructLayout>,
}

impl DynStruct {
    #[inline(always)]
    pub fn get<T: Pod + Zeroable>(&self, path: &[&str]) -> Option<&T> {
        if let Some(field) = self.get_path::<T>(path) {
            assert_eq!(size_of::<T>(), field.size as usize);
            Some(self.get_raw(field.offset as usize))
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn get_mut<T: Pod + Zeroable>(&mut self, path: &[&str]) -> Option<&mut T> {
        if let Some(field) = self.get_path::<T>(path) {
            assert_eq!(size_of::<T>(), field.size as usize);
            Some(self.get_mut_raw(field.offset as usize))
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn get_path<T: Pod + Zeroable>(&self, path: &[&str]) -> Option<&DynField> {
        let mut layout = Some(&self.layout);
        let mut field = None;

        for s in path {
            field = layout?.fields.get(*s);
            layout = field?.struct_.as_ref();
        }

        if let Some(field) = field {
            assert_eq!(size_of::<T>(), field.size as usize);
            Some(field)
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

    #[inline(always)]
    pub fn cast_unchecked<T: Pod + Zeroable>(&self) -> &T {
        bytemuck::from_bytes(&self.data[..])
    }

    #[inline(always)]
    pub fn cast<T: Pod + Zeroable>(&self, _layout: Arc<DynStructLayout>) -> &T {
        // TODO check some hash in the layout regarding the offsets/layout
        bytemuck::from_bytes(&self.data[..])
    }

    #[inline(always)]
    pub fn cast_mut_unchecked<T: Pod + Zeroable>(&mut self) -> &mut T {
        bytemuck::from_bytes_mut(&mut self.data[..])
    }

    #[inline(always)]
    pub fn cast_mut<T: Pod + Zeroable>(&mut self, _layout: Arc<DynStructLayout>) -> &mut T {
        // TODO check some hash in the layout regarding the offsets/layout
        bytemuck::from_bytes_mut(&mut self.data[..])
    }
}

pub trait HasDynStructLayout {
    fn dyn_struct_layout() -> Arc<DynStructLayout>;
}
