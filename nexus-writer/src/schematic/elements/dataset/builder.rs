use crate::schematic::elements::traits::{self, Buildable};
use hdf5::{types::VarLenAscii, Dataset, Group, H5Type};
use std::marker::PhantomData;

use super::{
    super::{attribute::NexusAttribute, group::GroupContentRegister},
    AttributeRegister, NexusDataset, NxDataset, UnderlyingNexusDataset,
};

#[derive(Clone)]
pub(crate) struct NexusDatasetBuilder<T, D, C0, C>
where
    T: H5Type + Clone,
    D: NxDataset,
    C0: traits::tags::Tag<T, Group, Dataset>,
    C: traits::tags::Tag<T, Group, Dataset>,
{
    name: String,
    class: C0::ClassType,
    phantom: PhantomData<(T, D, C)>,
}

impl<T, D, C> NexusDatasetBuilder<T, D, (), C>
where
    T: H5Type + Clone,
    D: NxDataset,
    C: traits::tags::Tag<T, Group, Dataset>,
{
    pub(super) fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            class: (),
            phantom: PhantomData,
        }
    }
}

impl<T, D> NexusDatasetBuilder<T, D, (), traits::tags::Constant>
where
    T: H5Type + Clone,
    D: NxDataset,
{
    pub(crate) fn fixed_value(
        self,
        value: T,
    ) -> NexusDatasetBuilder<T, D, traits::tags::Constant, ()> {
        NexusDatasetBuilder {
            name: self.name,
            class: traits::Constant(value),
            phantom: PhantomData,
        }
    }
}

impl<T, D> NexusDatasetBuilder<T, D, (), traits::tags::Resizable>
where
    T: H5Type + Clone,
    D: NxDataset,
{
    pub(crate) fn resizable(
        self,
        initial_size: usize,
        chunk_size: usize,
    ) -> NexusDatasetBuilder<T, D, traits::tags::Resizable, ()> {
        NexusDatasetBuilder {
            name: self.name,
            class: traits::Resizable {
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
    D: NxDataset + 'static,
    C0: traits::tags::Tag<T, Group, Dataset> + 'static,
{
    pub(crate) fn finish(
        self,
        parent_content_register: &GroupContentRegister,
    ) -> NexusDataset<T, D, C0> {
        let attributes_register = AttributeRegister::default();

        if let Some(units) = D::UNITS {
            NexusAttribute::begin("units")
                .fixed_value(VarLenAscii::from_ascii(&units.to_string()).expect(""))
                .finish(&attributes_register);
        }

        let rc = NexusDataset::new(UnderlyingNexusDataset {
            name: self.name,
            attributes: D::new(attributes_register.clone()),
            attributes_register,
            class: self.class,
            dataset: None,
        });
        parent_content_register.lock_mutex().push(rc.clone_inner());
        rc
    }
}
