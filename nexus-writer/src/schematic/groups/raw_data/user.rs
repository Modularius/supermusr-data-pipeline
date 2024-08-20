use hdf5::types::VarLenAscii;

use crate::schematic::elements::{dataset::NexusDataset, NxGroup};


pub(super) struct User {
    name: NexusDataset<VarLenAscii>,
    affiliation: NexusDataset<VarLenAscii>,
    address: NexusDataset<VarLenAscii>,
    telephone_number: NexusDataset<VarLenAscii>,
    fax_number: NexusDataset<VarLenAscii>,
    email: NexusDataset<VarLenAscii>,
    facility_user_id: NexusDataset<u32>,
}

impl NxGroup for User {
    const CLASS_NAME : &'static str = "NXuser";

    fn new() -> Self {
        Self {
            name: NexusDataset::new(""),
            affiliation: NexusDataset::new(""),
            address: NexusDataset::new(""),
            telephone_number: NexusDataset::new(""),
            fax_number: NexusDataset::new(""),
            email: NexusDataset::new(""),
            facility_user_id: NexusDataset::new(""),
        }
    }
}