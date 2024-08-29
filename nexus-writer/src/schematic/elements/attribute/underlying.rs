use std::{marker::PhantomData, rc::Rc, sync::Mutex};

use builder::NexusAttributeBuilder;
use hdf5::{Attribute, Dataset, H5Type};
use tracing::instrument;
