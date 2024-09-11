use hdf5::{Group, Location};
use supermusr_streaming_types::ecs_pl72_run_start_generated::RunStart;

use crate::schematic::{
    elements::{
        attribute::NexusAttribute, dataset::NexusDataset, NexusBuildable, NexusBuilderFinished, NexusDatasetDef, NexusError, NexusGroupDef, NexusHandleMessage, NexusPushMessage
    },
    nexus_class, H5String,
};

#[derive(Clone)]
struct NameAttributes {
    role: NexusAttribute<H5String>,
}

impl NexusDatasetDef for NameAttributes {
    fn new() -> Self {
        Self {
            role: NexusAttribute::begin("role")
                .default_value(Default::default())
                .finish(),
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

impl NexusGroupDef for User {
    const CLASS_NAME: &'static str = nexus_class::USER;

    fn new() -> Self {
        Self {
            name: NexusDataset::begin("name")
                .default_value(Default::default())
                .finish(),
            affiliation: NexusDataset::begin("affiliation")
                .default_value(Default::default())
                .finish(),
            address: NexusDataset::begin("address")
                .default_value(Default::default())
                .finish(),
            telephone_number: NexusDataset::begin("telephone_number")
                .default_value(Default::default())
                .finish(),
            fax_number: NexusDataset::begin("fax_number")
                .default_value(Default::default())
                .finish(),
            email: NexusDataset::begin("email")
                .default_value(Default::default())
                .finish(),
            facility_user_id: NexusDataset::begin("facility_user_id")
                .default_value(Default::default())
                .finish(),
        }
    }
}

impl<'a> NexusHandleMessage<RunStart<'a>> for User {
    fn handle_message(&mut self, message: &RunStart<'a>, location: &Group) -> Result<(), NexusError> {
        Ok(())
    }
}
