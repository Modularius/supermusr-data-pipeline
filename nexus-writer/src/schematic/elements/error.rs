use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum NexusError {
    #[error("HDF5 Error: {0}")]
    HDF5(#[from] HDF5Error),
    #[error("Cannot Create HDF5 Object: {0}")]
    Create(#[from] CreationError),
    #[error("Cannot Open HDF5 Object: {0}")]
    Open(#[from] OpeningError),
    #[error("Cannot Close HDF5 Object: {0}")]
    Close(#[from] ClosingError),
}

#[derive(Debug, Error)]
pub(crate) enum CreationError {
    #[error("HDF5 Error: {0}")]
    HDF5(#[from] HDF5Error),
    #[error("Already Open")]
    AlreadyOpen,
}

#[derive(Debug, Error)]
pub(crate) enum OpeningError {
    #[error("HDF5 Error: {0}")]
    HDF5(#[from] HDF5Error),
    #[error("Already Open")]
    AlreadyOpen,
}

#[derive(Debug, Error)]
pub(crate) enum ClosingError {
    #[error("HDF5 Error: {0}")]
    HDF5(#[from] HDF5Error),
    #[error("Already Closed")]
    AlreadyClosed,
}

#[derive(Debug, Error)]
pub(crate) enum HDF5Error {
    #[error("HDF5 String Error")]
    String(hdf5::types::StringError),
    #[error("HDF5 Error")]
    General(hdf5::Error),
}
