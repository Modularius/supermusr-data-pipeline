use hdf5::Group;
use supermusr_streaming_types::ecs_pl72_run_start_generated::RunStart;

use crate::{
    error::NexusPushError,
    nexus::NexusSettings,
    schematic::{
        elements::{
            attribute::NexusAttribute,
            dataset::{NexusDataset, NexusDatasetMut},
            traits::{NexusBuildable, NexusDatasetDef, NexusGroupDef, NexusHandleMessage},
        },
        nexus_class, H5String,
    },
};

#[derive(Clone)]
struct NameAttributes {
    role: NexusAttribute<H5String>,
}

impl NexusDatasetDef for NameAttributes {
    fn new() -> Self {
        Self {
            role: NexusAttribute::begin("role").finish_with_auto_default(),
        }
    }
}

pub(super) struct User {
    name: NexusDatasetMut<H5String, NameAttributes>,
    affiliation: NexusDatasetMut<H5String>,
    address: NexusDatasetMut<H5String>,
    telephone_number: NexusDatasetMut<H5String>,
    fax_number: NexusDatasetMut<H5String>,
    email: NexusDatasetMut<H5String>,
    facility_user_id: NexusDatasetMut<H5String>,
}

impl NexusGroupDef for User {
    const CLASS_NAME: &'static str = nexus_class::USER;
    type Settings = NexusSettings;

    fn new(_settings: &NexusSettings) -> Self {
        Self {
            name: NexusDataset::begin("name").finish_with_auto_default(),
            affiliation: NexusDataset::begin("affiliation").finish_with_auto_default(),
            address: NexusDataset::begin("address").finish_with_auto_default(),
            telephone_number: NexusDataset::begin("telephone_number").finish_with_auto_default(),
            fax_number: NexusDataset::begin("fax_number").finish_with_auto_default(),
            email: NexusDataset::begin("email").finish_with_auto_default(),
            facility_user_id: NexusDataset::begin("facility_user_id").finish_with_auto_default(),
        }
    }
}

impl<'a> NexusHandleMessage<RunStart<'a>> for User {
    fn handle_message(
        &mut self,
        message: &RunStart<'a>,
        location: &Group,
    ) -> Result<(), NexusPushError> {
        Ok(())
    }
}
