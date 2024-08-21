use hdf5::{types::VarLenAscii, Group};

use crate::schematic::elements::{
    attribute::{NexusAttribute, NexusUnits},
    dataset::{MustEnterAttributes, NexusDataset},
    group::{NexusGroup, NxGroup, NxPushMessage},
};

pub(super) struct Log {
    time: NexusDataset<u32, MustEnterAttributes<2>>,
    value: NexusDataset<u32>,
}

impl NxGroup for Log {
    const CLASS_NAME: &'static str = "NXperiod";

    fn new() -> Self {
        Self {
            time: NexusDataset::begin()
                .attributes([NexusAttribute::units(NexusUnits::Nanoseconds)])
                .finish("time"),
            value: NexusDataset::begin().finish("value"),
        }
    }

    fn create(&mut self, this: &Group) {
        self.time.create(this);
        self.value.create(this);
    }

    fn open(&mut self, this: &Group) {
        self.time.open(this);
        self.value.open(this);
    }

    fn close(&mut self) {
        self.time.close();
        self.value.close();
    }
}

pub(super) struct ValueLog {
    alarm_severity: NexusDataset<VarLenAscii>,
    alarm_status: NexusDataset<VarLenAscii>,
    alarm_time: NexusDataset<i64>,
    time: NexusDataset<u32, MustEnterAttributes<1>>,
    value: NexusDataset<u32>,
}

impl NxGroup for ValueLog {
    const CLASS_NAME: &'static str = "NXperiod";

    fn new() -> Self {
        Self {
            alarm_severity: NexusDataset::begin().finish("alarm_severity"),
            alarm_status: NexusDataset::begin().finish("alarm_status"),
            alarm_time: NexusDataset::begin().finish("alarm_time"),
            time: NexusDataset::begin().finish("time"),
            value: NexusDataset::begin().finish("value"),
        }
    }

    fn create(&mut self, this: &Group) {
        self.alarm_severity.create(this);
        self.alarm_status.create(this);
        self.alarm_time.create(this);
        self.time.create(this);
        self.value.create(this);
    }

    fn open(&mut self, this: &Group) {
        self.alarm_severity.open(this);
        self.alarm_status.open(this);
        self.alarm_time.open(this);
        self.time.open(this);
        self.value.open(this);
    }

    fn close(&mut self) {
        self.alarm_severity.close();
        self.alarm_status.close();
        self.alarm_time.close();
        self.time.close();
        self.value.close();
    }
}