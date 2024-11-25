#[cfg(test)]
mod tests {

    use bytemuck::Zeroable;
    use dyn_pod_struct::dyn_layout::{DynLayout, HasDynLayout};
    use glam::{Mat4, Vec3};
    use hassle_rs::compile_hlsl;

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
        let spirv = compile_hlsl(
            "fragment.hlsl",
            r#"
                struct NestedStruct
                {
                    float3 a;                  
                    float b;              
                    float3 c;                  
                    uint d;    
                };

                struct InstanceData
                {
                    float4x4 local_to_world;         
                    float4x4 world_to_local;         
                    float4x4 previous_local_to_world;
                    float3 aabb_min;                  
                    uint material_index;              
                    float3 aabb_max;                  
                    uint bindpose_start;  
                    NestedStruct nested;   
                    uint index_count;
                    uint first_index;
                    uint vertex_count;
                    uint first_vertex;
                };

                // For InstanceData to show up in spirq we need to use it in a binding.
                [[vk::binding(0, 0)]]
                RWStructuredBuffer<InstanceData> instances : register(u0, space0);

                [numthreads(1, 1, 1)]
                void main() { 
                    return; // Because of -Od we don't need to actually do anything with the StructuredBuffer
                }
                "#,
            "main",
            "cs_6_5",
            &vec!["-spirv", "-Od"],
            &vec![],
        )
        .unwrap();
        let hlsl_layout = DynLayout::from_spirv(&spirv, "InstanceData").unwrap();
        let rust_layout = InstanceData::dyn_layout();
        assert_eq!(hlsl_layout, rust_layout);
        println!("{}", hlsl_layout);
    }
}
