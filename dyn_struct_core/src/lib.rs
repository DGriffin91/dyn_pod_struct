use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use fxhash::FxHashMap;

#[derive(Clone, Default, Debug)]
pub struct DynField {
    pub offset: u32, // In bytes
    pub size: u32,   // In bytes
    pub struct_: Option<Arc<DynStructLayout>>,
}

#[derive(Clone, Debug)]
pub struct DynStructLayout {
    pub name: String,
    // Fields in struct order
    pub fields: Vec<(String, DynField)>,
    /// HashMap for fast hash lookup.
    // (IndexMap seemed much slower for hash retrieval, also tried boomphf and it was also slower for hash retrieval)
    // Most the wasted space here is just the String, the DynField is only 16 bytes.
    pub fields_hash: FxHashMap<String, DynField>,
}

impl DynStructLayout {
    pub fn new(name: &str, fields: Vec<(String, DynField)>) -> Self {
        let mut field_hash = FxHashMap::default();
        fields.iter().for_each(|(name, field)| {
            field_hash.insert(name.clone(), field.clone());
        });
        DynStructLayout {
            name: name.to_string(),
            fields,
            fields_hash: field_hash,
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
            // non-hashed iterative lookup example.
            // On a 11 field struct this is still faster than an index map, but slower than FxHashMap
            //for (cand_str, cand_field) in &layout?.fields {
            //    if cand_str == s {
            //        field = Some(cand_field);
            //        layout = cand_field.struct_.as_ref();
            //        break;
            //    }
            //}

            field = layout?.fields_hash.get(*s);
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
    pub fn cast<T: Pod + Zeroable>(&self) -> &T {
        bytemuck::from_bytes(&self.data[..])
    }

    #[inline(always)]
    pub fn cast_mut<T: Pod + Zeroable>(&mut self) -> &mut T {
        bytemuck::from_bytes_mut(&mut self.data[..])
    }
}

pub trait HasDynStructLayout {
    fn dyn_struct_layout() -> Arc<DynStructLayout>;
}
