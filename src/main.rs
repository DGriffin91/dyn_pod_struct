// Just a little temp bench before setting up something more realistic

use std::hint::black_box;

#[cfg(feature = "bevy_reflect")]
use bevy_reflect::{DynamicStruct, GetField, Reflect};

use dyn_pod_struct::{dyn_layout::HasDynLayout, tracked_dyn_struct::TrackedDynStruct};

use bytemuck::{Pod, Zeroable};
use dyn_pod_struct_derive::DynLayout;
use glam::{Mat4, Vec3};

#[repr(C)]
#[derive(DynLayout, Copy, Clone, Default, Zeroable, Debug, PartialEq)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
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

    #[cfg(feature = "bevy_reflect")]
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

    #[cfg(feature = "bevy_reflect")]
    timeit!["Access bevy reflect TrackedDynStructs",
    let sum: u64 = black_box(instances
        .iter()
        .map(|instance| *instance.get_field::<u32>("first_index").unwrap() as u64)
        .sum());
    assert_eq!(native_sum, sum);
    ];

    #[cfg(feature = "bevy_reflect")]
    timeit!["Access bevy reflect native",
    let sum: u64 = black_box(native_instances
        .iter()
        .map(|instance| *instance.get_field::<u32>("first_index").unwrap() as u64)
        .sum());
    assert_eq!(native_sum, sum);
    ];

    #[cfg(feature = "bevy_reflect")]
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

    #[cfg(feature = "bevy_reflect")]
    timeit!["Modify bevy reflect TrackedDynStructs",
    black_box(instances.iter_mut().for_each(|instance| *instance.get_field_mut::<u32>("first_index").unwrap() = 0));
    ];

    #[cfg(feature = "bevy_reflect")]
    timeit!["Modify bevy reflect native",
    black_box(native_instances.iter_mut().for_each(|instance| *instance.get_field_mut::<u32>("first_index").unwrap() = 0));
    ];

    #[cfg(feature = "bevy_reflect")]
    timeit!["Modify bevy reflect DynamicStruct",
    black_box(bevy_dyn_struct.iter_mut().for_each(|instance| *instance.get_field_mut::<u32>("first_index").unwrap() = 0));
    ];
}

use std::time::Duration;

// Temp
// from: https://github.com/DGriffin91/obvhs/blob/cfc8031fd8f86e4f784e0fe2777425f8b817409c/src/lib.rs#L143

/// A macro to measure and print the execution time of a block of code.
///
/// # Arguments
/// * `$label` - A string label to identify the code block being timed.
/// * `$($code:tt)*` - The code block whose execution time is to be measured.
///
/// # Usage
/// ```rust
/// use dyn_pod_struct::timeit;
/// timeit!["example",
///     // code to measure
/// ];
/// ```
///
/// # Note
/// The macro purposefully doesn't include a scope so variables don't need to
/// be passed out of it. This allows it to be trivially added to existing code.
#[macro_export]
#[doc(hidden)]
macro_rules! timeit {
    [$label:expr, $($code:tt)*] => {
        //#[cfg(feature = "timeit")]
        let timeit_start = std::time::Instant::now();
        $($code)*
        //#[cfg(feature = "timeit")]
        println!("{:>8} {}", format!("{}", $crate::PrettyDuration(timeit_start.elapsed())), $label);
    };
}

/// A wrapper struct for `std::time::Duration` to provide pretty-printing of durations.
#[doc(hidden)]
pub struct PrettyDuration(pub Duration);

impl std::fmt::Display for PrettyDuration {
    /// Durations are formatted as follows:
    /// - If the duration is greater than or equal to 1 second, it is formatted in seconds (s).
    /// - If the duration is greater than or equal to 1 millisecond but less than 1 second, it is formatted in milliseconds (ms).
    /// - If the duration is less than 1 millisecond, it is formatted in microseconds (µs).
    /// In the case of seconds & milliseconds, the duration is always printed with a precision of two decimal places.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let duration = self.0;
        if duration.as_secs() > 0 {
            let seconds =
                duration.as_secs() as f64 + f64::from(duration.subsec_nanos()) / 1_000_000_000.0;
            write!(f, "{:.2}s ", seconds)
        } else if duration.subsec_millis() > 0 {
            let milliseconds =
                duration.as_millis() as f64 + f64::from(duration.subsec_micros() % 1_000) / 1_000.0;
            write!(f, "{:.2}ms", milliseconds)
        } else {
            let microseconds = duration.as_micros();
            write!(f, "{}µs", microseconds)
        }
    }
}
