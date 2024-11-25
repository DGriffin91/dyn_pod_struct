use __macro_exports::RegisterForReflection;
use bevy::reflect::*;
use glam::*;

use crate::{BaseType, DynField, TrackedDynStruct};

impl Reflect for TrackedDynStruct {
    fn get_represented_type_info(&self) -> Option<&'static TypeInfo> {
        None
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn into_reflect(self: Box<Self>) -> Box<dyn Reflect> {
        self
    }

    fn as_reflect(&self) -> &dyn Reflect {
        self
    }

    fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
        self
    }

    fn try_apply(&mut self, _value: &dyn Reflect) -> Result<(), ApplyError> {
        todo!()
    }

    fn set(&mut self, _value: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>> {
        todo!()
    }

    fn reflect_ref(&self) -> ReflectRef {
        ReflectRef::Struct(self)
    }

    fn reflect_mut(&mut self) -> ReflectMut {
        ReflectMut::Struct(self)
    }

    fn reflect_owned(self: Box<Self>) -> ReflectOwned {
        ReflectOwned::Struct(self)
    }

    fn clone_value(&self) -> Box<dyn Reflect> {
        Box::new(Struct::clone_dynamic(self))
    }
}

impl Struct for TrackedDynStruct {
    fn field(&self, name: &str) -> Option<&dyn Reflect> {
        let Some(field) = self.dyn_struct.layout.get_path(&[name]) else {
            return None;
        };
        self.reflect_field(field)
    }
    fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect> {
        let Some(field) = self.dyn_struct.layout.get_path(&[name]) else {
            return None;
        };
        self.reflect_field_mut(&field.clone()) // TODO avoid clone
    }
    fn field_at(&self, index: usize) -> Option<&dyn Reflect> {
        let Some((_, field)) = self.dyn_struct.layout.fields.get(index) else {
            return None;
        };
        self.reflect_field(field)
    }
    fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        let Some((_, field)) = self.dyn_struct.layout.fields.get(index) else {
            return None;
        };
        self.reflect_field_mut(&field.clone()) // TODO avoid clone
    }
    fn name_at(&self, index: usize) -> Option<&str> {
        self.dyn_struct
            .layout
            .fields
            .get(index)
            .map(|f| f.0.as_str())
    }
    fn field_len(&self) -> usize {
        self.dyn_struct.layout.fields.len()
    }
    fn iter_fields(&self) -> FieldIter {
        FieldIter::new(self)
    }
    fn clone_dynamic(&self) -> DynamicStruct {
        let mut dynamic: DynamicStruct = ::core::default::Default::default();
        dynamic.set_represented_type(Reflect::get_represented_type_info(self));
        for (name, field) in &self.dyn_struct.layout.fields {
            dynamic.insert_boxed(
                name,
                Reflect::clone_value(self.reflect_field(&field).unwrap()),
            );
        }
        dynamic
    }
}

