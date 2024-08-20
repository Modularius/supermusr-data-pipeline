use hdf5::{types::VarLenAscii, AttributeBuilderData};

use crate::schematic::elements::{attribute::NexusAttribute, dataset::{NexusDataset, NexusUnits}, NexusGroup, NxGroup};


pub(super) struct Log {
    time: NexusDataset<u32, "time",
        {[AttributeBuilderData::new().create("Start")]},
        {NexusUnits::Second}>,
    value: NexusDataset<u32, "value">,
}

impl NxGroup for Log {
    const CLASS_NAME : &'static str = "NXperiod";

    fn new() -> Self {
        Self {
            time: NexusDataset::new("time"),
            value: NexusDataset::new("value"),
        }
    }
}


pub(super) struct ValueLog {
    alarm_severity: NexusDataset<VarLenAscii, "alarm_severity">,
    alarm_status: NexusDataset<VarLenAscii, "alarm_status">,
    alarm_time: NexusDataset<i64, "alarm_time">,
    time: NexusDataset<u32, "time",
        {[AttributeBuilderData::new().create("offset")]}
    >,
    value: NexusDataset<u32, "value">,
}

impl NxGroup for ValueLog {
    const CLASS_NAME : &'static str = "NXperiod";

    fn new() -> Self {
        Self {
            alarm_severity: NexusDataset::new(),
            alarm_status: NexusDataset::new(),
            alarm_time: NexusDataset::new(),
            time: NexusDataset::new(),
            value: NexusDataset::new(),
        }
    }
}