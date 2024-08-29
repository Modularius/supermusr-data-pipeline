use std::{marker::PhantomData, rc::Rc, sync::Mutex};

use hdf5::{Attribute, Dataset, H5Type};
use tracing::instrument;

/// NexusAttributeBuilder
#[derive(Clone)]
pub(crate) struct NexusAttributeBuilder<T: H5Type, F0: FixedValueOption, F: FixedValueOption> {
    pub(super) fixed_value: Option<T>,
    pub(super) phantom: PhantomData<(F0, F)>,
}

impl<T: H5Type, F0: FixedValueOption> NexusAttributeBuilder<T, F0, MustEnterFixedValue> {
    pub(crate) fn fixed_value(self, value: T) -> NexusAttributeBuilder<T, F0, NoFixedValueNeeded> {
        NexusAttributeBuilder::<T, F0, NoFixedValueNeeded> {
            fixed_value: Some(value),
            phantom: PhantomData,
        }
    }
}

impl<T: H5Type + Clone, F0: FixedValueOption + 'static>
    NexusAttributeBuilder<T, F0, NoFixedValueNeeded>
{
    #[instrument(skip_all)]
    pub(crate) fn finish(
        self,
        name: &str,
        register: AttributeRegister,
    ) -> Rc<Mutex<NexusAttribute<T, F0>>> {
        let rc = Rc::new(Mutex::new(NexusAttribute {
            name: name.to_owned(),
            fixed_value: self.fixed_value,
            attribute: None,
            phantom: PhantomData::<F0>,
        }));
        register.lock().expect("Lock Exists").push(rc.clone());
        rc
    }
}
