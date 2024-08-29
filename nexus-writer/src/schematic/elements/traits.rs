use hdf5::{Group, H5Type};

pub(crate) trait Class<T, O>: Clone {
    fn create(&self, parent: &Group, name: &str) -> Result<O, anyhow::Error>;
}

#[derive(Clone)]
pub(crate) struct Constant<T: H5Type>(pub(crate) T);

#[derive(Clone)]
pub(crate) struct Resizable {
    pub(crate) initial_size: usize,
    pub(crate) chunk_size: usize,
}

pub(crate) mod tags {
    use hdf5::H5Type;

    pub(crate) trait Tag<T: H5Type,O>: Clone {
        type ClassType: super::Class<T,O>;
    }

    #[derive(Clone)]
    pub(crate) struct Constant;

    #[derive(Clone)]
    pub(crate) struct Resizable;
}

//  Traits for Dataset and Attribute
pub(crate) trait Buildable<T, D, C>
where
    T: H5Type + Clone,
{
    type BuilderType;

    fn begin(name: &str) -> Self::BuilderType;
}

pub(crate) trait CanWriteScalar {
    type Type: H5Type;
    fn write_scalar(&self, value: Self::Type) -> Result<(), hdf5::Error>;
}

pub(crate) trait CanAppend {
    type Type: H5Type;
    fn append(&self, value: &[Self::Type]) -> Result<(), hdf5::Error>;
}
