#[cfg(test)]
mod tests {

    use bytemuck::{cast_slice, Zeroable};
    use dyn_struct::{DynStructLayout, HasDynStructLayout};
    use dyn_struct_derive::DynLayout;
    use glam::{Mat4, Vec3};
    use naga::{
        back::spv,
        front::wgsl,
        valid::{Capabilities, ValidationFlags, Validator},
    };

    #[repr(C)]
    #[derive(DynLayout, Copy, Clone, Default, Zeroable, Debug, PartialEq)]
    pub struct NestedStruct {
        pub a: Vec3,
        pub b: f32,
        pub c: Vec3,
        pub d: u32,
    }

    #[repr(C)]
    #[derive(DynLayout, Copy, Clone, Default, Zeroable, Debug, PartialEq)]
    pub struct InstanceData {
        pub local_to_world: Mat4,
        pub world_to_local: Mat4,
        pub previous_local_to_world: Mat4,
        pub aabb_min: Vec3,
        pub material_index: u32,
        pub aabb_max: Vec3,
        pub bindpose_start: u32,
        pub nested: NestedStruct,
        pub index_count: u32,
        pub first_index: u32,
        pub vertex_count: u32,
        pub first_vertex: u32,
    }

    #[test]
    fn test_get_simple_field() {
        let module = wgsl::parse_str(
            r#"
                struct NestedStruct {
                    a: vec3<f32>,
                    b: f32,
                    c: vec3<f32>,
                    d: u32,
                }

                struct InstanceData {
                    local_to_world: mat4x4<f32>,
                    world_to_local: mat4x4<f32>,
                    previous_local_to_world: mat4x4<f32>,
                    aabb_min: vec3<f32>,
                    material_index: u32,
                    aabb_max: vec3<f32>,
                    bindpose_start: u32,
                    nested: NestedStruct,
                    index_count: u32,
                    first_index: u32,
                    vertex_count: u32,
                    first_vertex: u32,
                };

                // For InstanceData to show up in spirq we need to use it in a binding.
                @group(0) @binding(0)
                var<storage, read_write> instances: array<InstanceData>;

                @compute @workgroup_size(1, 1, 1)
                fn main() {
                    return; 
                }
                "#,
        )
        .unwrap();

        let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());
        let module_info = validator.validate(&module).unwrap();

        let spirv = spv::write_vec(
            &module,
            &module_info,
            &spv::Options {
                lang_version: (1, 5),
                ..Default::default()
            },
            None,
        )
        .unwrap();

        let wgsl_layout = DynStructLayout::from_spirv(cast_slice(&spirv), "InstanceData").unwrap();
        let rust_layout = InstanceData::dyn_struct_layout();
        assert_eq!(wgsl_layout, rust_layout);
        dbg!(&wgsl_layout.name);
        dbg!(&wgsl_layout
            .fields
            .iter()
            .map(|(n, t)| format!("{n}: {:?},", t.ty))
            .collect::<Vec<_>>());
    }
}
