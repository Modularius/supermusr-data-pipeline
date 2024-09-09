use crate::schematic::elements::{
    attribute::NexusAttribute,
    dataset::{AttributeRegister, NexusDataset, NxDataset, UnderlyingNexusDataset},
    group::GroupContentRegister,
    traits::{self, Buildable},
};
use hdf5::{types::VarLenAscii, Dataset, Group, H5Type};
use std::marker::PhantomData;

#[derive(Clone)]
pub(crate) struct NexusDatasetBuilder<T, D, C, const finished: bool>
where
    T: H5Type + Clone,
    D: NxDataset,
    C: traits::tags::Tag<T, Group, Dataset>,
{
    name: String,
    class: C::ClassType,
    phantom: PhantomData<(T, D)>,
}

impl<T, D, C> NexusDatasetBuilder<T, D, C, false>
where
    T: H5Type + Clone + Default,
    D: NxDataset,
    C: traits::tags::Tag<T, Group, Dataset>,
{
    pub(super) fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            class: Default::default(),
            phantom: PhantomData,
        }
    }
}

impl<T, D> NexusDatasetBuilder<T, D, traits::tags::Mutable, false>
where
    T: H5Type + Clone + Default,
    D: NxDataset,
{
    pub(crate) fn default_value(
        self,
        default_value: T,
    ) -> NexusDatasetBuilder<T, D, traits::tags::Mutable, true> {
        NexusDatasetBuilder {
            name: self.name,
            class: traits::Mutable(default_value),
            phantom: PhantomData,
        }
    }
}

impl<T, D> NexusDatasetBuilder<T, D, traits::tags::Constant, false>
where
    T: H5Type + Clone + Default,
    D: NxDataset,
{
    pub(crate) fn fixed_value(
        self,
        fixed_value: T,
    ) -> NexusDatasetBuilder<T, D, traits::tags::Constant, true> {
        NexusDatasetBuilder {
            name: self.name,
            class: traits::Constant(fixed_value),
            phantom: PhantomData,
        }
    }
}

impl<T, D> NexusDatasetBuilder<T, D, traits::tags::Resizable, false>
where
    T: H5Type + Clone + Default,
    D: NxDataset,
{
    pub(crate) fn resizable(
        self,
        default_value: T,
        initial_size: usize,
        chunk_size: usize,
    ) -> NexusDatasetBuilder<T, D, traits::tags::Resizable, true> {
        NexusDatasetBuilder {
            name: self.name,
            class: traits::Resizable {
                default_value,
                initial_size,
                chunk_size,
            },
            phantom: PhantomData,
        }
    }
}

impl<T, D, C> NexusDatasetBuilder<T, D, C, true>
where
    T: H5Type + Clone,
    D: NxDataset + 'static,
    C: traits::tags::Tag<T, Group, Dataset> + 'static,
{
    pub(crate) fn finish(
        self,
        parent_content_register: &GroupContentRegister,
    ) -> NexusDataset<T, D, C> {
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
