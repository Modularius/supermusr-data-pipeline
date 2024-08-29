use std::{marker::PhantomData, rc::Rc, sync::Mutex};

use builder::NexusAttributeBuilder;
use hdf5::{Attribute, Dataset, H5Type};
use tracing::instrument;

mod builder;
mod underlying;

use super::{
    dataset::AttributeRegister, traits::Buildable, FixedValueOption, MustEnterFixedValue, NoFixedValueNeeded
};

#[derive(strum::Display)]
pub(crate) enum NexusUnits {
    #[strum(to_string = "second")]
    Seconds,
    #[strum(to_string = "us")]
    Microseconds,
    #[strum(to_string = "ns")]
    Nanoseconds,
    #[strum(to_string = "ISO8601")]
    ISO8601,
    #[strum(to_string = "mEv")]
    MegaElectronVolts,
    #[strum(to_string = "uAh")]
    MicroAmpHours,
    #[strum(to_string = "counts")]
    Counts,
}

pub(crate) trait NxAttribute {
    fn create(&mut self, dataset: &Dataset) -> anyhow::Result<()>;
    fn open(&mut self, dataset: &Dataset) -> anyhow::Result<()>;
    fn close(&mut self) -> anyhow::Result<()>;
}

pub(crate) type NexusAttribute<T,C> = Rc<Mutex<UnderlyingNexusAttribute<T, C>>>;
pub(crate) type RcNexusAttributeVar<T> = Rc<Mutex<UnderlyingNexusAttribute<T, NoFixedValueNeeded>>>;
pub(crate) type RcNexusAttributeFixed<T> = Rc<Mutex<UnderlyingNexusAttribute<T, MustEnterFixedValue>>>;

/// NexusAttribute
pub(crate) struct UnderlyingNexusAttribute<T: H5Type, F: FixedValueOption> {
    name: String,
    fixed_value: Option<T>,
    attribute: Option<Attribute>,
    phantom: PhantomData<F>,
}

impl<T: H5Type, F: FixedValueOption> Buildable<T> for NexusAttribute<T, F> {
    fn begin() -> NexusAttributeBuilder<T, F, F> {
        NexusAttributeBuilder::<T, F, F> {
            fixed_value: None,
            phantom: PhantomData,
        }
    }
    
    type BuilderType = NexusAttributeBuilder<T,F,F>;
}

impl<T: H5Type + Clone, F: FixedValueOption> NxAttribute for NexusAttribute<T, F> {
    #[instrument(skip_all, level = "debug", fields(name = self.name), err(level = "error"))]
    fn create(&mut self, dataset: &Dataset) -> anyhow::Result<()> {
        if self.attribute.is_some() {
            Err(anyhow::anyhow!("{} attribute already open", self.name))
        } else {
            let attribute = dataset.new_attr::<T>().create(self.name.as_str())?;
            if let Some(fixed_value) = &self.fixed_value {
                attribute.write_scalar(fixed_value)?
            }

            self.attribute = Some(attribute);
            Ok(())
        }
    }

    #[instrument(skip_all, level = "debug", fields(name = self.name), err(level = "error"))]
    fn open(&mut self, dataset: &Dataset) -> anyhow::Result<()> {
        if self.attribute.is_some() {
            Err(anyhow::anyhow!("{} attribute already open", self.name))
        } else {
            self.attribute = Some(dataset.attr(&self.name).expect("Attribute Exists"));
            Ok(())
        }
    }

    #[instrument(skip_all, level = "debug", fields(name = self.name), err(level = "error"))]
    fn close(&mut self) -> anyhow::Result<()> {
        if self.attribute.is_none() {
            Err(anyhow::anyhow!("{} attribute already closed", self.name))
        } else {
            self.attribute = None;
            Ok(())
        }
    }
}
