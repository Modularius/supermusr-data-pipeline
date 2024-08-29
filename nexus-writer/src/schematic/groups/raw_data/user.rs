use supermusr_streaming_types::ecs_pl72_run_start_generated::RunStart;

use crate::schematic::{
    elements::{
        attribute::{NexusAttribute, RcNexusAttributeVar},
        dataset::{AttributeRegister, NexusDataset, NxDataset},
        group::{NxGroup, NxPushMessage, RcGroupContentRegister}, traits::Buildable,
    },
    nexus_class, H5String,
};

#[derive(Clone)]
struct NameAttributes {
    role: RcNexusAttributeVar<H5String>,
}

impl NxDataset for NameAttributes {
    fn new(attribute_register: AttributeRegister) -> Self {
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
            name: NexusDataset::begin("name").finish(&dataset_register),
            affiliation: NexusDataset::begin("affiliation").finish(&dataset_register),
            address: NexusDataset::begin("address").finish(&dataset_register),
            telephone_number: NexusDataset::begin("telephone_number").finish(&dataset_register),
            fax_number: NexusDataset::begin("fax_number").finish(&dataset_register),
            email: NexusDataset::begin("email").finish(&dataset_register),
            facility_user_id: NexusDataset::begin("facility_user_id").finish(&dataset_register),
        }
    }
}

impl<'a> NxPushMessage<RunStart<'a>> for User {
    type MessageType = RunStart<'a>;

    fn push_message(&self, message: &Self::MessageType) -> anyhow::Result<()> {
        Ok(())
    }
}
