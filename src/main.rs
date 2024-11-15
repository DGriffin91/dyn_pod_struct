use std::time::Instant;

use bytemuck::{Pod, Zeroable};
use dyn_struct_core::{DynStruct, HasDynStructLayout};
use dyn_struct_derive::DynLayout;
use glam::{Affine3A, Vec3};

#[repr(C)]
#[derive(DynLayout, Copy, Clone, Default, Zeroable, Debug, PartialEq)]
pub struct InstanceData {
    pub local_to_world: Affine3A,          // model
    pub world_to_local: Affine3A,          // inverse model
    pub previous_local_to_world: Affine3A, // previous model
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
    let size = 30_000_000;
    let layout = InstanceData::dyn_struct_layout();
    dbg!(&layout.name);
    dbg!(&layout
        .fields
        .iter()
        .map(|(n, t)| format!("{n}: {:?},", t.ty_))
        .collect::<Vec<_>>());

    let start = Instant::now();
    let instances = (0..size)
        .map(|i| {
            let mut data = InstanceData::default();
            data.first_index = i;
            data
        })
        .collect::<Vec<_>>();
    println!(
        "{:.2}\tCreate native",
        start.elapsed().as_secs_f32() * 1000.0
    );
    let start = Instant::now();
    let sum: u32 = instances.iter().map(|instance| instance.first_index).sum();
    println!(
        "{:.2}\tAccess native ({sum})",
        start.elapsed().as_secs_f32() * 1000.0
    );

    let start = Instant::now();
    let instances = (0..size)
        .map(|i| {
            let mut data = InstanceData::default();
            data.first_index = i;
            DynStruct::from_struct_with_layout(&data, &layout)
        })
        .collect::<Vec<_>>();
    println!("{:.2}\tCreate", start.elapsed().as_secs_f32() * 1000.0);

    //std::thread::sleep(std::time::Duration::from_millis(1000));

    let start = Instant::now();
    let sum: u32 = instances
        .iter()
        .map(|instance| instance.get::<u32>(&["first_index"]).unwrap())
        .sum();
    println!(
        "{:.2}\tAccess dyn ({sum})",
        start.elapsed().as_secs_f32() * 1000.0
    );
    //std::thread::sleep(std::time::Duration::from_millis(1000));
    let offset = instances[0]
        .get_path::<u32>(&["first_index"])
        .unwrap()
        .offset as usize;
    let start = Instant::now();
    let sum: u32 = instances
        .iter()
        .map(|instance| instance.get_raw::<u32>(offset))
        .sum();
    println!(
        "{:.2}\tAccess dyn fast ({sum})",
        start.elapsed().as_secs_f32() * 1000.0
    );
}
