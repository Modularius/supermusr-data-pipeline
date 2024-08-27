pub(crate) mod elements;
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
    file: Option<File>,
    nx_root: RcNexusGroup<NXRoot>,
}

impl Nexus {
    pub(crate) fn new(filename: &Path) -> anyhow::Result<Self> {
        Ok(Self {
            file: Some(File::create(filename).expect("")),
            nx_root: NexusGroup::new(
                filename
                    .file_name()
                    .ok_or(anyhow::anyhow!("Path Error: {filename:?}"))?
                    .to_str()
                    .ok_or(anyhow::anyhow!("Conversion Error: {filename:?}"))?,
            None),
        })
    }

    pub(crate) fn get_root(&mut self) -> RcNexusGroup<NXRoot> {
        self.nx_root.clone()
    }

    pub(crate) fn create(&mut self) -> anyhow::Result<()> {
        if let Some(file) = &mut self.file {
            Ok(self.nx_root.lock().expect("Can lock").create(file)?)
        } else {
            Err(anyhow::anyhow!("No File"))
        }
    }

    pub(crate) fn open(&mut self) -> anyhow::Result<()> {
        if let Some(file) = &mut self.file {
            Ok(self.nx_root.lock().expect("Can lock").open(file)?)
        } else {
            Err(anyhow::anyhow!("No File"))
        }
    }

    pub(crate) fn close(&mut self) -> anyhow::Result<()> {
        if let Some(file) = &mut self.file {
            Ok(self.nx_root.lock().expect("Can lock").close()?)
        } else {
            Err(anyhow::anyhow!("No File"))
        }
    }

    pub(crate) fn close_file(&mut self) -> anyhow::Result<()> {
        if let Some(file) = self.file.take() {
            Ok(file.close()?)
        } else {
            Err(anyhow::anyhow!("No File"))
        }
    }
}
