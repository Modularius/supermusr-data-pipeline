pub(crate) mod elements;
mod groups;

use std::path::{Path, PathBuf};
use elements::{error::{ClosingError, CreationError, HDF5Error, NexusError, OpeningError}, group::{NexusGroup, TopLevelNexusGroup}, traits::TopGroupBuildable, NxLivesInGroup};
use groups::NXRoot;
use hdf5::{types::VarLenUnicode, File, FileBuilder};
use thiserror::Error;

use crate::nexus::NexusSettings;

type H5String = VarLenUnicode;

#[derive(Debug,Error)]
pub(crate) enum NexusRootError {
    #[error("HDF5 Error: {0}")]
    HDF5(#[from] HDF5Error),
    #[error("Cannot Create HDF5 Object: {0}")]
    Create(#[from] CreationError),
    #[error("Cannot Open HDF5 Object: {0}")]
    Open(#[from] OpeningError),
    #[error("Cannot Close HDF5 Object: {0}")]
    Close(#[from] ClosingError),
    #[error("Cannot Create HDF5 File")]
    CreateFile,
    #[error("Cannot Open HDF5 File")]
    OpenFile,
    #[error("Cannot Close HDF5 File")]
    CloseFile,
    #[error("Nexus Error {0}")]
    Nexus(#[from]NexusError),
    #[error("Path Error {0}")]
    Path(PathBuf),
    #[error("Path Conversion Error {0}")]
    PathConversion(PathBuf),
    #[error("SWMR Error {0} with file {1}")]
    Swmr(i32, PathBuf),
}

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
    settings: NexusSettings,
    file: Option<File>,
    nx_root: TopLevelNexusGroup<NXRoot>,
}

impl Nexus {
    pub(crate) fn new(filename: &Path, settings: &NexusSettings) -> Result<Self, NexusRootError> {
        let file = FileBuilder::new()
            .with_fapl(|fapl| {
                fapl.libver_bounds(
                    hdf5::file::LibraryVersion::V110,
                    hdf5::file::LibraryVersion::V110,
                )
            })
            .create(filename).map_err(HDF5Error::General)?;
        {
            if settings.use_swmr {
                let err = unsafe { hdf5_sys::h5f::H5Fstart_swmr_write(file.id()) };
                if err != 0 {
                    return Err(NexusRootError::Swmr(err, filename.to_owned()));
                }
            }
        }
        Ok(Self {
            file: Some(file),
            nx_root: NexusGroup::new_toplevel(
                filename
                    .file_name()
                    .ok_or(NexusRootError::Path(filename.to_owned()))?
                    .to_str()
                    .ok_or(NexusRootError::PathConversion(filename.to_owned()))?,
            ),
            settings: settings.clone()
        })
    }

    pub(crate) fn get_root(&self) -> &TopLevelNexusGroup<NXRoot> {
        &self.nx_root
    }

    pub(crate) fn get_root_mut(&mut self) -> &mut TopLevelNexusGroup<NXRoot> {
        &mut self.nx_root
    }

    pub(crate) fn create(&mut self) -> Result<(),NexusRootError> {
        if let Some(file) = &mut self.file {
            Ok(self.nx_root.apply_lock().create(file)?)
        } else {
            Err(NexusRootError::CreateFile)
        }
    }

    pub(crate) fn open(&mut self) -> Result<(),NexusRootError> {
        if let Some(file) = &mut self.file {
            Ok(self.nx_root.apply_lock().open(file)?)
        } else {
            Err(NexusRootError::OpenFile)
        }
    }

    pub(crate) fn close(&mut self) -> Result<(),NexusRootError> {
        if self.file.is_some() {
            Ok(self.nx_root.apply_lock().close()?)
        } else {
            Err(NexusRootError::CloseFile)
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
