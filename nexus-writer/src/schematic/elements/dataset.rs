use std::{marker::PhantomData, rc::Rc, sync::Mutex};

use class::Class;
use hdf5::{types::VarLenAscii, Dataset, Group, H5Type};
use ndarray::s;
use tracing::instrument;

use super::{
    attribute::{NexusAttribute, NexusUnits, NxAttribute},
    group::RcGroupContentRegister,
    NxLivesInGroup,
};

// Dataset Resizable Option
mod class {
    use hdf5::{Dataset, Group, H5Type, SimpleExtents};

    pub(crate) trait Class<T>: Clone {
        fn create_dataset(&self, parent: &Group, name: &str) -> Result<Dataset, anyhow::Error>;
    }

    #[derive(Clone)]
    pub(crate) struct Constant<T: H5Type>(pub(crate) T);

    #[derive(Clone)]
    pub(crate) struct Resizable {
        pub(crate) initial_size: usize,
        pub(crate) chunk_size: usize,
    }

    impl<T: H5Type> Class<T> for () {
        fn create_dataset(&self, parent: &Group, name: &str) -> Result<Dataset, anyhow::Error> {
            let dataset = parent.new_dataset::<T>().create(name)?;
            Ok(dataset)
        }
    }
    impl<T: H5Type + Clone> Class<T> for Constant<T> {
        fn create_dataset(&self, parent: &Group, name: &str) -> Result<Dataset, anyhow::Error> {
            let dataset = parent.new_dataset::<T>().create(name)?;
            dataset.write_scalar(&self.0).expect("");
            Ok(dataset)
        }
    }

    impl<T: H5Type> Class<T> for Resizable {
        fn create_dataset(&self, parent: &Group, name: &str) -> Result<Dataset, anyhow::Error> {
            let dataset = parent
                .new_dataset::<T>()
                .shape(SimpleExtents::resizable(vec![self.initial_size]))
                .chunk(vec![self.chunk_size])
                .create(name)?;
            Ok(dataset)
        }
    }

    pub(crate) mod tags {
        use hdf5::H5Type;

        pub(crate) trait Tag<T: H5Type>: Clone {
            type ClassType: super::Class<T>;
        }

        #[derive(Clone)]
        pub(crate) struct Constant;

        #[derive(Clone)]
        pub(crate) struct Resizable;

        impl<T: H5Type> Tag<T> for () {
            type ClassType = ();
        }
        impl<T: H5Type + Clone> Tag<T> for Constant {
            type ClassType = super::Constant<T>;
        }
        impl<T: H5Type> Tag<T> for Resizable {
            type ClassType = super::Resizable;
        }
    }
}

//pub(crate) type RcNexusDatasetVar<T, D = ()> = Rc<Mutex<UnderlyingNexusDataset<T, (), D>>>;
pub(crate) type NexusDatasetFixed<T, D = ()> =
    Rc<Mutex<UnderlyingNexusDataset<T, D, class::tags::Constant>>>;
pub(crate) type NexusDatasetResize<T, D = ()> =
    Rc<Mutex<UnderlyingNexusDataset<T, D, class::tags::Resizable>>>;

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
pub(crate) struct UnderlyingNexusDataset<
    T: H5Type,
    D: NxContainerAttributes = (),
    C: class::tags::Tag<T> = (),
> {
    name: String,
    attributes_register: RcAttributeRegister,
    attributes: D,
    class: C::ClassType,
    dataset: Option<Dataset>,
}

pub(crate) type NexusDataset<T, D = (), C = ()> = Rc<Mutex<UnderlyingNexusDataset<T, D, C>>>;

pub(crate) trait Buildable<T, D, C>
where
    T: H5Type + Clone,
    D: NxContainerAttributes,
    C: class::tags::Tag<T>,
{
    fn begin() -> NexusDatasetBuilder<T, D, (), C>;
}

