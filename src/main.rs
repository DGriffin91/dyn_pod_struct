use std::time::Instant;

use bytemuck::{bytes_of, Pod, Zeroable};
use dyn_struct_core::{DynStruct, HasDynStructLayout};
use dyn_struct_derive::DynLayout;
use glam::{Affine3A, Vec3};

#[repr(C)]
#[derive(DynLayout, Copy, Clone, Default, Zeroable, Debug, PartialEq)]
pub struct DynInstanceData {
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

unsafe impl Pod for DynInstanceData {}

fn main() {
    let size = 10_000_000;

    use boomphf::*;

    // sample set of objects
    let possible_objects = vec![
        String::from("local_to_world"),
        String::from("world_to_local"),
        String::from("previous_local_to_world"),
        String::from("aabb_min"),
        String::from("material_index"),
        String::from("aabb_max"),
        String::from("bindpose_start"),
        String::from("index_count"),
        String::from("first_index"),
        String::from("vertex_count"),
        String::from("first_vertex"),
        String::from("0"),
        String::from("1"),
        String::from("2"),
        String::from("3"),
        String::from("4"),
        String::from("5"),
        String::from("6"),
        String::from("7"),
        String::from("8"),
        String::from("9"),
    ];
    let n = possible_objects.len();

    // generate a minimal perfect hash function of these items
    let phf = Mphf::new(1.7, &possible_objects.clone());

    // Get hash value of all objects
    let mut hashes = Vec::new();
    for v in possible_objects {
        hashes.push(phf.hash(&v));
    }
    dbg!(&hashes);
    hashes.sort();

    // Expected hash output is set of all integers from 0..n
    let expected_hashes: Vec<u64> = (0..n as u64).collect();
    assert!(hashes == expected_hashes);

    //if true {
    //    return;
    //}

    //-----------------------------------------------
    //-----------------------------------------------
    //-----------------------------------------------
    //-----------------------------------------------
    //-----------------------------------------------

    let start = Instant::now();
    let instances = (0..size)
        .map(|i| {
            let mut data = DynInstanceData::default();
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

    let layout = DynInstanceData::dyn_struct_layout();

    let start = Instant::now();
    let instances = (0..size)
        .map(|i| {
            let mut data = DynInstanceData::default();
            data.first_index = i;
            DynStruct {
                data: bytes_of(&data).to_vec(),
                layout: layout.clone(),
            }
        })
        .collect::<Vec<_>>();
    println!("{:.2}\tCreate", start.elapsed().as_secs_f32() * 1000.0);
    let start = Instant::now();
    let sum: u32 = instances
        .iter()
        .map(|instance| instance.get::<u32>(&["first_index"]).unwrap())
        .sum();
    println!(
        "{:.2}\tAccess dyn ({sum})",
        start.elapsed().as_secs_f32() * 1000.0
    );
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
