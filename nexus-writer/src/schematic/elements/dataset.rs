use std::{marker::PhantomData, rc::Rc, sync::Mutex};

use hdf5::{types::VarLenAscii, Dataset, Group, H5Type, SimpleExtents};
use ndarray::s;
use tracing::{info, instrument};

use super::{
    attribute::{NexusAttribute, NexusUnits, NxAttribute},
    group::RcGroupContentRegister,
    MustEnterFixedValue, NxLivesInGroup,
};

// Dataset Resizable Option
/*
trait DatasetResizableOption<T : H5Type> {
    fn create_dataset(parent: &Group) -> Dataset;
}

struct DatasetNoResize;
impl<T : H5Type> DatasetResizableOption<T> for DatasetNoResize {
    fn create_dataset(parent: &Group) -> Dataset {
        parent
            .new_dataset_builder()
            .with_data(&[fixed_value.clone()])
            .create(self.name.as_str())
    }
}
 */

pub(crate) type RcNexusDatasetVar<T, D = ()> = Rc<Mutex<NexusDataset<T, D, false>>>;
pub(crate) type RcNexusDatasetFixed<T, D = ()> = Rc<Mutex<NexusDataset<T, D, true>>>;
pub(crate) type RcNexusDatasetResize<T, D = ()> = Rc<Mutex<NexusDataset<T, D, false, true>>>;

pub(crate) type RcAttributeRegister = Rc<Mutex<Vec<Rc<Mutex<dyn NxAttribute>>>>>;

pub(crate) trait NxContainerAttributes: Sized {
    const UNITS: Option<NexusUnits> = None;

    fn new(attribute_register: RcAttributeRegister) -> Self;
}

impl NxContainerAttributes for () {
    fn new(_attribute_register: RcAttributeRegister) -> Self {
        Default::default()
    }
}

/// NxDataset Trait
#[derive(Default)]
pub(crate) struct NexusDataset<
    T: H5Type,
    D: NxContainerAttributes = (),
    const F: bool = false,
    const R: bool = false,
> {
    name: String,
    fixed_value: Option<T>,
    attributes: RcAttributeRegister,
    initial_size: usize,
    chunk_size: usize,
    class: D,
    dataset: Option<Dataset>,
}

impl<T, D, const F: bool, const R: bool> NexusDataset<T, D, F, R>
where
    T: H5Type + Clone,
    D: NxContainerAttributes,
{
    pub(crate) fn begin() -> NexusDatasetBuilder<T, D, F, R, F, R> {
        NexusDatasetBuilder::<T, D, F, R, F, R> {
            fixed_value: None,
            initial_size: Default::default(),
            chunk_size: Default::default(),
            phantom: PhantomData,
        }
    }
}

pub(crate) trait CanWriteScalar {
    type Type: H5Type;
    fn write_scalar(&self, value: Self::Type) -> Result<(), hdf5::Error>;
}

impl<T, D> CanWriteScalar for Rc<Mutex<NexusDataset<T, D, false, false>>>
where
    T: H5Type + Clone,
    D: NxContainerAttributes,
{
    type Type = T;

    fn write_scalar(&self, value: T) -> Result<(), hdf5::Error> {
        if let Some(dataset) = &self.lock().expect("Can Lock").dataset {
            info!("{}", &self.lock().expect("Can Lock").name);
            dataset.write_scalar(&value)
        } else {
            panic!("Dataset Not Created")
        }
    }
}

pub(crate) trait CanAppend {
    type Type: H5Type;
    fn append(&self, value: &[Self::Type]) -> Result<(), hdf5::Error>;
}
impl<T, D> CanAppend for Rc<Mutex<NexusDataset<T, D, false, true>>>
where
    T: H5Type + Clone,
    D: NxContainerAttributes,
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

