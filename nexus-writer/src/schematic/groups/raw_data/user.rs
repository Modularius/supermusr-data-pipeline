use supermusr_streaming_types::ecs_pl72_run_start_generated::RunStart;

use crate::schematic::{
    elements::{
        attribute::{NexusAttribute, RcNexusAttributeVar},
        dataset::{Buildable, NexusDataset, NxContainerAttributes, RcAttributeRegister},
        group::{NxGroup, NxPushMessage, RcGroupContentRegister},
    },
    nexus_class, H5String,
};

#[derive(Clone)]
struct NameAttributes {
    role: RcNexusAttributeVar<H5String>,
}

impl NxContainerAttributes for NameAttributes {
    fn new(attribute_register: RcAttributeRegister) -> Self {
        Self {
            role: NexusAttribute::begin().finish("role", attribute_register.clone()),
        }
    }
}

pub(super) struct User {
    name: NexusDataset<H5String, NameAttributes>,
    affiliation: NexusDataset<H5String>,
    address: NexusDataset<H5String>,
    telephone_number: NexusDataset<H5String>,
    fax_number: NexusDataset<H5String>,
    email: NexusDataset<H5String>,
    facility_user_id: NexusDataset<H5String>,
}

impl NxGroup for User {
    const CLASS_NAME: &'static str = nexus_class::USER;

    fn new(dataset_register: RcGroupContentRegister) -> Self {
        Self {
            name: NexusDataset::begin().finish("name", dataset_register.clone()),
            affiliation: NexusDataset::begin().finish("affiliation", dataset_register.clone()),
            address: NexusDataset::begin().finish("address", dataset_register.clone()),
            telephone_number: NexusDataset::begin()
                .finish("telephone_number", dataset_register.clone()),
            fax_number: NexusDataset::begin().finish("fax_number", dataset_register.clone()),
            email: NexusDataset::begin().finish("email", dataset_register.clone()),
            facility_user_id: NexusDataset::begin()
                .finish("facility_user_id", dataset_register.clone()),
        }
    }
}

impl<'a> NxPushMessage<RunStart<'a>> for User {
    type MessageType = RunStart<'a>;

    fn push_message(&self, message: &Self::MessageType) -> anyhow::Result<()> {
        Ok(())
    }
}
