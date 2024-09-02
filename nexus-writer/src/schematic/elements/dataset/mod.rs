use super::{
    attribute::{NexusUnits, NxAttribute},
    traits::{Buildable, CanAppend, CanWriteScalar},
};
use crate::schematic::elements::traits;
use builder::NexusDatasetBuilder;
use hdf5::{Dataset, Group, H5Type, SimpleExtents};
use ndarray::s;
use std::{rc::Rc, sync::Mutex};
use tracing::instrument;
use underlying::UnderlyingNexusDataset;
#[cfg(test)]
use super::traits::Examine;

mod builder;
mod underlying;

/// NxDataset Trait
pub(crate) trait NxDataset: Sized {
    const UNITS: Option<NexusUnits> = None;

    fn new(attribute_register: AttributeRegister) -> Self;
}

impl NxDataset for () {
    fn new(_attribute_register: AttributeRegister) -> Self {
        Default::default()
    }
}

/// Class Implementation
impl<T: H5Type> traits::Class<T, Group, Dataset> for () {
    fn create(&self, parent: &Group, name: &str) -> Result<Dataset, anyhow::Error> {
        let dataset = parent.new_dataset::<T>().create(name)?;
        Ok(dataset)
    }
}

impl<T: H5Type + Clone> traits::Class<T, Group, Dataset> for traits::Constant<T> {
    fn create(&self, parent: &Group, name: &str) -> Result<Dataset, anyhow::Error> {
        let dataset = parent.new_dataset::<T>().create(name)?;
        dataset.write_scalar(&self.0).expect("");
        Ok(dataset)
    }
}

impl<T: H5Type> traits::Class<T, Group, Dataset> for traits::Resizable {
    fn create(&self, parent: &Group, name: &str) -> Result<Dataset, anyhow::Error> {
        let dataset = parent
            .new_dataset::<T>()
            .shape(SimpleExtents::resizable(vec![self.initial_size]))
            .chunk(vec![self.chunk_size])
            .create(name)?;
        Ok(dataset)
    }
}

/// Class Tag Implementation
impl<T: H5Type> traits::tags::Tag<T, Group, Dataset> for ()
where
    T: H5Type + Clone,
{
    type ClassType = ();
}
impl<T> traits::tags::Tag<T, Group, Dataset> for traits::tags::Constant
where
    T: H5Type + Clone,
{
    type ClassType = traits::Constant<T>;
}
impl<T> traits::tags::Tag<T, Group, Dataset> for traits::tags::Resizable
where
    T: H5Type + Clone,
{
    type ClassType = traits::Resizable;
}

/// Defining Types
pub(crate) type NexusDataset<T, D = (), C = ()> = Rc<Mutex<UnderlyingNexusDataset<T, D, C>>>;
pub(crate) type AttributeRegister = Rc<Mutex<Vec<Rc<Mutex<dyn NxAttribute>>>>>;

// Aliases to hide the class structrure
pub(crate) type NexusDatasetFixed<T, D = ()> = NexusDataset<T, D, traits::tags::Constant>;
pub(crate) type NexusDatasetResize<T, D = ()> = NexusDataset<T, D, traits::tags::Resizable>;

// Dataset Implementations
impl<T, D, C> Buildable<T> for NexusDataset<T, D, C>
where
    T: H5Type + Clone,
    D: NxDataset,
    C: traits::tags::Tag<T, Group, Dataset>,
{
    type BuilderType = NexusDatasetBuilder<T, D, (), C>;
    fn begin(name: &str) -> NexusDatasetBuilder<T, D, (), C> {
        NexusDatasetBuilder::new(name)
    }
}

impl<T, D> CanWriteScalar for NexusDataset<T, D, ()>
where
    T: H5Type + Clone,
    D: NxDataset,
{
    type Type = T;

    fn write_scalar(&self, value: T) -> Result<(), hdf5::Error> {
        self.lock()
            .expect("Can Lock")
            .dataset
            .as_ref()
            .map(|dataset| dataset.write_scalar(&value).unwrap())
            .ok_or_else(|| hdf5::Error::Internal("No Dataset Present".to_owned()))
    }
}

impl<T, D> CanAppend for NexusDataset<T, D, traits::tags::Resizable>
where
    T: H5Type + Clone,
    D: NxDataset,
{
    type Type = T;

    #[instrument(skip_all, level = "debug", fields(name = tracing::field::Empty), err(level = "error"))]
    fn append(&self, values: &[T]) -> Result<(), hdf5::Error> {
        let lock_self = self.lock().expect("Can Lock");
        lock_self
            .dataset
            .as_ref()
            .ok_or_else(|| hdf5::Error::Internal("No Dataset Present".to_owned()))
            .and_then(|dataset| {
                let size = dataset.size();
                let next_values_slice = s![size..(size + values.len())];
                dataset.resize(size + 1)?;
                dataset.write_slice(values, next_values_slice)?;
                Ok(())
            })
    }
}


#[cfg(test)]
impl<T, D> Examine<Rc<Mutex<dyn NxAttribute>>, D> for NexusDataset<T,D>
where 
    T: H5Type + Clone,
    D: NxDataset,
{
    fn examine<F, X>(&self, f: F) -> X
    where
        F: Fn(&D) -> X {
            f(&self.lock().unwrap().attributes)
        }

    fn examine_children<F, X>(&self, f: F) -> X
    where
        F: Fn(&[Rc<Mutex<dyn NxAttribute>>]) -> X {
            f(&self.lock().unwrap().attributes_register.lock().unwrap())
        }
}