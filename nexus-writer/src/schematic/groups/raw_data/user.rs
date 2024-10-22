use hdf5::{Dataset, Group};
use supermusr_streaming_types::ecs_pl72_run_start_generated::RunStart;

use crate::{
    elements::{
        attribute::{NexusAttribute, NexusAttributeMut},
        dataset::{NexusDataset, NexusDatasetMut},
        traits::{
            NexusDataHolderScalarMutable, NexusDataHolderStringMutable, NexusDatasetDef, NexusGroupDef, NexusHandleMessage, NexusPushMessage
        },
    },
    error::NexusPushError,
    nexus::NexusSettings,
    schematic::{nexus_class, H5String},
};

#[derive(Clone)]
struct Name {
    /// role of user e.g. `PI``, `Contact` etc, multiple roles are allowed.
    role: NexusAttributeMut<H5String>,
}

impl NexusDatasetDef for Name {
    fn new() -> Self {
        Self {
            role: NexusAttribute::new_with_default("role"),
        }
    }
}
impl<'a> NexusHandleMessage<RunStart<'a>, Dataset> for Name {
    fn handle_message(
        &mut self,
        message: &RunStart<'a>,
        parent: &Dataset,
    ) -> Result<(), NexusPushError> {
        self.role.write_string(parent, "User's Role")?;
        Ok(())
    }
}


pub(super) struct User {
    name: NexusDatasetMut<H5String, Name>,
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
        message: &RunStart<'a>,
        parent: &Group,
    ) -> Result<(), NexusPushError> {
        self.name.push_message(message, parent)?;
        self.name.write_string(parent, "User's Name")?;
        self.affiliation.write_string(parent, "User's Affiliation")?;
        self.address.write_string(parent, "User's Address")?;
        self.telephone_number.write_string(parent, "User's Phone Number")?;
        self.fax_number.write_string(parent, "User's Fax Number")?;
        self.email.write_string(parent, "User's Email")?;
        self.facility_user_id.write_string(parent, "User ID")?;
        Ok(())
    }
}
