use hdf5::{
    types::{TypeDescriptor, VarLenAscii},
    Group,
};
use supermusr_streaming_types::ecs_pl72_run_start_generated::RunStart;

use crate::schematic::elements::{
    attribute::NexusAttribute,
    dataset::{MustEnterAttributes, NexusDataset, RcNexusDatasetVar},
    group::{NexusGroup, NxGroup, NxPushMessage, RcDatasetRegister},
};

pub(super) struct User {
    name: RcNexusDatasetVar<VarLenAscii, MustEnterAttributes<1>>,
    affiliation: RcNexusDatasetVar<VarLenAscii>,
    address: RcNexusDatasetVar<VarLenAscii>,
    telephone_number: RcNexusDatasetVar<VarLenAscii>,
    fax_number: RcNexusDatasetVar<VarLenAscii>,
    email: RcNexusDatasetVar<VarLenAscii>,
    facility_user_id: RcNexusDatasetVar<VarLenAscii>,
}

impl NxGroup for User {
    const CLASS_NAME: &'static str = "NXuser";

    fn new(dataset_register : RcDatasetRegister) -> Self {
        Self {
            name: NexusDataset::begin()
                .attributes([NexusAttribute::new("role", TypeDescriptor::VarLenAscii)])
                .finish("name", dataset_register.clone()),
            affiliation: NexusDataset::begin().finish("affiliation", dataset_register.clone()),
            address: NexusDataset::begin().finish("address", dataset_register.clone()),
            telephone_number: NexusDataset::begin().finish("telephone_number", dataset_register.clone()),
            fax_number: NexusDataset::begin().finish("fax_number", dataset_register.clone()),
            email: NexusDataset::begin().finish("email", dataset_register.clone()),
            facility_user_id: NexusDataset::begin().finish("facility_user_id", dataset_register.clone()),
        }
    }
}

impl<'a> NxPushMessage<RunStart<'a>> for User {
    type MessageType = RunStart<'a>;
    
    fn push_message(&mut self, message: &Self::MessageType) {
        todo!()
    }
}