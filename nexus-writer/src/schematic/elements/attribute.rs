use std::{marker::PhantomData, rc::Rc, sync::Mutex};

use hdf5::{
    types::{TypeDescriptor, VarLenAscii},
    Attribute, Dataset, H5Type, Location,
};
use tracing::{info, instrument};

use super::{
    dataset::RcAttributeRegister, FixedValueOption, MustEnterFixedValue, NoFixedValueNeeded,
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

pub(crate) type RcNexusAttributeVar<T> = Rc<Mutex<NexusAttribute<T, NoFixedValueNeeded>>>;
pub(crate) type RcNexusAttributeFixed<T> = Rc<Mutex<NexusAttribute<T, MustEnterFixedValue>>>;

/// NexusAttribute
pub(crate) struct NexusAttribute<T: H5Type, F: FixedValueOption> {
    name: String,
    fixed_value: Option<T>,
    attribute: Option<Attribute>,
    phantom: PhantomData<F>,
}

impl<T: H5Type, F: FixedValueOption> NexusAttribute<T, F> {
    pub(crate) fn begin() -> NexusAttributeBuilder<T, F> {
        NexusAttributeBuilder::<T, F> {
            fixed_value: None,
            phantom: PhantomData,
        }
    }
}

impl<T: H5Type + Clone, F: FixedValueOption> NxAttribute for NexusAttribute<T, F> {
    #[instrument(skip_all, level = "debug", fields(name = self.name), err(level = "error"))]
    fn create(&mut self, dataset: &Dataset) -> anyhow::Result<()> {
        if self.attribute.is_some() {
            Err(anyhow::anyhow!("{} attribute already open", self.name))
        } else {
            let mut attribute = dataset
                .new_attr::<T>()
                .create(self.name.as_str())?;
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
        if self.attribute.is_some() {
            Err(anyhow::anyhow!("{} attribute already open", self.name))
        } else {
            self.attribute = None;
            Ok(())
        }
    }
}

/// NexusAttributeBuilder
#[derive(Clone)]
pub(crate) struct NexusAttributeBuilder<T: H5Type, F: FixedValueOption> {
    fixed_value: Option<T>,
    phantom: PhantomData<F>,
}

impl<T: H5Type> NexusAttributeBuilder<T, MustEnterFixedValue> {
    pub(crate) fn fixed_value(self, value: T) -> NexusAttributeBuilder<T, NoFixedValueNeeded> {
        NexusAttributeBuilder::<T, NoFixedValueNeeded> {
            fixed_value: Some(value),
            phantom: PhantomData,
        }
    }
}

impl<T: H5Type + Clone> NexusAttributeBuilder<T, NoFixedValueNeeded> {
    #[instrument(skip_all)]
    pub(crate) fn finish<F: FixedValueOption + 'static>(
        self,
        name: &str,
        register: RcAttributeRegister,
    ) -> Rc<Mutex<NexusAttribute<T, F>>> {
        let attributes = RcAttributeRegister::new(Vec::new().into());
        let rc = Rc::new(Mutex::new(NexusAttribute {
            name: name.to_owned(),
            fixed_value: self.fixed_value,
            attribute: None,
            phantom: PhantomData::<F>,
        }));
        register.lock().expect("Lock Exists").push(rc.clone());
        rc
    }
}