impl<T, D, C> Buildable<T, D, C> for NexusDataset<T, D, C>
where
    T: H5Type + Clone,
    D: NxContainerAttributes,
    C: class::tags::Tag<T>,
{
    fn begin() -> NexusDatasetBuilder<T, D, (), C> {
        NexusDatasetBuilder {
            class: (),
            phantom: PhantomData,
        }
    }
}

pub(crate) trait CanWriteScalar {
    type Type: H5Type;
    fn write_scalar(&self, value: Self::Type) -> Result<(), hdf5::Error>;
}

impl<T, D> CanWriteScalar for NexusDataset<T, D, ()>
where
    T: H5Type + Clone,
    D: NxContainerAttributes,
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

pub(crate) trait CanAppend {
    type Type: H5Type;
    fn append(&self, value: &[Self::Type]) -> Result<(), hdf5::Error>;
}
impl<T, D> CanAppend for NexusDataset<T, D, class::tags::Resizable>
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

impl<T, D, C> NxLivesInGroup for UnderlyingNexusDataset<T, D, C>
where
    T: H5Type + Clone,
    D: NxContainerAttributes,
    C: class::tags::Tag<T>,
{
    #[instrument(skip_all, level = "debug", fields(name = tracing::field::Empty), err(level = "error"))]
    fn create(&mut self, parent: &Group) -> Result<(), anyhow::Error> {
        if self.dataset.is_some() {
            Err(anyhow::anyhow!("{} dataset already open", self.name))
        } else {
            let dataset = self.class.create_dataset(parent, &self.name)?;
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

/// NexusDatasetBuilder
#[derive(Clone)]
pub(crate) struct NexusDatasetBuilder<T, D, C0, C>
where
    T: H5Type + Clone,
    D: NxContainerAttributes,
    C0: class::tags::Tag<T>,
    C: class::tags::Tag<T>,
{
    class: C0::ClassType,
    phantom: PhantomData<(T, D, C)>,
}

impl<T, D> NexusDatasetBuilder<T, D, (), class::tags::Constant>
where
    T: H5Type + Clone,
    D: NxContainerAttributes,
{
    pub(crate) fn fixed_value(
        self,
        value: T,
    ) -> NexusDatasetBuilder<T, D, class::tags::Constant, ()> {
        NexusDatasetBuilder {
            class: class::Constant(value),
            phantom: PhantomData,
        }
    }
}

impl<T, D> NexusDatasetBuilder<T, D, (), class::tags::Resizable>
where
    T: H5Type + Clone,
    D: NxContainerAttributes,
{
    pub(crate) fn resizable(
        self,
        initial_size: usize,
        chunk_size: usize,
    ) -> NexusDatasetBuilder<T, D, class::tags::Resizable, ()> {
        NexusDatasetBuilder {
            class: class::Resizable {
                initial_size,
                chunk_size,
            },
            phantom: PhantomData,
        }
    }
}

impl<T, D, C0> NexusDatasetBuilder<T, D, C0, ()>
where
    T: H5Type + Clone,
    D: NxContainerAttributes + 'static,
    C0: class::tags::Tag<T> + 'static,
{
    pub(crate) fn finish(
        self,
        name: &str,
        parent_content_register: RcGroupContentRegister,
    ) -> NexusDataset<T, D, C0> {
        let attributes_register = RcAttributeRegister::new(Mutex::new(Vec::new()));

        if let Some(units) = D::UNITS {
            NexusAttribute::begin()
                .fixed_value(VarLenAscii::from_ascii(&units.to_string()).expect(""))
                .finish("units", attributes_register.clone());
        }

        let rc = Rc::new(Mutex::new(UnderlyingNexusDataset {
            name: name.to_owned(),
            attributes: D::new(attributes_register.clone()),
            attributes_register,
            class: self.class,
            dataset: None,
        }));
        parent_content_register
            .lock()
            .expect("Lock Exists")
            .push(rc.clone());
        rc
    }
}