impl TrackedDynStruct {
    pub fn reflect_field(&self, field: &DynField) -> Option<&dyn Reflect> {
        let ofs = field.offset as usize;
        match &field.ty {
            BaseType::None => return None,
            BaseType::U8 => return Some(self.get_raw::<u8>(ofs)),
            BaseType::U16 => return Some(self.get_raw::<u16>(ofs)),
            BaseType::U32 => return Some(self.get_raw::<u32>(ofs)),
            BaseType::U64 => return Some(self.get_raw::<u64>(ofs)),
            BaseType::U128 => return Some(self.get_raw::<u128>(ofs)),
            BaseType::I8 => return Some(self.get_raw::<i8>(ofs)),
            BaseType::I16 => return Some(self.get_raw::<i16>(ofs)),
            BaseType::I32 => return Some(self.get_raw::<i32>(ofs)),
            BaseType::I64 => return Some(self.get_raw::<i64>(ofs)),
            BaseType::I128 => return Some(self.get_raw::<i128>(ofs)),
            BaseType::F32 => return Some(self.get_raw::<f32>(ofs)),
            BaseType::F64 => return Some(self.get_raw::<f64>(ofs)),
            BaseType::UVec2 => return Some(self.get_raw::<UVec2>(ofs)),
            BaseType::UVec3 => return Some(self.get_raw::<UVec3>(ofs)),
            BaseType::UVec4 => return Some(self.get_raw::<UVec4>(ofs)),
            BaseType::IVec2 => return Some(self.get_raw::<IVec2>(ofs)),
            BaseType::IVec3 => return Some(self.get_raw::<IVec3>(ofs)),
            BaseType::IVec4 => return Some(self.get_raw::<IVec4>(ofs)),
            BaseType::Vec2 => return Some(self.get_raw::<Vec2>(ofs)),
            BaseType::Vec3 => return Some(self.get_raw::<Vec3>(ofs)),
            BaseType::Vec4 => return Some(self.get_raw::<Vec4>(ofs)),
            BaseType::Mat2 => return Some(self.get_raw::<Mat2>(ofs)),
            BaseType::Mat3 => return Some(self.get_raw::<Mat3>(ofs)),
            BaseType::Mat4 => return Some(self.get_raw::<Mat4>(ofs)),
            BaseType::Quat => return Some(self.get_raw::<Quat>(ofs)),
            BaseType::DVec2 => return Some(self.get_raw::<DVec2>(ofs)),
            BaseType::DVec3 => return Some(self.get_raw::<DVec3>(ofs)),
            BaseType::DVec4 => return Some(self.get_raw::<DVec4>(ofs)),
            BaseType::DMat2 => return Some(self.get_raw::<DMat2>(ofs)),
            BaseType::DMat3 => return Some(self.get_raw::<DMat3>(ofs)),
            BaseType::DMat4 => return Some(self.get_raw::<DMat4>(ofs)),
            BaseType::DAffine2 => return Some(self.get_raw::<DAffine2>(ofs)),
            BaseType::DAffine3 => return Some(self.get_raw::<DAffine3>(ofs)),
            // TODO Need a DynFieldRef that can hold this field and a slice of bytes
            // How do we return a reference to the new DynFieldRef though?
            BaseType::Struct(_arc) => todo!(),
        };
    }

    fn reflect_field_mut(&mut self, field: &DynField) -> Option<&mut dyn Reflect> {
        let ofs = field.offset as usize;
        match &field.ty {
            BaseType::None => return None,
            BaseType::U8 => return Some(self.get_mut_raw::<u8>(ofs)),
            BaseType::U16 => return Some(self.get_mut_raw::<u16>(ofs)),
            BaseType::U32 => return Some(self.get_mut_raw::<u32>(ofs)),
            BaseType::U64 => return Some(self.get_mut_raw::<u64>(ofs)),
            BaseType::U128 => return Some(self.get_mut_raw::<u128>(ofs)),
            BaseType::I8 => return Some(self.get_mut_raw::<i8>(ofs)),
            BaseType::I16 => return Some(self.get_mut_raw::<i16>(ofs)),
            BaseType::I32 => return Some(self.get_mut_raw::<i32>(ofs)),
            BaseType::I64 => return Some(self.get_mut_raw::<i64>(ofs)),
            BaseType::I128 => return Some(self.get_mut_raw::<i128>(ofs)),
            BaseType::F32 => return Some(self.get_mut_raw::<f32>(ofs)),
            BaseType::F64 => return Some(self.get_mut_raw::<f64>(ofs)),
            BaseType::UVec2 => return Some(self.get_mut_raw::<UVec2>(ofs)),
            BaseType::UVec3 => return Some(self.get_mut_raw::<UVec3>(ofs)),
            BaseType::UVec4 => return Some(self.get_mut_raw::<UVec4>(ofs)),
            BaseType::IVec2 => return Some(self.get_mut_raw::<IVec2>(ofs)),
            BaseType::IVec3 => return Some(self.get_mut_raw::<IVec3>(ofs)),
            BaseType::IVec4 => return Some(self.get_mut_raw::<IVec4>(ofs)),
            BaseType::Vec2 => return Some(self.get_mut_raw::<Vec2>(ofs)),
            BaseType::Vec3 => return Some(self.get_mut_raw::<Vec3>(ofs)),
            BaseType::Vec4 => return Some(self.get_mut_raw::<Vec4>(ofs)),
            BaseType::Mat2 => return Some(self.get_mut_raw::<Mat2>(ofs)),
            BaseType::Mat3 => return Some(self.get_mut_raw::<Mat3>(ofs)),
            BaseType::Mat4 => return Some(self.get_mut_raw::<Mat4>(ofs)),
            BaseType::Quat => return Some(self.get_mut_raw::<Quat>(ofs)),
            BaseType::DVec2 => return Some(self.get_mut_raw::<DVec2>(ofs)),
            BaseType::DVec3 => return Some(self.get_mut_raw::<DVec3>(ofs)),
            BaseType::DVec4 => return Some(self.get_mut_raw::<DVec4>(ofs)),
            BaseType::DMat2 => return Some(self.get_mut_raw::<DMat2>(ofs)),
            BaseType::DMat3 => return Some(self.get_mut_raw::<DMat3>(ofs)),
            BaseType::DMat4 => return Some(self.get_mut_raw::<DMat4>(ofs)),
            BaseType::DAffine2 => return Some(self.get_mut_raw::<DAffine2>(ofs)),
            BaseType::DAffine3 => return Some(self.get_mut_raw::<DAffine3>(ofs)),
            // TODO Need a DynFieldRefMut that can hold this field and a slice of bytes
            // How do we return a reference to the new DynFieldRefMut though?
            BaseType::Struct(_arc) => todo!(),
        };
    }
}

