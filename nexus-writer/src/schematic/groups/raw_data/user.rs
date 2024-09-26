use hdf5::Group;
use supermusr_streaming_types::ecs_pl72_run_start_generated::RunStart;

use crate::{
    error::NexusPushError,
    nexus::NexusSettings,
    schematic::{
        elements::{
            attribute::{NexusAttribute, NexusAttributeMut},
            dataset::{NexusDataset, NexusDatasetMut},
            traits::{
                NexusDataHolderScalarMutable, NexusDatasetDef, NexusGroupDef, NexusHandleMessage,
            },
        },
        nexus_class, H5String,
    },
};

#[derive(Clone)]
struct NameAttributes {
    role: NexusAttributeMut<H5String>,
}

impl NexusDatasetDef for NameAttributes {
    fn new() -> Self {
        Self {
            role: NexusAttribute::new_with_default("role"),
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
            name: NexusDataset::new_with_default("name"),
            affiliation: NexusDataset::new_with_default("affiliation"),
            address: NexusDataset::new_with_default("address"),
            telephone_number: NexusDataset::new_with_default("telephone_number"),
            fax_number: NexusDataset::new_with_default("fax_number"),
            email: NexusDataset::new_with_default("email"),
            facility_user_id: NexusDataset::new_with_default("facility_user_id"),
        }
    }
}

impl<'a> NexusHandleMessage<RunStart<'a>> for User {
    fn handle_message(
        &mut self,
        _message: &RunStart<'a>,
        _location: &Group,
    ) -> Result<(), NexusPushError> {
        Ok(())
    }
}
