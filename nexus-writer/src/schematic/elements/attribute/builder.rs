use std::{marker::PhantomData, rc::Rc, sync::Mutex};

use hdf5::{Attribute, Dataset, H5Type};
use tracing::instrument;

use crate::schematic::elements::{dataset::AttributeRegister, traits};

use super::{underlying::UnderlyingNexusAttribute, NexusAttribute};

/// NexusAttributeBuilder
#[derive(Clone)]
pub(crate) struct NexusAttributeBuilder<T: H5Type, C0, C>
where
    T: H5Type + Clone,
    C0: traits::tags::Tag<T, Dataset, Attribute>,
    C: traits::tags::Tag<T, Dataset, Attribute>,
{
    name: String,
    class: C0::ClassType,
    phantom: PhantomData<(T, C)>,
}

impl<T, C> NexusAttributeBuilder<T, (), C>
where
    T: H5Type + Clone,
    C: traits::tags::Tag<T, Dataset, Attribute>,
{
    pub(super) fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            class: (),
            phantom: PhantomData,
        }
    }
}

impl<T: H5Type> NexusAttributeBuilder<T, (), traits::tags::Constant>
where
    T: H5Type + Clone,
{
    pub(crate) fn fixed_value(
        self,
        value: T,
    ) -> NexusAttributeBuilder<T, traits::tags::Constant, ()> {
        NexusAttributeBuilder {
            name: self.name,
            class: traits::Constant(value),
            phantom: PhantomData,
        }
    }
}

impl<T, C0> NexusAttributeBuilder<T, C0, ()>
where
    T: H5Type + Clone,
    C0: traits::tags::Tag<T, Dataset, Attribute> + 'static,
{
    pub(crate) fn finish(
        self,
        parent_content_register: &AttributeRegister,
    ) -> NexusAttribute<T, C0> {
        let rc = NexusAttribute::new(UnderlyingNexusAttribute {
            name: self.name,
            class: self.class,
            attribute: None,
        });
        parent_content_register.lock().push(rc.clone_inner());
        rc
    }
}
