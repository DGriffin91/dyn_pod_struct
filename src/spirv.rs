use std::sync::Arc;

use bytemuck::cast_slice;
use spirq::{var::Variable, ReflectConfig};

use crate::{BaseType, DynField, DynStructLayout};

impl DynStructLayout {
    pub fn from_spirv(spirv: &[u8], name: &str) -> Arc<DynStructLayout> {
        let entry_points = ReflectConfig::new()
            .spv(cast_slice::<u8, u32>(spirv))
            .ref_all_rscs(true)
            .reflect()
            .unwrap();

        let mut fields = Vec::new();

        for var in &entry_points[0].vars {
            if let Variable::Descriptor {
                name: _,
                desc_bind: _,
                desc_ty: _,
                ty,
                nbind: _,
            } = var
            {
                if let spirq::ty::Type::Struct(struct_type) = ty {
                    //dbg!(&struct_type.name);
                    //dbg!(&ty);
                    let Some(ref struct_name) = struct_type.name else {
                        continue;
                    };
                    // This will be like: type.RWStructuredBuffer.InstanceData
                    if struct_name.split(".").last() != Some(name) {
                        continue;
                    }

                    if let spirq::ty::Type::Array(inner_ty) = &struct_type.members[0].ty {
                        if let spirq::ty::Type::Struct(inner_struct_type) =
                            *inner_ty.element_ty.clone()
                        {
                            for member in &inner_struct_type.members {
                                // TODO detect padding and disallow?
                                let name = member
                                    .name
                                    .clone()
                                    .unwrap_or(format!("param_{}", fields.len()));
                                let offset = member.offset.unwrap() as u32;
                                let dyn_ty = spirq_ty_to_dyn(member);
                                //dbg!(&name, (&u32_offset, &dyn_ty));
                                fields.push((name, DynField { offset, ty: dyn_ty }));
                            }
                        } else {
                            panic!("Expected inner struct type {:?}", name)
                        }
                    } else {
                        panic!("Expected array type {:?}", name)
                    }
                } else {
                    panic!("Dyn reflect only support struct type. {:?}", name)
                }
                break;
            }
        }
        let mut total_size = 0;
        if let Some((_, last)) = fields.last() {
            total_size = last.offset as usize + last.ty.size_of();
        }

        Arc::new(DynStructLayout::new(name, total_size, fields))
    }
}

pub fn spirq_ty_to_dyn(member: &spirq::ty::StructMember) -> BaseType {
    let dyn_ty = match &member.ty {
        spirq::ty::Type::Scalar(scalar_type) => match *scalar_type {
            spirq::ty::ScalarType::Void => BaseType::None,
            spirq::ty::ScalarType::Boolean => BaseType::Bool,
            spirq::ty::ScalarType::Integer { bits, is_signed } => match bits {
                8 => match is_signed {
                    true => unimplemented!(),
                    false => BaseType::U8,
                },
                16 => match is_signed {
                    true => BaseType::I16,
                    false => BaseType::U16,
                },
                32 => match is_signed {
                    true => BaseType::I32,
                    false => BaseType::U32,
                },
                64 => match is_signed {
                    true => BaseType::I64,
                    false => BaseType::U64,
                },
                _ => unimplemented!(),
            },
            spirq::ty::ScalarType::Float { bits } => match bits {
                32 => BaseType::F32,
                64 => BaseType::F64,
                _ => unimplemented!(),
            },
        },
        spirq::ty::Type::Vector(vector_type) => match vector_type.scalar_ty {
            spirq::ty::ScalarType::Void => unimplemented!(),
            spirq::ty::ScalarType::Boolean => unimplemented!(),
            spirq::ty::ScalarType::Integer { bits, is_signed } => match bits {
                32 => match vector_type.nscalar {
                    2 => match is_signed {
                        true => BaseType::IVec2,
                        false => BaseType::UVec2,
                    },
                    3 => match is_signed {
                        true => BaseType::IVec3,
                        false => BaseType::UVec3,
                    },
                    4 => match is_signed {
                        true => BaseType::IVec4,
                        false => BaseType::UVec4,
                    },
                    _ => unimplemented!(),
                },
                _ => unimplemented!(),
            },
            spirq::ty::ScalarType::Float { bits } => match vector_type.nscalar {
                2 => match bits {
                    32 => BaseType::Vec2,
                    64 => BaseType::DVec2,
                    _ => unimplemented!(),
                },
                3 => match bits {
                    32 => BaseType::Vec3,
                    64 => BaseType::DVec3,
                    _ => unimplemented!(),
                },
                4 => match bits {
                    32 => BaseType::Vec4,
                    64 => BaseType::DVec4,
                    _ => unimplemented!(),
                },
                _ => unimplemented!(),
            },
        },
        spirq::ty::Type::Matrix(matrix_type) => match matrix_type.vector_ty.scalar_ty {
            spirq::ty::ScalarType::Void => unimplemented!(),
            spirq::ty::ScalarType::Boolean => unimplemented!(),
            spirq::ty::ScalarType::Integer {
                bits: _,
                is_signed: _,
            } => unimplemented!(),
            spirq::ty::ScalarType::Float { bits } => match bits {
                // TODO affine
                32 => match matrix_type.nvector {
                    2 => match matrix_type.vector_ty.nscalar {
                        2 => BaseType::Mat2,
                        _ => unimplemented!(),
                    },
                    3 => match matrix_type.vector_ty.nscalar {
                        3 => BaseType::Mat3,
                        _ => unimplemented!(),
                    },
                    4 => match matrix_type.vector_ty.nscalar {
                        4 => BaseType::Mat4,
                        _ => unimplemented!(),
                    },
                    _ => unimplemented!(),
                },
                _ => unimplemented!(),
            },
        },
        spirq::ty::Type::Image(_image_type) => unimplemented!(),
        spirq::ty::Type::CombinedImageSampler(_combined_image_sampler_type) => unimplemented!(),
        spirq::ty::Type::SampledImage(_sampled_image_type) => {
            unimplemented!()
        }
        spirq::ty::Type::StorageImage(_storage_image_type) => {
            unimplemented!()
        }
        spirq::ty::Type::Sampler(_sampler_type) => unimplemented!(),
        spirq::ty::Type::SubpassData(_subpass_data_type) => {
            unimplemented!()
        }
        spirq::ty::Type::Array(_array_type) => unimplemented!("{:?}", _array_type),
        spirq::ty::Type::Struct(_struct_type) => unimplemented!(),
        spirq::ty::Type::AccelStruct(_accel_struct_type) => {
            unimplemented!()
        }
        spirq::ty::Type::DeviceAddress(_device_address_type) => {
            unimplemented!()
        }
        spirq::ty::Type::DevicePointer(_pointer_type) => unimplemented!(),
        spirq::ty::Type::RayQuery(_ray_query_type) => unimplemented!(),
        _ => unimplemented!(),
    };
    dyn_ty
}
