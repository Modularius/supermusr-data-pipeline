use builder::NexusDatasetBuilder;
use hdf5::{Dataset, Group, H5Type, SimpleExtents};
use ndarray::s;
use std::{rc::Rc, sync::Mutex};
use super::{
    attribute::{NexusUnits, NxAttribute}, traits::{Buildable, CanAppend, CanWriteScalar, Class}, NxLivesInGroup
};
use tracing::instrument;
use crate::schematic::elements::traits;

mod builder;

// Implement Database Classes 

impl<T: H5Type> traits::Class<T, Dataset> for () {
    fn create(&self, parent: &Group, name: &str) -> Result<Dataset, anyhow::Error> {
        let dataset = parent.new_dataset::<T>().create(name)?;
        Ok(dataset)
    }
}
impl<T: H5Type + Clone> traits::Class<T,Dataset> for traits::Constant<T> {
    fn create(&self, parent: &Group, name: &str) -> Result<Dataset, anyhow::Error> {
        let dataset = parent.new_dataset::<T>().create(name)?;
        dataset.write_scalar(&self.0).expect("");
        Ok(dataset)
    }
}

impl<T: H5Type> traits::Class<T, Dataset> for traits::Resizable {
    fn create(&self, parent: &Group, name: &str) -> Result<Dataset, anyhow::Error> {
        let dataset = parent
            .new_dataset::<T>()
            .shape(SimpleExtents::resizable(vec![self.initial_size]))
            .chunk(vec![self.chunk_size])
            .create(name)?;
        Ok(dataset)
    }
}


impl<T: H5Type> traits::tags::Tag<T, Dataset> for () {
    type ClassType = ();
}
impl<T: H5Type + Clone> traits::tags::Tag<T, Dataset> for traits::tags::Constant {
    type ClassType = traits::Constant<T>;
}
impl<T: H5Type> traits::tags::Tag<T, Dataset> for traits::tags::Resizable {
    type ClassType = traits::Resizable;
}

// Aliases to hide the class structrure
pub(crate) type NexusDatasetFixed<T, D = ()> =
    Rc<Mutex<UnderlyingNexusDataset<T, D, traits::tags::Constant>>>;
pub(crate) type NexusDatasetResize<T, D = ()> =
    Rc<Mutex<UnderlyingNexusDataset<T, D, traits::tags::Resizable>>>;

pub(crate) type AttributeRegister = Rc<Mutex<Vec<Rc<Mutex<dyn NxAttribute>>>>>;

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

#[derive(Default)]
pub(crate) struct UnderlyingNexusDataset<T: H5Type, D: NxDataset = (), C: traits::tags::Tag<T,Dataset> = ()>
{
    name: String,
    attributes_register: AttributeRegister,
    attributes: D,
    class: C::ClassType,
    dataset: Option<Dataset>,
}

pub(crate) type NexusDataset<T, D = (), C = ()> = Rc<Mutex<UnderlyingNexusDataset<T, D, C>>>;

impl<T, D, C> Buildable<T, D, C> for NexusDataset<T, D, C>
where
    T: H5Type + Clone,
    D: NxDataset,
    C: traits::tags::Tag<T,Dataset>,
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

impl<T, D, C> NxLivesInGroup for UnderlyingNexusDataset<T, D, C>
where
    T: H5Type + Clone,
    D: NxDataset,
    C: traits::tags::Tag<T,Dataset>,
{
    #[instrument(skip_all, level = "debug", fields(name = tracing::field::Empty), err(level = "error"))]
    fn create(&mut self, parent: &Group) -> Result<(), anyhow::Error> {
        if self.dataset.is_some() {
            Err(anyhow::anyhow!("{} dataset already open", self.name))
        } else {
            let dataset = self.class.create(parent, &self.name)?;
            for attribute in self
                .attributes_register
                .lock()
                .expect("Lock Exists")
                .iter_mut()
            {
                attribute.lock().expect("Lock Exists").create(&dataset)?;
            }
            self.dataset = Some(dataset);
            Ok(())
        }
    }

    #[instrument(skip_all, level = "debug", fields(name = tracing::field::Empty), err(level = "error"))]
    fn open(&mut self, parent: &Group) -> Result<(), anyhow::Error> {
        if self.dataset.is_some() {
            Err(anyhow::anyhow!("{} dataset already open", self.name))
        } else {
            match parent.dataset(&self.name) {
                Ok(dataset) => {
                    for attribute in self
                        .attributes_register
                        .lock()
                        .expect("Lock Exists")
                        .iter_mut()
                    {
                        attribute.lock().expect("Lock Exists").open(&dataset)?;
                    }
                    self.dataset = Some(dataset);
                    Ok(())
                }
                Err(e) => Err(e.into()),
            }
        }
    }

    #[instrument(skip_all, level = "debug", fields(name = tracing::field::Empty), err(level = "error"))]
    fn close(&mut self) -> Result<(), anyhow::Error> {
        if self.dataset.is_none() {
            Err(anyhow::anyhow!("{} dataset already closed", self.name))
        } else {
            for attribute in self
                .attributes_register
                .lock()
                .expect("Lock Exists")
                .iter_mut()
            {
                attribute.lock().expect("Lock Exists").close()?;
            }
            self.dataset = None;
            Ok(())
        }
    }
}