impl<T, D, const F: bool, const R: bool> NxLivesInGroup for NexusDataset<T, D, F, R>
where
    T: H5Type + Clone,
    D: NxContainerAttributes,
{
    #[instrument(skip_all, level = "debug", fields(name = tracing::field::Empty), err(level = "error"))]
    fn create(&mut self, parent: &Group) -> Result<(), anyhow::Error> {
        if self.dataset.is_some() {
            Err(anyhow::anyhow!("{} dataset already open", self.name))
        } else {
            let dataset = if R {
                parent
                    .new_dataset::<T>()
                    .shape(SimpleExtents::resizable(vec![self.initial_size]))
                    .chunk(vec![self.chunk_size])
                    .create(self.name.as_str())
            } else {
                parent.new_dataset::<T>().create(self.name.as_str())
            };
            match dataset {
                Ok(dataset) => {
                    if let Some(fixed_value) = &self.fixed_value {
                        dataset.write_scalar(fixed_value).expect("");
                    }
                    for attribute in self.attributes.lock().expect("Lock Exists").iter_mut() {
                        attribute.lock().expect("Lock Exists").create(&dataset)?;
                    }
                    self.dataset = Some(dataset);
                    Ok(())
                }
                Err(e) => Err(e.into()),
            }
        }
    }

    #[instrument(skip_all, level = "debug", fields(name = tracing::field::Empty), err(level = "error"))]
    fn open(&mut self, parent: &Group) -> Result<(), anyhow::Error> {
        if self.dataset.is_some() {
            Err(anyhow::anyhow!("{} dataset already open", self.name))
        } else {
            match parent.dataset(&self.name) {
                Ok(dataset) => {
                    for attribute in self.attributes.lock().expect("Lock Exists").iter_mut() {
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
            Err(anyhow::anyhow!("{} dataset already open", self.name))
        } else {
            for attribute in self.attributes.lock().expect("Lock Exists").iter_mut() {
                attribute.lock().expect("Lock Exists").close()?;
            }
            self.dataset = None;
            Ok(())
        }
    }
}

/// NexusDatasetBuilder
#[derive(Clone)]
pub(crate) struct NexusDatasetBuilder<
    T,
    D,
    const F0: bool,
    const R0: bool,
    const F: bool,
    const R: bool,
> where
    T: H5Type,
    D: NxContainerAttributes,
{
    fixed_value: Option<T>,
    initial_size: usize,
    chunk_size: usize,
    phantom: PhantomData<D>,
}

impl<T, D, const F0: bool, const R0: bool> NexusDatasetBuilder<T, D, F0, R0, true, false>
where
    T: H5Type,
    D: NxContainerAttributes,
{
    pub(crate) fn fixed_value(self, value: T) -> NexusDatasetBuilder<T, D, F0, R0, false, false> {
        NexusDatasetBuilder {
            fixed_value: Some(value),
            initial_size: self.initial_size,
            chunk_size: self.chunk_size,
            phantom: PhantomData,
        }
    }
}

impl<T, D, const F0: bool, const R0: bool> NexusDatasetBuilder<T, D, F0, R0, false, true>
where
    T: H5Type,
    D: NxContainerAttributes,
{
    pub(crate) fn resizable(
        self,
        initial_size: usize,
        chunk_size: usize,
    ) -> NexusDatasetBuilder<T, D, F0, R0, false, false> {
        NexusDatasetBuilder {
            fixed_value: self.fixed_value,
            initial_size,
            chunk_size,
            phantom: PhantomData,
        }
    }
}

impl<T, D, const F0: bool, const R0: bool> NexusDatasetBuilder<T, D, F0, R0, false, false>
where
    T: H5Type + Clone,
    D: NxContainerAttributes + Clone + 'static,
{
    pub(crate) fn finish(
        self,
        name: &str,
        parent_content_register: RcGroupContentRegister,
    ) -> Rc<Mutex<NexusDataset<T, D, F0, R0>>> {
        let attributes = RcAttributeRegister::new(Mutex::new(Vec::new()));

        if let Some(units) = D::UNITS {
            NexusAttribute::begin()
                .fixed_value(VarLenAscii::from_ascii(&units.to_string()).expect(""))
                .finish("units", attributes.clone());
        }

        let rc = Rc::new(Mutex::new(NexusDataset::<_, _, F0, R0> {
            name: name.to_owned(),
            fixed_value: self.fixed_value,
            initial_size: self.initial_size,
            chunk_size: self.chunk_size,
            class: D::new(attributes.clone()),
            attributes,
            dataset: None,
        }));
        parent_content_register
            .lock()
            .expect("Lock Exists")
            .push(rc.clone());
        rc
    }
}
