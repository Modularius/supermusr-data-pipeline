use crate::schematic::{elements::{dataset::NexusDataset, NexusGroup, NxGroup}, groups::log::Log};


pub(super) struct Periods {
    number: NexusDataset<u32>,
    period_types: NexusDataset<u32>,
    frames_requested: NexusDataset<u32>,
    output: NexusDataset<u32>,
    labels: NexusDataset<u32>,
    raw_frames: NexusDataset<u32>,
    good_frames: NexusDataset<u32>,
    sequences: NexusDataset<u32>,
    counts: NexusGroup<Log>,
}

impl NxGroup for Periods {
    const CLASS_NAME : &'static str = "NXperiod";

    fn new() -> Self {
        Self {
            number: NexusDataset::new("number"),
            period_types: NexusDataset::new("type"),
            frames_requested: NexusDataset::new("frames_requested"),
            output: NexusDataset::new("output"),
            labels: NexusDataset::new("labels"),
            raw_frames: NexusDataset::new("raw_frames"),
            good_frames: NexusDataset::new("good_frames"),
            sequences: NexusDataset::new("sequences"),
            counts: NexusGroup::new("counts"),
        }
    }
}
