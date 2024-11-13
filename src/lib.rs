use bytemuck::{Pod, Zeroable};

use dyn_struct_derive::DynLayout;
use glam::Vec4;

#[repr(C)]
#[derive(DynLayout, Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct NestedStruct {
    pub a: u32,
    pub b: f32,
    pub c: u32,
    pub d: u32,
}

#[repr(C)]
#[derive(DynLayout, Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct MyStruct {
    pub nested: NestedStruct,
    pub b: f32,
    pub c: u32,
}

#[repr(C)]
#[derive(DynLayout, Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct MyStruct2 {
    pub nested: NestedStruct,
    pub p: Vec4,
    pub a1: u32,
    pub a2: u32,
    pub a3: u32,
    pub a4: u32,
}

#[cfg(test)]
mod tests {

    use super::*;

    use bytemuck::bytes_of;
    use dyn_struct_core::{DynStruct, HasDynStructLayout};
    use glam::vec4;
    use std::fmt::Debug;

    fn check_eq<T: PartialEq<T> + Pod + Debug>(test_dyn: &DynStruct, path: &[&str], v: T) {
        assert_eq!(*test_dyn.get::<T>(path).unwrap(), v);
    }

    #[test]
    fn test_get_simple_field() {
        let layout = MyStruct::dyn_struct_layout();
        let data = MyStruct {
            nested: NestedStruct {
                a: 1,
                b: 2.0,
                c: 3,
                d: 4,
            },
            b: 5.0,
            c: 6,
        };
        let test_dyn = DynStruct {
            data: bytes_of(&data).to_vec(),
            layout,
        };

        check_eq(&test_dyn, &["nested", "a"], 1u32);
        check_eq(&test_dyn, &["nested", "b"], 2.0f32);
        check_eq(&test_dyn, &["nested", "c"], 3u32);
        check_eq(&test_dyn, &["nested", "d"], 4u32);
        check_eq(
            &test_dyn,
            &["nested"],
            NestedStruct {
                a: 1,
                b: 2.0,
                c: 3,
                d: 4,
            },
        );
        check_eq(&test_dyn, &["b"], 5.0f32);
        check_eq(&test_dyn, &["c"], 6u32);

        let layout = MyStruct2::dyn_struct_layout();
        let data = MyStruct2 {
            nested: NestedStruct {
                a: 1,
                b: 2.0,
                c: 3,
                d: 4,
            },
            p: vec4(1.0, 2.0, 3.0, 4.0),
            a1: 1,
            a2: 2,
            a3: 3,
            a4: 4,
        };
        let test_dyn = DynStruct {
            data: bytes_of(&data).to_vec(),
            layout,
        };

        check_eq(&test_dyn, &["nested", "a"], 1u32);
        check_eq(&test_dyn, &["nested", "b"], 2.0f32);
        check_eq(&test_dyn, &["nested", "c"], 3u32);
        check_eq(&test_dyn, &["nested", "d"], 4u32);
        check_eq(&test_dyn, &["p"], vec4(1.0, 2.0, 3.0, 4.0));
        check_eq(&test_dyn, &["a1"], 1u32);
        check_eq(&test_dyn, &["a2"], 2u32);
        check_eq(&test_dyn, &["a3"], 3u32);
        check_eq(&test_dyn, &["a4"], 4u32);
    }
}
