use hdf5::{types::TypeDescriptor, Dataset, DatasetBuilder, H5Type};

use crate::error::{HDF5Error, NexusDatasetError};

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

pub(crate) trait DatasetBuilderNumericExt {
    fn create_numeric(self, name: &str, type_desc: &TypeDescriptor) -> Result<Dataset,NexusDatasetError>;
}

impl DatasetBuilderNumericExt for DatasetBuilder {
    fn create_numeric(self, name: &str, type_desc: &TypeDescriptor) -> Result<Dataset,NexusDatasetError> {   
        let dataset = match type_desc {
            TypeDescriptor::Integer(int_size) => match int_size {
                hdf5::types::IntSize::U1 => self
                    .with_data(&[i8::default(); 0])
                    .create(name),
                hdf5::types::IntSize::U2 => self
                    .with_data(&[i16::default(); 0])
                    .create(name),
                hdf5::types::IntSize::U4 => self
                    .with_data(&[i32::default(); 0])
                    .create(name),
                hdf5::types::IntSize::U8 => self
                    .with_data(&[i64::default(); 0])
                    .create(name),
            },
            TypeDescriptor::Unsigned(int_size) => match int_size {
                hdf5::types::IntSize::U1 => self
                    .with_data(&[u8::default(); 0])
                    .create(name),
                hdf5::types::IntSize::U2 => self
                    .with_data(&[u16::default(); 0])
                    .create(name),
                hdf5::types::IntSize::U4 => self
                    .with_data(&[u32::default(); 0])
                    .create(name),
                hdf5::types::IntSize::U8 => self
                    .with_data(&[u64::default(); 0])
                    .create(name),
            },
            TypeDescriptor::Float(float_size) => match float_size {
                hdf5::types::FloatSize::U4 => self
                    .with_data(&[f32::default(); 0])
                    .create(name),
                hdf5::types::FloatSize::U8 => self
                    .with_data(&[f64::default(); 0])
                    .create(name),
            },
            _ => Err(hdf5::Error::Internal(Default::default())),
        }.map_err(HDF5Error::HDF5)?;
        Ok(dataset)
    }
}

pub(crate) trait DatasetNumericExt {
    fn write_numeric_slice(&self, values: &NumericVector, next_values_slice: ndarray::SliceInfo<[ndarray::SliceInfoElem; 1], ndarray::Dim<[usize; 1]>, ndarray::Dim<[usize; 1]>>) -> Result<(), NexusDatasetError>;
}

impl DatasetNumericExt for Dataset {
    fn write_numeric_slice(&self, values: &NumericVector, next_values_slice: ndarray::SliceInfo<[ndarray::SliceInfoElem; 1], ndarray::Dim<[usize; 1]>, ndarray::Dim<[usize; 1]>>) -> Result<(), NexusDatasetError> {
        match values {
            NumericVector::I1(vec) => self.write_slice(vec, next_values_slice),
            NumericVector::I2(vec) => self.write_slice(vec, next_values_slice),
            NumericVector::I4(vec) => self.write_slice(vec, next_values_slice),
            NumericVector::I8(vec) => self.write_slice(vec, next_values_slice),
            NumericVector::U1(vec) => self.write_slice(vec, next_values_slice),
            NumericVector::U2(vec) => self.write_slice(vec, next_values_slice),
            NumericVector::U4(vec) => self.write_slice(vec, next_values_slice),
            NumericVector::U8(vec) => self.write_slice(vec, next_values_slice),
            NumericVector::F4(vec) => self.write_slice(vec, next_values_slice),
            NumericVector::F8(vec) => self.write_slice(vec, next_values_slice),
        }.map_err(HDF5Error::HDF5)?;
        Ok(())
    }
}