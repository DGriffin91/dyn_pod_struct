use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use fxhash::FxHashMap;

#[derive(Clone, Default, Debug)]
pub enum BaseType {
    #[default]
    None,
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
    F32,
    F64,
    Bool,
    Vec2,
    Vec3,
    Vec4,
    Mat2,
    Mat3,
    Mat4,
    Quat,
    Affine2,
    Affine3A,
    DVec2,
    DVec3,
    DVec4,
    DMat2,
    DMat3,
    DMat4,
    DAffine2,
    DAffine3,
    Struct(Arc<DynStructLayout>),
}

pub trait BaseTypeInfo {
    const SIZE: usize;
}

macro_rules! impl_base_type_info {
    ($($t:ty => $variant:ident),* $(,)?) => {
        $(
            impl BaseTypeInfo for $t {
                const SIZE: usize = std::mem::size_of::<$t>();
            }

            impl IntoBaseType for $t {
                #[inline(always)]
                fn into_base_type() -> BaseType {
                    BaseType::$variant
                }
            }
        )*
        impl BaseType {
            #[inline(always)]
            pub const fn const_size_of(&self) -> Option<usize> {
                match self {
                    $(
                        BaseType::$variant => Some(<$t as BaseTypeInfo>::SIZE),
                    )*
                    BaseType::None => Some(0),
                    BaseType::Struct(_) => None,
                }
            }
            #[inline(always)]
            pub fn size_of(&self) -> usize {
                match self {
                    $(
                        BaseType::$variant => <$t as BaseTypeInfo>::SIZE,
                    )*
                    BaseType::None => 0,
                    BaseType::Struct(s) => s.size as usize,
                }
            }
        }
    };
}

pub trait IntoBaseType {
    fn into_base_type() -> BaseType;
}

#[inline(always)]
pub fn get_base_type<T: IntoBaseType>() -> BaseType {
    T::into_base_type()
}

impl_base_type_info!(
    u8 => U8,
    u16 => U16,
    u32 => U32,
    u64 => U64,
    u128 => U128,
    i8 => I8,
    i16 => I16,
    i32 => I32,
    i64 => I64,
    i128 => I128,
    f32 => F32,
    f64 => F64,
    bool => Bool,
    glam::Vec2 => Vec2,
    glam::Vec3 => Vec3,
    glam::Vec4 => Vec4,
    glam::Mat2 => Mat2,
    glam::Mat3 => Mat3,
    glam::Mat4 => Mat4,
    glam::Quat => Quat,
    glam::Affine2 => Affine2,
    glam::Affine3A => Affine3A,
    glam::DVec2 => DVec2,
    glam::DVec3 => DVec3,
    glam::DVec4 => DVec4,
    glam::DMat2 => DMat2,
    glam::DMat3 => DMat3,
    glam::DMat4 => DMat4,
    glam::DAffine2 => DAffine2,
    glam::DAffine3 => DAffine3
);

#[derive(Clone, Default, Debug)]
pub struct DynField {
    pub offset: u32, // In bytes
    // Spare 32 bits of padding here, could cache size here. Is faster than checking size of type with .size_of()
    pub ty_: BaseType,
}

#[derive(Clone, Debug)]
pub struct DynStructLayout {
    pub name: String,
    // Fields in struct order
    pub fields: Vec<(String, DynField)>,
    /// HashMap for fast hash lookup.
    // (IndexMap & FxIndexMap seemed much slower for hash retrieval, also tried boomphf and it was also slower for hash retrieval)
    // Most the wasted space here is just the String, the DynField is only 16 bytes.
    pub fields_hash: FxHashMap<String, DynField>,
    /// Size of this struct in bytes
    pub size: usize,
}

impl DynStructLayout {
    pub fn new(name: &str, size: usize, fields: Vec<(String, DynField)>) -> Self {
        let mut field_hash = FxHashMap::default();
        fields.iter().for_each(|(name, field)| {
            field_hash.insert(name.clone(), field.clone());
        });
        DynStructLayout {
            name: name.to_string(),
            fields,
            fields_hash: field_hash,
            size,
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
            // If this shouldn't be debug, bring back DynField size, field.ty_.size_of() is too slow
            debug_assert_eq!(size_of::<T>(), field.ty_.size_of() as usize);
            Some(self.get_raw(field.offset as usize))
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn get_mut<T: Pod + Zeroable>(&mut self, path: &[&str]) -> Option<&mut T> {
        if let Some(field) = self.get_path::<T>(path) {
            // If this shouldn't be debug, bring back DynField size, field.ty_.size_of() is too slow
            debug_assert_eq!(size_of::<T>(), field.ty_.size_of() as usize);
            Some(self.get_mut_raw(field.offset as usize))
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn get_path<T: Pod + Zeroable>(&self, path: &[&str]) -> Option<&DynField> {
        let mut layout = &self.layout;
        let mut field = None;

        let last = path.len() - 1;

        for (i, s) in path.iter().enumerate() {
            field = layout.fields_hash.get(*s);
            if let BaseType::Struct(field_layout) = &field?.ty_ {
                layout = &field_layout;
            } else if last != i {
                // If this isn't the end of the path, a struct is expected.
                return None;
            }
        }

        if let Some(field) = field {
            // If this shouldn't be debug, bring back DynField size, field.ty_.size_of() is too slow
            debug_assert_eq!(size_of::<T>(), field.ty_.size_of() as usize);
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
