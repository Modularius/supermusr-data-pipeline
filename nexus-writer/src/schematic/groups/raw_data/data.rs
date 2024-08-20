use hdf5::types::VarLenAscii;

use crate::schematic::elements::{attribute::NexusAttribute, dataset::{NexusAttributedUnitedDataset, NexusDataset, NexusUnitedDataset, NexusUnits, NxDataset}, NexusGroup, NxGroup};


pub(super) struct Data {
    event_id: NexusDataset<u32>,
    event_index: NexusDataset<u32>,
    event_time_offset: NexusUnitedDataset<u32>,
    event_time_zero: NexusAttributedUnitedDataset<u64, 1>,
    event_period_number: NexusDataset<u64>,
    event_pulse_height: NexusDataset<f64>,
}

impl NxGroup for Data {
    const CLASS_NAME : &'static str = "NXperiod";

    fn new() -> Self {
        Self {
            event_id: NexusDataset::new("event_id"),
            event_index: NexusDataset::new("event_index"),
            event_time_offset: NexusDataset::new("event_time_offset")
                .with_units(NexusUnits::Nanoseconds),
            event_time_zero: NexusDataset::new("event_time_zero")
                .with_units(NexusUnits::Nanoseconds)
                .with_attributes([NexusAttribute::new()]),
            event_period_number: NexusDataset::new("event_period_number"),
            event_pulse_height: NexusDataset::new("event_pulse_height"),
        }
    }
}