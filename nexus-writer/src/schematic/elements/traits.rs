use super::{error::CreationError, group::GroupContentRegister};
use hdf5::H5Type;
use thiserror::Error;

/// Both NexusDataset and NexusAttribute own a field `class` whose type implements this trait.
/// They use it to create their respective hdf5 objects.
/// # Generics
/// - T: H5Type
/// - P: H5DF Container type of O
/// - O: H5DF type of this class
/// # Method
/// - create: creates an instance of O as a child of `parent`, with given name.
pub(crate) trait Class<T, P, O>: Clone + Default
where
    T: H5Type,
{
    fn create(&self, parent: &P, name: &str) -> Result<O, CreationError>;
}

/// Facilitates a Dataset or Attribute with a writable scalar value of type T.
#[derive(Default, Clone)]
pub(crate) struct Mutable<T: H5Type>(pub(crate) T);

/// Facilitates a Dataset or Attribute with a constant value of type T.
#[derive(Default, Clone)]
pub(crate) struct Constant<T: H5Type>(pub(crate) T);

/// Facilitates a Dataset or Attribute with a resizable array of values
#[derive(Default, Clone)]
pub(crate) struct Resizable<T: H5Type> {
    pub(crate) default_value: T,
    pub(crate) initial_size: usize,
    pub(crate) chunk_size: usize,
}

/// Module consists of structs which "tag" an instance of NexusDataset or NexusAttribute
/// as expecting a `class` field of a prescribed type
pub(crate) mod tags {
    use hdf5::H5Type;

    /// "Tags" implement this trait which defines the type of the `class` field
    /// in NexusDataset and NexusAttribute instances
    pub(crate) trait Tag<T: H5Type, P, O>: Clone
    where
        T: H5Type,
    {
        type ClassType: super::Class<T, P, O>;
    }

    #[derive(Clone)]
    pub(crate) struct Mutable;

    #[derive(Clone)]
    pub(crate) struct Constant;

    #[derive(Clone)]
    pub(crate) struct Resizable;
}

/// Trait for NexusDataset and NexusAttribute
/// Buildable means these types can be constructed
/// by calling `build` to create the appropriate builder.
pub(crate) trait Buildable<T>
where
    T: H5Type + Clone,
{
    type BuilderType;

    fn begin(name: &str) -> Self::BuilderType;
}

/// Trait for those instances of NexusDataset and NexusAttribute
/// which represent writable scalars.
pub(crate) trait CanWriteScalar {
    type Type: H5Type;

    fn write_scalar(&self, value: Self::Type) -> Result<(), hdf5::Error>;

    fn read_scalar(&self) -> Result<Self::Type, hdf5::Error>;
}

/// Trait for NexusDataset instances which represent appendable array datasets.
pub(crate) trait CanAppend {
    type Type: H5Type;

    fn append(&self, value: &[Self::Type]) -> Result<usize, hdf5::Error>;
}

/// Trait for NexusGroup instances which represent subgroups.
pub(crate) trait SubgroupBuildable {
    fn new_subgroup(name: &str, parent_content_register: &GroupContentRegister) -> Self;
}

/// Trait for NexusGroup instances which represent top level groups.
pub(crate) trait TopGroupBuildable {
    fn new_toplevel(name: &str) -> Self;
}

/// Trait which is implemented in testing only.
/// examine and examine_children allows tests
/// to inspect the inner workings.
/// Not used in production.
#[cfg(test)]
pub(crate) trait Examine<R, C> {
    fn examine<F, T>(&self, f: F) -> T
    where
        F: Fn(&C) -> T;

    fn examine_children<F, T>(&self, f: F) -> T
    where
        F: Fn(&[R]) -> T;
}
