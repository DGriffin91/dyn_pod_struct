use std::sync::Arc;

use boomphf::Mphf;
use bytemuck::{Pod, Zeroable};

// Looking up by index in IndexMap should be similar perf to indexing into a Vec
// Looking up by string should be similar perf to HashMap.
// Consider using a runtime PHF map since this will be immutable after building.
// https://crates.io/crates/ph ?

#[derive(Clone, Default, Debug)]
pub struct DynField {
    pub offset: u32, // In bytes
    pub size: u32,   // In bytes
    pub struct_: Option<Arc<DynStructLayout>>,
}

#[derive(Clone, Debug)]
pub struct DynStructLayout {
    pub phf: Mphf<String>,
    /// Fields are laid out in phf order for fastest access.
    /// Use fields_order to get fields in struct order.
    pub fields: Vec<DynField>,
    pub field_names: Vec<String>,
}

impl DynStructLayout {
    pub fn new(fields: Vec<(String, DynField)>) -> Self {
        let names = fields
            .iter()
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>();
        let phf = Mphf::new(1.7, &names);

        let mut list = vec![DynField::default(); fields.len()];
        let mut field_names = vec![String::new(); fields.len()];
        fields.iter().enumerate().for_each(|(i, (name, field))| {
            let index = phf.hash(name) as usize;
            list[index] = field.clone();
            field_names[i] = name.to_string();
        });

        DynStructLayout {
            phf,
            fields: list,
            field_names,
        }
    }
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
            let index = layout?.phf.try_hash(*s)? as usize;
            field = Some(&layout?.fields[index]);
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
