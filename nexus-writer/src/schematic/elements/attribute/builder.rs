use std::marker::PhantomData;

use hdf5::{Attribute, Dataset, H5Type};

use crate::schematic::elements::{dataset::AttributeRegister, traits};

use super::{underlying::UnderlyingNexusAttribute, NexusAttribute};

/// NexusAttributeBuilder
#[derive(Clone)]
pub(crate) struct NexusAttributeBuilder<T: H5Type, C, const finished: bool>
where
    T: H5Type + Clone,
    C: traits::tags::Tag<T, Dataset, Attribute>,
{
    name: String,
    class: C::ClassType,
    phantom: PhantomData<(T, C)>,
}

impl<T, C> NexusAttributeBuilder<T, C, false>
where
    T: H5Type + Clone,
    C: traits::tags::Tag<T, Dataset, Attribute>,
{
    pub(super) fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            class: Default::default(),
            phantom: PhantomData,
        }
    }
}

impl<T> NexusAttributeBuilder<T, traits::tags::Mutable, false>
where
    T: H5Type + Default + Clone,
{
    pub(crate) fn default_value(
        self,
        value: T,
    ) -> NexusAttributeBuilder<T, traits::tags::Mutable, true> {
        NexusAttributeBuilder {
            name: self.name,
            class: traits::Mutable(value),
            phantom: PhantomData,
        }
    }
}

impl<T> NexusAttributeBuilder<T, traits::tags::Constant, false>
where
    T: H5Type + Default + Clone,
{
    pub(crate) fn fixed_value(
        self,
        value: T,
    ) -> NexusAttributeBuilder<T, traits::tags::Constant, true> {
        NexusAttributeBuilder {
            name: self.name,
            class: traits::Constant(value),
            phantom: PhantomData,
        }
    }
}

impl<T, C> NexusAttributeBuilder<T, C, true>
where
    T: H5Type + Clone,
    C: traits::tags::Tag<T, Dataset, Attribute> + 'static,
{
    pub(crate) fn finish(
        self,
        parent_content_register: &AttributeRegister,
    ) -> NexusAttribute<T, C> {
        let rc = NexusAttribute::new(UnderlyingNexusAttribute {
            name: self.name,
            class: self.class,
            attribute: None,
        });
        parent_content_register.lock_mutex().push(rc.clone_inner());
        rc
    }
}
