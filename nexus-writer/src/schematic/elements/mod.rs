use dataset::NxContainerAttributes;
use hdf5::{Group, Location};

pub(crate) mod attribute;
pub(crate) mod dataset;
pub(crate) mod group;

pub(crate) trait NxLivesInGroup {
    fn create(&mut self, parent: &Group) -> anyhow::Result<()>;
    fn open(&mut self, parent: &Group) -> anyhow::Result<()>;
    fn close(&mut self) -> anyhow::Result<()>;
}

// Nexus Value Options
pub(crate) trait FixedValueOption: Clone {}

#[derive(Clone)]
pub(crate) struct MustEnterFixedValue {}
impl FixedValueOption for MustEnterFixedValue {}

#[derive(Clone)]
pub(crate) struct NoFixedValueNeeded {}
impl FixedValueOption for NoFixedValueNeeded {}
