pub(crate) mod elements;
mod groups;

use std::path::Path;

use elements::{
    group::{GroupBuildable, NexusGroup},
    NxLivesInGroup,
};
use groups::NXRoot;
use hdf5::{types::VarLenUnicode, File};

type H5String = VarLenUnicode;

pub(crate) mod nexus_class {
    pub(crate) const DETECTOR: &str = "NXdetector";
    pub(crate) const ENTRY: &str = "NXentry";
    pub(crate) const ENVIRONMENT: &str = "NXenvironment";
    pub(crate) const EVENT_DATA: &str = "NXevent_data";
    pub(crate) const GEOMETRY: &str = "NXgeometry";
    pub(crate) const INSTRUMENT: &str = "NXinstrument";
    pub(crate) const LOG: &str = "NXlog";
    pub(crate) const PERIOD: &str = "NXperiod";
    pub(crate) const ROOT: &str = "NX_root";
    pub(crate) const RUNLOG: &str = "NXrunlog";
    pub(crate) const SAMPLE: &str = "NXsample";
    pub(crate) const SELOG: &str = "IXselog";
    pub(crate) const SELOG_BLOCK: &str = "IXseblock";
    pub(crate) const SOURCE: &str = "NXsource";
    pub(crate) const USER: &str = "NXuser";
}

pub(crate) struct Nexus {
    file: Option<File>,
    nx_root: NexusGroup<NXRoot>,
}

impl Nexus {
    pub(crate) fn new(filename: &Path) -> anyhow::Result<Self> {
        Ok(Self {
            file: Some(File::create(filename).expect("")),
            nx_root: NexusGroup::new_toplevel(
                filename
                    .file_name()
                    .ok_or(anyhow::anyhow!("Path Error: {filename:?}"))?
                    .to_str()
                    .ok_or(anyhow::anyhow!("Conversion Error: {filename:?}"))?,
            ),
        })
    }

    pub(crate) fn get_root(&self) -> &NexusGroup<NXRoot> {
        &self.nx_root
    }

    pub(crate) fn get_root_mut(&mut self) -> &mut NexusGroup<NXRoot> {
        &mut self.nx_root
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
        if self.file.is_some() {
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