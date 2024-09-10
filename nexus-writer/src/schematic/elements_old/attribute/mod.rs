use std::{
    rc::Rc,
    sync::{Mutex, MutexGuard},
};

use builder::NexusAttributeBuilder;
use hdf5::{Attribute, Dataset, H5Type};
use underlying::UnderlyingNexusAttribute;

mod builder;
mod underlying;

use super::{
    error::{ClosingError, CreationError, HDF5Error, OpeningError},
    traits::{self, Buildable, CanWriteScalar},
    SmartPointer,
};

pub(crate) trait NxAttribute {
    fn create(&mut self, dataset: &Dataset) -> Result<(), CreationError>;
    fn open(&mut self, dataset: &Dataset) -> Result<(), OpeningError>;
    fn close(&mut self) -> Result<(), ClosingError>;
}

#[derive(strum::Display)]
pub(crate) enum NexusUnits {
    #[strum(to_string = "second")]
    Seconds,
    #[strum(to_string = "us")]
    Microseconds,
    #[strum(to_string = "ns")]
    Nanoseconds,
    #[strum(to_string = "ISO8601")]
    ISO8601,
    #[strum(to_string = "mEv")]
    MegaElectronVolts,
    #[strum(to_string = "uAh")]
    MicroAmpHours,
    #[strum(to_string = "counts")]
    Counts,
}

#[derive(Clone)]
pub(crate) struct NexusAttribute<T, C = traits::tags::Mutable>(
    SmartPointer<UnderlyingNexusAttribute<T, C>>,
)
where
    T: H5Type + Clone,
    C: traits::tags::Tag<T, Dataset, Attribute>;

impl<T, C> NexusAttribute<T, C>
where
    T: H5Type + Clone,
    C: traits::tags::Tag<T, Dataset, Attribute>,
{
    fn new(attribute: UnderlyingNexusAttribute<T, C>) -> Self {
        NexusAttribute(Rc::new(Mutex::new(attribute)))
    }

    pub(crate) fn lock_mutex(&self) -> MutexGuard<'_, UnderlyingNexusAttribute<T, C>> {
        self.0.lock().expect("Lock exists")
    }

    pub(crate) fn clone_inner(&self) -> SmartPointer<UnderlyingNexusAttribute<T, C>> {
        self.0.clone()
    }
}

pub(crate) type NexusAttributeFixed<T> = NexusAttribute<T, traits::tags::Constant>;

/// Class Implementation
impl<T: H5Type + Clone + Default> traits::Class<T, Dataset, Attribute> for traits::Mutable<T> {
    fn create(&self, parent: &Dataset, name: &str) -> Result<Attribute, CreationError> {
        let attribute = parent
            .new_attr::<T>()
            .create(name)
            .map_err(HDF5Error::General)?;
        attribute
            .write_scalar(&self.0)
            .map_err(HDF5Error::General)?;
        Ok(attribute)
    }
}
impl<T: H5Type + Clone + Default> traits::Class<T, Dataset, Attribute> for traits::Constant<T> {
    fn create(&self, parent: &Dataset, name: &str) -> Result<Attribute, CreationError> {
        let attribute = parent
            .new_attr::<T>()
            .create(name)
            .map_err(HDF5Error::General)?;
        attribute
            .write_scalar(&self.0)
            .map_err(HDF5Error::General)?;
        Ok(attribute)
    }
}

/// Class Tag Implementation
impl<T: H5Type + Clone + Default> traits::tags::Tag<T, Dataset, Attribute>
    for traits::tags::Mutable
{
    type ClassType = traits::Mutable<T>;
}
impl<T: H5Type + Clone + Default> traits::tags::Tag<T, Dataset, Attribute>
    for traits::tags::Constant
{
    type ClassType = traits::Constant<T>;
}

/// NexusAttribute

impl<T, C> Buildable<T> for NexusAttribute<T, C>
where
    T: H5Type + Clone,
    C: traits::tags::Tag<T, Dataset, Attribute>,
{
    type BuilderType = NexusAttributeBuilder<T, C, false>;

    fn begin(name: &str) -> NexusAttributeBuilder<T, C, false> {
        NexusAttributeBuilder::new(name)
    }
}

impl<T> CanWriteScalar for NexusAttribute<T, traits::tags::Mutable>
where
    T: H5Type + Clone + Default,
{
    type Type = T;

    fn write_scalar(&self, value: T) -> Result<(), hdf5::Error> {
        self.lock_mutex()
            .attribute
            .as_ref()
            .ok_or_else(|| hdf5::Error::Internal("No Attribute Present".to_owned()))
            .and_then(|attribute| attribute.write_scalar(&value))
    }

    fn read_scalar(&self) -> Result<T, hdf5::Error> {
        self.lock_mutex()
            .attribute
            .as_ref()
            .ok_or_else(|| hdf5::Error::Internal("No Attribute Present".to_owned()))
            .and_then(|attribute| attribute.read_scalar())
    }
}
