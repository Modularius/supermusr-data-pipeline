use hdf5::{
    types::{TypeDescriptor, VarLenAscii},
    Group,
};
use supermusr_streaming_types::ecs_pl72_run_start_generated::RunStart;

use crate::schematic::elements::{
    attribute::NexusAttribute,
    dataset::{MustEnterAttributes, NexusDataset},
    group::{NexusGroup, NxGroup, NxPushMessage},
};

pub(super) struct User {
    name: NexusDataset<VarLenAscii, MustEnterAttributes<1>>,
    affiliation: NexusDataset<VarLenAscii>,
    address: NexusDataset<VarLenAscii>,
    telephone_number: NexusDataset<VarLenAscii>,
    fax_number: NexusDataset<VarLenAscii>,
    email: NexusDataset<VarLenAscii>,
    facility_user_id: NexusDataset<VarLenAscii>,
}

impl NxGroup for User {
    const CLASS_NAME: &'static str = "NXuser";

    fn new() -> Self {
        Self {
            name: NexusDataset::begin()
                .attributes([NexusAttribute::new("role", TypeDescriptor::VarLenAscii)])
                .finish("name"),
            affiliation: NexusDataset::begin().finish("affiliation"),
            address: NexusDataset::begin().finish("address"),
            telephone_number: NexusDataset::begin().finish("telephone_number"),
            fax_number: NexusDataset::begin().finish("fax_number"),
            email: NexusDataset::begin().finish("email"),
            facility_user_id: NexusDataset::begin().finish("facility_user_id"),
        }
    }

    fn create(&mut self, this: &Group) {
        self.name.create(this);
        self.affiliation.create(this);
        self.address.create(this);
        self.telephone_number.create(this);
        self.fax_number.create(this);
        self.email.create(this);
        self.facility_user_id.create(this);
    }

    fn open(&mut self, this: &Group) {
        self.name.open(this);
        self.affiliation.open(this);
        self.address.open(this);
        self.telephone_number.open(this);
        self.fax_number.open(this);
        self.email.open(this);
        self.facility_user_id.open(this);
    }

    fn close(&mut self) {
        self.name.close();
        self.affiliation.close();
        self.address.close();
        self.telephone_number.close();
        self.fax_number.close();
        self.email.close();
        self.facility_user_id.close();
    }
}

impl<'a> NxPushMessage<RunStart<'a>> for User {
    type MessageType = RunStart<'a>;
    
    fn push_message(&mut self, message: &Self::MessageType) {
        todo!()
    }
}