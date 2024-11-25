pub use dyn_pod_struct_derive::DynLayout;
use std::{
    fmt::{self, Display},
    sync::Arc,
};

use difference::{Changeset, Difference};
use fxhash::FxHashMap;

use crate::{base_type::BaseType, dyn_struct::DynField};

#[derive(Clone, Debug, PartialEq, Default)]
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

impl std::hash::Hash for DynLayout {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.fields.hash(state);
        //self.fields_hash.hash(state); We can skip the fields_hash since this is duplicate data
        self.size.hash(state);
    }
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

    /// Append type to end of layout. Assumes no padding between last type and the one being added.
    /// Assumes ty is not a struct. (Does not setup absolute offsets for struct fields down the hierarchy)
    pub fn append_type(&mut self, name: &str, ty: BaseType) {
        let new_field = DynField {
            offset: self.size as u32,
            ty,
        };
        self.size += new_field.ty.size_of();
        self.fields.push((name.to_string(), new_field.clone()));
        self.fields_hash.insert(name.to_string(), new_field);
    }

    /// Append type to end of layout. Assumes no padding between last type and the one being added.
    /// Helper for easily making a New Type. Assumes base_ty is not a struct.
    /// (Does not setup absolute offsets for struct fields down the hierarchy, only for the new type)
    /// Access the new type data with ["parent", "inner"]
    /// (not using 0 here since that would be incompatible with shader languages)
    pub fn append_new_type(&mut self, name: &str, base_ty: BaseType, type_name: &str) {
        let offset = self.size as u32;
        let new_field = DynField {
            offset,
            ty: BaseType::Struct(Arc::new(DynLayout::new(
                type_name,
                base_ty.size_of(),
                vec![(
                    "inner".to_string(),
                    DynField {
                        offset,
                        ty: base_ty,
                    },
                )],
            ))),
        };
        self.size += new_field.ty.size_of();
        self.fields.push((name.to_string(), new_field.clone()));
        self.fields_hash.insert(name.to_string(), new_field);
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

    #[inline(always)]
    pub fn get_path(&self, path: &[&str]) -> Option<&DynField> {
        let mut layout = self;
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
            Some(field)
        } else {
            None
        }
    }
}

pub trait HasDynLayout {
    /// Creating a layout can be slow, prefer creating a layout once and reusing.
    fn dyn_layout() -> Arc<DynLayout>;
}
