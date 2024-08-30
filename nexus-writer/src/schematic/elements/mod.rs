use hdf5::Group;

pub(crate) mod attribute;
pub(crate) mod dataset;
pub(crate) mod group;
pub(crate) mod traits;

pub(crate) trait NxLivesInGroup {
    fn create(&mut self, parent: &Group) -> anyhow::Result<()>;
    fn open(&mut self, parent: &Group) -> anyhow::Result<()>;
    fn close(&mut self) -> anyhow::Result<()>;
}
