use std::sync::Arc;

use crate::DynLayout;

#[derive(Clone, Default, Debug, PartialEq, Hash)]
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
    UVec2,
    UVec3,
    UVec4,
    IVec2,
    IVec3,
    IVec4,
    Vec2,
    Vec3,
    Vec4,
    Mat2,
    Mat3,
    Mat4,
    Quat,
    DVec2,
    DVec3,
    DVec4,
    DMat2,
    DMat3,
    DMat4,
    DAffine2,
    DAffine3,
    Struct(Arc<DynLayout>),
}

impl BaseType {
    pub fn rust_base_type(&self) -> bool {
        match &self {
            BaseType::None => false, // This is actually a zst, maybe shouldn't exist or be named differently.
            BaseType::U8 => true,
            BaseType::U16 => true,
            BaseType::U32 => true,
            BaseType::U64 => true,
            BaseType::U128 => true,
            BaseType::I8 => true,
            BaseType::I16 => true,
            BaseType::I32 => true,
            BaseType::I64 => true,
            BaseType::I128 => true,
            BaseType::F32 => true,
            BaseType::F64 => true,
            BaseType::UVec2 => false,
            BaseType::UVec3 => false,
            BaseType::UVec4 => false,
            BaseType::IVec2 => false,
            BaseType::IVec3 => false,
            BaseType::IVec4 => false,
            BaseType::Vec2 => false,
            BaseType::Vec3 => false,
            BaseType::Vec4 => false,
            BaseType::Mat2 => false,
            BaseType::Mat3 => false,
            BaseType::Mat4 => false,
            BaseType::Quat => false,
            BaseType::DVec2 => false,
            BaseType::DVec3 => false,
            BaseType::DVec4 => false,
            BaseType::DMat2 => false,
            BaseType::DMat3 => false,
            BaseType::DMat4 => false,
            BaseType::DAffine2 => false,
            BaseType::DAffine3 => false,
            BaseType::Struct(_) => false,
        }
    }
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
    glam::UVec2 => UVec2,
    glam::UVec3 => UVec3,
    glam::UVec4 => UVec4,
    glam::IVec2 => IVec2,
    glam::IVec3 => IVec3,
    glam::IVec4 => IVec4,
    glam::Vec2 => Vec2,
    glam::Vec3 => Vec3,
    glam::Vec4 => Vec4,
    glam::Mat2 => Mat2,
    glam::Mat3 => Mat3,
    glam::Mat4 => Mat4,
    glam::Quat => Quat,
    glam::DVec2 => DVec2,
    glam::DVec3 => DVec3,
    glam::DVec4 => DVec4,
    glam::DMat2 => DMat2,
    glam::DMat3 => DMat3,
    glam::DMat4 => DMat4,
    glam::DAffine2 => DAffine2,
    glam::DAffine3 => DAffine3
);
