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
    traits::{self, Buildable},
    SmartPointer,
};

pub(crate) trait NxAttribute {
    fn create(&mut self, dataset: &Dataset) -> anyhow::Result<()>;
    fn open(&mut self, dataset: &Dataset) -> anyhow::Result<()>;
    fn close(&mut self) -> anyhow::Result<()>;
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
pub(crate) struct NexusAttribute<T, C = ()>(SmartPointer<UnderlyingNexusAttribute<T, C>>)
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
impl<T: H5Type> traits::Class<T, Dataset, Attribute> for () {
    fn create(&self, parent: &Dataset, name: &str) -> Result<Attribute, anyhow::Error> {
        let attribute = parent.new_attr::<T>().create(name)?;
        Ok(attribute)
    }
}
impl<T: H5Type + Clone> traits::Class<T, Dataset, Attribute> for traits::Constant<T> {
    fn create(&self, parent: &Dataset, name: &str) -> Result<Attribute, anyhow::Error> {
        let attribute = parent.new_attr::<T>().create(name)?;
        attribute
            .write_scalar(&self.0)
            .expect("Attribute can be writen to");
        Ok(attribute)
    }
}

/// Class Tag Implementation
impl<T: H5Type> traits::tags::Tag<T, Dataset, Attribute> for () {
    type ClassType = ();
}
impl<T: H5Type + Clone> traits::tags::Tag<T, Dataset, Attribute> for traits::tags::Constant {
    type ClassType = traits::Constant<T>;
}

/// NexusAttribute

impl<T, C> Buildable<T> for NexusAttribute<T, C>
where
    T: H5Type + Clone,
    C: traits::tags::Tag<T, Dataset, Attribute>,
{
    type BuilderType = NexusAttributeBuilder<T, (), C>;

    fn begin(name: &str) -> NexusAttributeBuilder<T, (), C> {
        NexusAttributeBuilder::new(name)
    }
}
