use std::slice;
use std::mem;

pub enum UniformPrimitiveType {
    Float,
    Int
}

pub enum UniformDatumType {
    Vec1(UniformPrimitiveType),
    Vec2(UniformPrimitiveType),
    Vec3(UniformPrimitiveType),
    Vec4(UniformPrimitiveType),
    Matrix2x2,
    Matrix3x3,
    Matrix4x4
}

pub trait UniformData {
    fn uniform_datum_type() -> UniformDatumType;
    fn uniform_bytes(&self) -> &[u8];
    fn uniform_elements(&self) -> usize;
}



pub trait UniformDatum {
    fn uniform_datum_type() -> UniformDatumType;
}


pub trait UniformPrimitive {
    fn uniform_primitive_type() -> UniformPrimitiveType;
}

impl UniformPrimitive for f32 {
    fn uniform_primitive_type() -> UniformPrimitiveType {
        UniformPrimitiveType::Float
    }
}

impl UniformPrimitive for i32 {
    fn uniform_primitive_type() -> UniformPrimitiveType {
        UniformPrimitiveType::Int
    }
}



impl<T: UniformPrimitive> UniformDatum for T {
    fn uniform_datum_type() -> UniformDatumType {
        UniformDatumType::Vec1(T::uniform_primitive_type())
    }
}

impl<T: UniformPrimitive> UniformDatum for [T; 1] {
    fn uniform_datum_type() -> UniformDatumType {
        UniformDatumType::Vec1(T::uniform_primitive_type())
    }
}

impl<T: UniformPrimitive> UniformDatum for [T; 2] {
    fn uniform_datum_type() -> UniformDatumType {
        UniformDatumType::Vec1(T::uniform_primitive_type())
    }
}

impl<T: UniformPrimitive> UniformDatum for [T; 3] {
    fn uniform_datum_type() -> UniformDatumType {
        UniformDatumType::Vec1(T::uniform_primitive_type())
    }
}

impl<T> UniformDatum for [T; 4] where T: UniformPrimitive {
    fn uniform_datum_type() -> UniformDatumType {
        UniformDatumType::Vec1(T::uniform_primitive_type())
    }
}

impl UniformDatum for [[f32; 2]; 2] {
    fn uniform_datum_type() -> UniformDatumType {
        UniformDatumType::Matrix2x2
    }
}

impl UniformDatum for [[f32; 3]; 3] {
    fn uniform_datum_type() -> UniformDatumType {
        UniformDatumType::Matrix3x3
    }
}

impl UniformDatum for [[f32; 4]; 4] {
    fn uniform_datum_type() -> UniformDatumType {
        UniformDatumType::Matrix4x4
    }
}
