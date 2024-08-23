use hdf5::{types::VarLenAscii, Group};

use crate::schematic::elements::{
    attribute::{NexusAttribute, NexusUnits},
    dataset::{MustEnterAttributes, NexusDataset, RcNexusDatasetVar},
    group::{NexusGroup, NxGroup, NxPushMessage, RcDatasetRegister},
};

pub(super) struct Log {
    time: RcNexusDatasetVar<u32, MustEnterAttributes<2>>,
    value: RcNexusDatasetVar<u32>,
}

impl NxGroup for Log {
    const CLASS_NAME: &'static str = "NXperiod";

    fn new(dataset_register : RcDatasetRegister) -> Self {
        Self {
            time: NexusDataset::begin()
                .attributes([NexusAttribute::units(NexusUnits::Nanoseconds)])
                .finish("time", dataset_register.clone()),
            value: NexusDataset::begin().finish("value", dataset_register.clone()),
        }
    }
}

pub(super) struct ValueLog {
    alarm_severity: RcNexusDatasetVar<VarLenAscii>,
    alarm_status: RcNexusDatasetVar<VarLenAscii>,
    alarm_time: RcNexusDatasetVar<i64>,
    time: RcNexusDatasetVar<u32, MustEnterAttributes<1>>,
    value: RcNexusDatasetVar<u32>,
}

impl NxGroup for ValueLog {
    const CLASS_NAME: &'static str = "NXlog";

    fn new(dataset_register : RcDatasetRegister) -> Self {
        Self {
            alarm_severity: NexusDataset::begin().finish("alarm_severity", dataset_register.clone()),
            alarm_status: NexusDataset::begin().finish("alarm_status", dataset_register.clone()),
            alarm_time: NexusDataset::begin().finish("alarm_time", dataset_register.clone()),
            time: NexusDataset::begin().finish("time", dataset_register.clone()),
            value: NexusDataset::begin().finish("value", dataset_register.clone()),
        }
    }
}