impl FromReflect for TrackedDynStruct {
    fn from_reflect(_reflect: &dyn Reflect) -> Option<Self> {
        None
    }
}

impl GetTypeRegistration for TrackedDynStruct
where
    Self: ::core::any::Any + ::core::marker::Send + ::core::marker::Sync,
{
    fn get_type_registration() -> TypeRegistration {
        let mut registration = TypeRegistration::of::<Self>();
        registration.insert::<ReflectFromPtr>(FromType::<Self>::from_type());
        registration.insert::<ReflectFromReflect>(FromType::<Self>::from_type());
        registration
    }

    #[inline(never)]
    fn register_type_dependencies(registry: &mut TypeRegistry) {
        <u8 as RegisterForReflection>::__register(registry);
        <u16 as RegisterForReflection>::__register(registry);
        <u32 as RegisterForReflection>::__register(registry);
        <u64 as RegisterForReflection>::__register(registry);
        <u128 as RegisterForReflection>::__register(registry);
        <i8 as RegisterForReflection>::__register(registry);
        <i16 as RegisterForReflection>::__register(registry);
        <i32 as RegisterForReflection>::__register(registry);
        <i64 as RegisterForReflection>::__register(registry);
        <i128 as RegisterForReflection>::__register(registry);
        <f32 as RegisterForReflection>::__register(registry);
        <f64 as RegisterForReflection>::__register(registry);
        <UVec2 as RegisterForReflection>::__register(registry);
        <UVec3 as RegisterForReflection>::__register(registry);
        <UVec4 as RegisterForReflection>::__register(registry);
        <IVec2 as RegisterForReflection>::__register(registry);
        <IVec3 as RegisterForReflection>::__register(registry);
        <IVec4 as RegisterForReflection>::__register(registry);
        <Vec2 as RegisterForReflection>::__register(registry);
        <Vec3 as RegisterForReflection>::__register(registry);
        <Vec4 as RegisterForReflection>::__register(registry);
        <Mat2 as RegisterForReflection>::__register(registry);
        <Mat3 as RegisterForReflection>::__register(registry);
        <Mat4 as RegisterForReflection>::__register(registry);
        <Quat as RegisterForReflection>::__register(registry);
        <DVec2 as RegisterForReflection>::__register(registry);
        <DVec3 as RegisterForReflection>::__register(registry);
        <DVec4 as RegisterForReflection>::__register(registry);
        <DMat2 as RegisterForReflection>::__register(registry);
        <DMat3 as RegisterForReflection>::__register(registry);
        <DMat4 as RegisterForReflection>::__register(registry);
        <DAffine2 as RegisterForReflection>::__register(registry);
        <DAffine3 as RegisterForReflection>::__register(registry);
    }
}

impl Typed for TrackedDynStruct
where
    Self: ::core::any::Any + ::core::marker::Send + ::core::marker::Sync,
{
    fn type_info() -> &'static TypeInfo {
        static CELL: utility::NonGenericTypeInfoCell = utility::NonGenericTypeInfoCell::new();
        CELL.get_or_set(|| {
            TypeInfo::Struct(
                StructInfo::new::<Self>(&[]) // Can't fill fields statically 
                    .with_custom_attributes(attributes::CustomAttributes::default()),
            )
        })
    }
}
