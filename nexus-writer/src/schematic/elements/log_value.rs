use hdf5::{
    types::{EnumMember, EnumType, OwnedDynValue, TypeDescriptor},
    Dataset, Group, H5Type, SimpleExtents,
};

pub(crate) enum NumericVector {
    I1(Vec<i8>),
    I2(Vec<i16>),
    I4(Vec<i32>),
    I8(Vec<i64>),
    U1(Vec<u8>),
    U2(Vec<u16>),
    U4(Vec<u32>),
    U8(Vec<u64>),
    F4(Vec<f32>),
    F8(Vec<f64>),
}

impl NumericVector {
    pub(crate) fn len(&self) -> usize {
        match self {
            NumericVector::I1(vec) => vec.len(),
            NumericVector::I2(vec) => vec.len(),
            NumericVector::I4(vec) => vec.len(),
            NumericVector::I8(vec) => vec.len(),
            NumericVector::U1(vec) => vec.len(),
            NumericVector::U2(vec) => vec.len(),
            NumericVector::U4(vec) => vec.len(),
            NumericVector::U8(vec) => vec.len(),
            NumericVector::F4(vec) => vec.len(),
            NumericVector::F8(vec) => vec.len(),
        }
    }

    pub(crate) fn type_descriptor(&self) -> TypeDescriptor {
        match self {
            NumericVector::I1(_) => i8::type_descriptor(),
            NumericVector::I2(_) => i16::type_descriptor(),
            NumericVector::I4(_) => i32::type_descriptor(),
            NumericVector::I8(_) => i64::type_descriptor(),
            NumericVector::U1(_) => u8::type_descriptor(),
            NumericVector::U2(_) => u16::type_descriptor(),
            NumericVector::U4(_) => u32::type_descriptor(),
            NumericVector::U8(_) => u64::type_descriptor(),
            NumericVector::F4(_) => f32::type_descriptor(),
            NumericVector::F8(_) => f64::type_descriptor(),
        }
    }
}
