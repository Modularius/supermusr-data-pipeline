
use std::{fs::create_dir_all, marker::PhantomData, ops::Deref};
use chrono::{DateTime,Utc};
use hdf5::{plist::DatasetCreateBuilder, types::TypeDescriptor, Attribute, AttributeBuilder, AttributeBuilderData, Dataset, File, Group, H5Type};


#[derive(Default)]
pub(crate) struct NexusAttribute {
    name: String,
    type_descriptor : TypeDescriptor,
    value: Option<u32>,
    attribute: Option<Attribute>
}

impl NexusAttribute {
    pub(crate) fn new(name: &str, type_descriptor : TypeDescriptor) -> Self {
        Self {
            name: name.to_owned(),
            type_descriptor,
            ..Default::default()
        }
    }
}