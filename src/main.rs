use std::hint::black_box;

use bevy::{prelude::*, reflect::DynamicStruct};

use bytemuck::{Pod, Zeroable};
use dyn_pod_struct::{timeit, HasDynLayout, TrackedDynStruct};
use dyn_pod_struct_derive::DynLayout;
use glam::{Mat4, Vec3};

#[repr(C)]
#[derive(DynLayout, Reflect, Copy, Clone, Default, Zeroable, Debug, PartialEq)]
pub struct InstanceData {
    pub local_to_world: Mat4,          // model
    pub world_to_local: Mat4,          // inverse model
    pub previous_local_to_world: Mat4, // previous model
    pub aabb_min: Vec3,
    pub material_index: u32,
    pub aabb_max: Vec3,
    pub bindpose_start: u32,
    pub index_count: u32,
    pub first_index: u32,
    pub vertex_count: u32,
    pub first_vertex: u32,
}

unsafe impl Pod for InstanceData {}

fn main() {
    let size = 1_000_000;
    let layout = InstanceData::dyn_layout();
    println!("{layout}");
    println!("size: {size}");

    timeit!["Create native",
    let mut native_instances = black_box((0..size)
        .map(|i| InstanceData {
            first_index: i,
            ..Default::default()
        })
        .collect::<Vec<_>>());
    ];

    timeit!["Create TrackedDynStructs",
    let mut instances = black_box((0..size)
        .map(|i| {
            let data = InstanceData {
                first_index: i,
                ..Default::default()
            };
            TrackedDynStruct::new(&data, &layout, 4, false)
        })
        .collect::<Vec<_>>());
    ];

    timeit!["Create Bevy DynamicStructs",
    let mut bevy_dyn_struct = black_box((0..size)
        .map(|i| {
            let mut st = DynamicStruct::default();
            st.insert("local_to_world", Mat4::default());
            st.insert("world_to_local", Mat4::default());
            st.insert("previous_local_to_world", Mat4::default());
            st.insert("aabb_min", Vec3::default());
            st.insert("material_index", u32::default());
            st.insert("aabb_max", Vec3::default());
            st.insert("bindpose_start", u32::default());
            st.insert("index_count", u32::default());
            st.insert("first_index", i);
            st.insert("vertex_count", u32::default());
            st.insert("first_vertex", u32::default());
            st
        })
        .collect::<Vec<_>>());
    ];

    println!();

    timeit!["Access Native",
    let native_sum: u64 = black_box(native_instances
        .iter()
        .map(|instance| instance.first_index as u64)
        .sum());
    ];

    timeit!["Access TrackedDynStructs",
    let sum: u64 = black_box(instances
        .iter()
        .map(|instance| *instance.get::<u32>(&["first_index"]).unwrap() as u64)
        .sum());
    assert_eq!(native_sum, sum);
    ];

    timeit!["Access TrackedDynStructs fast",
    let offset = instances[0]
        .dyn_struct
        .layout
        .get_path(&["first_index"])
        .unwrap()
        .offset as usize;
    let sum: u64 = black_box(instances
        .iter()
        .map(|instance| *instance.get_raw::<u32>(offset) as u64)
        .sum());
    assert_eq!(native_sum, sum);
    ];

    timeit!["Access bevy reflect TrackedDynStructs",
    let sum: u64 = black_box(instances
        .iter()
        .map(|instance| *instance.get_field::<u32>("first_index").unwrap() as u64)
        .sum());
    assert_eq!(native_sum, sum);
    ];

    timeit!["Access bevy reflect native",
    let sum: u64 = black_box(native_instances
        .iter()
        .map(|instance| *instance.get_field::<u32>("first_index").unwrap() as u64)
        .sum());
    assert_eq!(native_sum, sum);
    ];

    timeit!["Access bevy reflect bevy DynamicStruct",
    let sum: u64 = black_box(bevy_dyn_struct
        .iter()
        .map(|instance| *instance.get_field::<u32>("first_index").unwrap() as u64)
        .sum());
    assert_eq!(native_sum, sum);
    ];

    println!();

    timeit!["Modify native",
    black_box(native_instances.iter_mut().for_each(|instance| instance.first_index = 0 ));
    ];

    timeit!["Modify TrackedDynStructs",
    black_box(instances.iter_mut().for_each(|instance| *instance.get_mut::<u32>(&["first_index"]).unwrap() = 0 ));
    ];

    timeit!["Modify TrackedDynStructs fast",
    let offset = instances[0].dyn_struct.layout.get_path(&["first_index"]).unwrap().offset as usize;
    black_box(instances.iter_mut().for_each(|instance| *instance.get_mut_raw::<u32>(offset) = 0 ));
    ];

    timeit!["Modify bevy reflect TrackedDynStructs",
    black_box(instances.iter_mut().for_each(|instance| *instance.get_field_mut::<u32>("first_index").unwrap() = 0));
    ];

    timeit!["Modify bevy reflect native",
    black_box(native_instances.iter_mut().for_each(|instance| *instance.get_field_mut::<u32>("first_index").unwrap() = 0));
    ];

    timeit!["Modify bevy reflect DynamicStruct",
    black_box(bevy_dyn_struct.iter_mut().for_each(|instance| *instance.get_field_mut::<u32>("first_index").unwrap() = 0));
    ];
}
