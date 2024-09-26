use hdf5::Group;
use supermusr_streaming_types::ecs_pl72_run_start_generated::RunStart;

use crate::{
    elements::{
        attribute::{NexusAttribute, NexusAttributeMut},
        dataset::{NexusDataset, NexusDatasetMut},
        traits::{
            NexusDataHolderScalarMutable, NexusDatasetDef, NexusGroupDef, NexusHandleMessage,
        },
    },
    error::NexusPushError,
    nexus::NexusSettings,
    schematic::{nexus_class, H5String},
};

#[derive(Clone)]
struct NameAttributes {
    _role: NexusAttributeMut<H5String>,
}

impl NexusDatasetDef for NameAttributes {
    fn new() -> Self {
        Self {
            _role: NexusAttribute::new_with_default("role"),
        }
    }
}

pub(super) struct User {
    _name: NexusDatasetMut<H5String, NameAttributes>,
    _affiliation: NexusDatasetMut<H5String>,
    _address: NexusDatasetMut<H5String>,
    _telephone_number: NexusDatasetMut<H5String>,
    _fax_number: NexusDatasetMut<H5String>,
    _email: NexusDatasetMut<H5String>,
    _facility_user_id: NexusDatasetMut<H5String>,
}

impl NexusGroupDef for User {
    const CLASS_NAME: &'static str = nexus_class::USER;
    type Settings = NexusSettings;

    fn new(_settings: &NexusSettings) -> Self {
        Self {
            _name: NexusDataset::new_with_default("name"),
            _affiliation: NexusDataset::new_with_default("affiliation"),
            _address: NexusDataset::new_with_default("address"),
            _telephone_number: NexusDataset::new_with_default("telephone_number"),
            _fax_number: NexusDataset::new_with_default("fax_number"),
            _email: NexusDataset::new_with_default("email"),
            _facility_user_id: NexusDataset::new_with_default("facility_user_id"),
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
