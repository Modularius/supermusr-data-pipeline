use std::{marker::PhantomData, rc::Rc, sync::Mutex};

use hdf5::{
    types::{TypeDescriptor, VarLenAscii},
    Attribute, Dataset, H5Type, Location,
};

use super::{
    dataset::RcAttributeRegister, FixedValueOption, MustEnterFixedValue, NoFixedValueNeeded,
};

#[derive(strum::Display)]
pub(crate) enum NexusUnits {
    #[strum(to_string = "second")]
    Second,
    #[strum(to_string = "us")]
    Microsecond,
    #[strum(to_string = "ns")]
    Nanoseconds,
    #[strum(to_string = "ISO8601")]
    ISO8601,
    #[strum(to_string = "mEv")]
    Mev,
    #[strum(to_string = "uAh")]
    MicroAmpHours,
    #[strum(to_string = "counts")]
    Counts,
}

pub(crate) trait NxAttribute {
    fn create(&mut self, dataset: &Dataset);
    fn open(&mut self, dataset: &Dataset);
    fn close(&mut self);
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
    fn create(&mut self, dataset: &Dataset) {
        if let Some(fixed_value) = &self.fixed_value {
            self.attribute = Some(
                dataset
                    .new_attr_builder()
                    .with_data(&[fixed_value.clone()])
                    .create(self.name.as_str())
                    .expect("Attribute Creates"),
            );
        }
    }

    fn open(&mut self, dataset: &Dataset) {
        self.attribute = Some(dataset.attr(&self.name).expect("Attribute Exists"));
    }

    fn close(&mut self) {
        self.attribute = None;
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
