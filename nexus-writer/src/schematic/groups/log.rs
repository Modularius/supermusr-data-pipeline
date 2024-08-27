use hdf5::{types::VarLenAscii, Group};

use crate::schematic::elements::{
    attribute::{NexusAttribute, NexusUnits, RcNexusAttributeFixed, RcNexusAttributeVar},
    dataset::{NexusDataset, NxContainerAttributes, RcAttributeRegister, RcNexusDatasetResize, RcNexusDatasetVar},
    group::{NexusGroup, NxGroup, NxPushMessage, RcGroupContentRegister},
};

#[derive(Clone)]
struct TimeAttributes {
    offset: RcNexusAttributeVar<VarLenAscii>,
}

impl NxContainerAttributes for TimeAttributes {
    const UNITS: Option<NexusUnits> = Some(NexusUnits::Nanoseconds);

    fn new(attribute_register: RcAttributeRegister) -> Self {
        Self {
            offset: NexusAttribute::begin().finish("offset", attribute_register.clone()),
        }
    }
}

pub(super) struct Log {
    time: RcNexusDatasetResize<u32, TimeAttributes>,
    value: RcNexusDatasetResize<u32>,
}

impl NxGroup for Log {
    const CLASS_NAME: &'static str = "NXlog";

    fn new(dataset_register: RcGroupContentRegister) -> Self {
        Self {
            time: NexusDataset::begin().resizable(0, 128).finish("time", dataset_register.clone()),
            value: NexusDataset::begin().resizable(0, 128).finish("value", dataset_register.clone()),
        }
    }
}

pub(super) struct ValueLog {
    alarm_severity: RcNexusDatasetResize<VarLenAscii>,
    alarm_status: RcNexusDatasetResize<VarLenAscii>,
    alarm_time: RcNexusDatasetResize<i64>,
    time: RcNexusDatasetResize<u32, TimeAttributes>,
    value: RcNexusDatasetResize<u32>,
}

impl NxGroup for ValueLog {
    const CLASS_NAME: &'static str = "NXlog";

    fn new(dataset_register: RcGroupContentRegister) -> Self {
        Self {
            alarm_severity: NexusDataset::begin().resizable(0, 128)
                .finish("alarm_severity", dataset_register.clone()),
            alarm_status: NexusDataset::begin().resizable(0, 128).finish("alarm_status", dataset_register.clone()),
            alarm_time: NexusDataset::begin().resizable(0, 128).finish("alarm_time", dataset_register.clone()),
            time: NexusDataset::begin().resizable(0, 128).finish("time", dataset_register.clone()),
            value: NexusDataset::begin().resizable(0, 128).finish("value", dataset_register.clone()),
        }
    }
}
