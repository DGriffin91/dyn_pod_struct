#[cfg(test)]
mod tests {

    use bytemuck::{Pod, Zeroable};
    use dyn_struct::{DynStruct, HasDynStructLayout};
    use dyn_struct_derive::DynLayout;
    use glam::{ivec4, uvec4, vec4, IVec4, UVec4, Vec4};
    use std::fmt::Debug;

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
        pub p: Vec4,
        pub nested: NestedStruct,
        pub a1: u32,
        pub a2: u32,
        pub a3: u32,
        pub a4: u32,
        pub u: UVec4,
        pub i: IVec4,
    }

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
        let test_dyn = DynStruct::from_struct_with_layout(&data, &layout);

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

        //dbg!(&layout.fields);

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
            u: uvec4(5, 6, 7, 8),
            i: ivec4(-5, -6, -7, -8),
        };
        let test_dyn = DynStruct::from_struct_with_layout(&data, &layout);

        check_eq(&test_dyn, &["nested", "a"], 1u32);
        check_eq(&test_dyn, &["nested", "b"], 2.0f32);
        check_eq(&test_dyn, &["nested", "c"], 3u32);
        check_eq(&test_dyn, &["nested", "d"], 4u32);
        check_eq(&test_dyn, &["p"], vec4(1.0, 2.0, 3.0, 4.0));
        check_eq(&test_dyn, &["a1"], 1u32);
        check_eq(&test_dyn, &["a2"], 2u32);
        check_eq(&test_dyn, &["a3"], 3u32);
        check_eq(&test_dyn, &["a4"], 4u32);
        check_eq(&test_dyn, &["u"], uvec4(5, 6, 7, 8));
        check_eq(&test_dyn, &["i"], ivec4(-5, -6, -7, -8));
    }
}
