mod elements;
mod groups;

use std::path::Path;

use elements::{group::{NexusGroup, NxPushMessage, RcNexusGroup}, NxLivesInGroup};
use groups::NXRoot;
use hdf5::File;
use supermusr_streaming_types::{
    aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage,
    ecs_pl72_run_start_generated::RunStart,
};

pub(crate) struct Nexus {
    file: File,
    nx_root: RcNexusGroup<NXRoot>,
}

impl Nexus {
    pub(crate) fn new(filename: &Path) -> Self {
        Self {
            file: File::create(filename).expect("File is created"),
            nx_root: NexusGroup::new(
                filename
                    .file_name()
                    .expect("Filename exists")
                    .to_str()
                    .expect("str conversion"),
            None),
        }
    }

    pub(crate) fn create(&mut self) {
        self.nx_root.lock().expect("").create(&self.file)
    }

    pub(crate) fn open(&mut self) {
        self.nx_root.lock().expect("").open(&self.file)
    }

    pub(crate) fn close(&mut self) {
        self.nx_root.lock().expect("").close()
    }
}
