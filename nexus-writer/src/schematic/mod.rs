pub(crate) mod groups;

use hdf5::types::VarLenUnicode;

pub(crate) type H5String = VarLenUnicode;

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