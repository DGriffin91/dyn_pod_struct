pub mod spirv;

use std::{
    fmt::{self, Display},
    sync::Arc,
};

use bytemuck::{bytes_of, Pod, Zeroable};
use difference::{Changeset, Difference};
use fxhash::FxHashMap;

#[derive(Clone, Default, Debug, PartialEq)]
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
    Affine2,
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
            BaseType::Bool => true, // Warning c bool and shader bool will probably never match
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
            BaseType::Affine2 => false,
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
    bool => Bool,
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
    glam::Affine2 => Affine2,
    glam::DVec2 => DVec2,
    glam::DVec3 => DVec3,
    glam::DVec4 => DVec4,
    glam::DMat2 => DMat2,
    glam::DMat3 => DMat3,
    glam::DMat4 => DMat4,
    glam::DAffine2 => DAffine2,
    glam::DAffine3 => DAffine3
);

#[derive(Clone, Default, Debug, PartialEq)]
pub struct DynField {
    pub offset: u32, // In bytes
    // Spare 32 bits of padding here, could cache size here. Is faster than checking size of type with .size_of()
    pub ty: BaseType,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DynLayout {
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

impl Display for DynLayout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.format_with_offsets(0, f)
    }
}

pub fn diff_display<T: Display, U: Display>(a: T, b: U) {
    // https://github.com/johannhof/difference.rs/blob/master/examples/github-style.rs

    let text1 = format!("{a}");
    let text2 = format!("{b}");

    // Compare both texts, the third parameter defines the split level.
    let Changeset { diffs, .. } = Changeset::new(&text1, &text2, "\n");

    let mut t = term::stdout().unwrap();

    for i in 0..diffs.len() {
        match diffs[i] {
            Difference::Same(ref x) => {
                t.reset().unwrap();
                for line in x.split("\n") {
                    writeln!(t, " {}", line).unwrap();
                }
            }
            Difference::Add(ref x) => {
                match diffs[i - 1] {
                    Difference::Rem(ref y) => {
                        t.fg(term::color::GREEN).unwrap();
                        write!(t, "+").unwrap();
                        let Changeset { diffs, .. } = Changeset::new(y, x, " ");
                        for c in diffs {
                            match c {
                                Difference::Same(ref z) => {
                                    t.fg(term::color::GREEN).unwrap();
                                    write!(t, "{}", z).unwrap();
                                    write!(t, " ").unwrap();
                                }
                                Difference::Add(ref z) => {
                                    t.fg(term::color::WHITE).unwrap();
                                    t.bg(term::color::GREEN).unwrap();
                                    write!(t, "{}", z).unwrap();
                                    t.reset().unwrap();
                                    write!(t, " ").unwrap();
                                }
                                _ => (),
                            }
                        }
                        writeln!(t, "").unwrap();
                    }
                    _ => {
                        t.fg(term::color::BRIGHT_GREEN).unwrap();
                        writeln!(t, "+{}", x).unwrap();
                    }
                };
            }
            Difference::Rem(ref x) => {
                t.fg(term::color::RED).unwrap();
                writeln!(t, "-{}", x).unwrap();
            }
        }
    }
    t.reset().unwrap();
    t.flush().unwrap();
}

impl DynLayout {
    pub fn new(name: &str, size: usize, fields: Vec<(String, DynField)>) -> Self {
        let mut field_hash = FxHashMap::default();
        fields.iter().for_each(|(name, field)| {
            field_hash.insert(name.clone(), field.clone());
        });
        DynLayout {
            name: name.to_string(),
            fields,
            fields_hash: field_hash,
            size,
        }
    }

    pub fn format_with_offsets(&self, depth: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let padding = " ".repeat(depth * 4 + 14);
        if depth == 0 {
            writeln!(f, "  Size Offset (bytes)")?;
            writeln!(f, "-----------------------")?;
            let size = self.size;
            let offset = 0;
            write!(f, "{size:>6} {offset:>6}  ")?;
        }
        writeln!(f, "{}", self.name)?;
        writeln!(f, "{padding} {{")?;
        for (field_name, field) in &self.fields {
            let padding = " ".repeat((depth + 1) * 4);
            let size = field.ty.size_of();
            let offset = field.offset;
            if let BaseType::Struct(layout) = &field.ty {
                write!(f, "{size:>6} {offset:>6}  {padding}{field_name}: ")?;
                layout.format_with_offsets(depth + 1, f)?;
            } else {
                let mut ty_name = format!("{:?}", &field.ty);
                if field.ty.rust_base_type() {
                    ty_name = ty_name.to_lowercase();
                }
                writeln!(f, "{size:>6} {offset:>6}  {padding}{field_name}: {ty_name}")?;
            }
        }
        let padding = " ".repeat(depth * 4 + 14);
        writeln!(f, "{padding} }}")
    }
}

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
        assert_eq!(size_of::<T>(), layout.size);
        DynStruct {
            data: bytes_of(data).to_vec(),
            layout: layout.clone(),
        }
    }

    #[inline(always)]
    pub fn get<T: Pod + Zeroable>(&self, path: &[&str]) -> Option<&T> {
        if let Some(field) = self.get_path::<T>(path) {
            // If this shouldn't be debug, bring back DynField size, field.ty_.size_of() is too slow
            debug_assert_eq!(size_of::<T>(), field.ty.size_of());
            Some(self.get_raw(field.offset as usize))
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn get_mut<T: Pod + Zeroable>(&mut self, path: &[&str]) -> Option<&mut T> {
        if let Some(field) = self.get_path::<T>(path) {
            // If this shouldn't be debug, bring back DynField size, field.ty_.size_of() is too slow
            debug_assert_eq!(size_of::<T>(), field.ty.size_of());
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
            if let BaseType::Struct(field_layout) = &field?.ty {
                layout = field_layout;
            } else if last != i {
                // If this isn't the end of the path, a struct is expected.
                return None;
            }
        }

        if let Some(field) = field {
            // If this shouldn't be debug, bring back DynField size, field.ty_.size_of() is too slow
            debug_assert_eq!(size_of::<T>(), field.ty.size_of());
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
}

pub trait HasDynLayout {
    /// Creating a layout can be slow, prefer creating a layout once and reusing.
    fn dyn_layout() -> Arc<DynLayout>;
}
