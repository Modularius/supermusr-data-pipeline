use std::{any::Any, marker::PhantomData, rc::Rc, sync::Mutex};

use hdf5::{Dataset, Group, H5Type};

use super::{attribute::NexusAttribute, group::RcDatasetRegister};

pub(crate) trait FixedValueOption : Clone {}

#[derive(Clone)]
pub(crate) struct MustEnterFixedValue {}
impl FixedValueOption for MustEnterFixedValue {}

#[derive(Clone)]
pub(crate) struct NoFixedValueNeeded {}
impl FixedValueOption for NoFixedValueNeeded {}

pub(crate) trait AttributesOption : Clone {}

#[derive(Clone)]
pub(crate) struct MustEnterAttributes<const N: usize> {}
impl<const N: usize> AttributesOption for MustEnterAttributes<N> {}

#[derive(Clone)]
pub(crate) struct NoAttributesNeeded {}
impl AttributesOption for NoAttributesNeeded {}

pub(crate) type RcNexusDatasetVar<T, A = NoAttributesNeeded> = Rc<Mutex<NexusDataset<T,A,NoFixedValueNeeded>>>;
pub(crate) type RcNexusDatasetFixed<T, A = NoAttributesNeeded> = Rc<Mutex<NexusDataset<T,A,MustEnterFixedValue>>>;

#[derive(Default)]
pub(crate) struct NexusDataset<
    T: H5Type,
    A: AttributesOption = NoAttributesNeeded,
    F: FixedValueOption = NoFixedValueNeeded,
> {
    name: String,
    fixed_value: Option<T>,
    attributes: Option<Vec<NexusAttribute>>,
    dataset: Option<Dataset>,
    phantom: PhantomData<(F, A)>,
    //ref_attributes: Vec<&'a dyn NxAttribute>,
}

impl<'a, T: H5Type + Clone, A: AttributesOption, F: FixedValueOption> NexusDataset<T, A, F> {
    pub(crate) fn begin() -> NexusDatasetBuilder<T, A, F> {
        NexusDatasetBuilder::<T, A, F> {
            fixed_value: None,
            attributes: None,
            phantom: PhantomData,
        }
    }

    pub(crate) fn validate_self(&self) -> bool {
        if let Some(dataset) = &self.dataset {
            dataset.name().cmp(&self.name).is_eq()
                && dataset
                    .attr_names()
                    .expect("Attribute names exist")
                    .iter()
                    .map(|name| {
                        let attr = dataset.attr(name).expect("Attribute exists");
                        if let Some(attributes) = &self.attributes {
                            if let Some(na) = attributes
                                .iter()
                                .find(|&na| na.get_name().cmp(name).is_eq())
                            {
                                na.get_type().type_id() == attr.type_id()
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    })
                    .find(|v| *v == false)
                    .is_none()
        } else {
            false
        }
    }
}

#[derive(Clone)]
pub(crate) struct NexusDatasetBuilder<T: H5Type, A: AttributesOption, F: FixedValueOption> {
    fixed_value: Option<T>,
    attributes: Option<Vec<NexusAttribute>>,
    phantom: PhantomData<(A, F)>,
}

impl<T: H5Type, F: FixedValueOption, const N: usize>
    NexusDatasetBuilder<T, MustEnterAttributes<N>, F>
{
    pub(crate) fn attributes(
        self,
        attributes: [NexusAttribute; N],
    ) -> NexusDatasetBuilder<T, NoAttributesNeeded, F> {
        NexusDatasetBuilder::<T, NoAttributesNeeded, F> {
            fixed_value: None,
            attributes: Some(attributes.to_vec()),
            phantom: PhantomData,
        }
    }
}

impl<T: H5Type> NexusDatasetBuilder<T, NoAttributesNeeded, MustEnterFixedValue> {
    pub(crate) fn fixed_value(
        self,
        value: T,
    ) -> NexusDatasetBuilder<T, NoAttributesNeeded, NoFixedValueNeeded> {
        NexusDatasetBuilder::<T, NoAttributesNeeded, NoFixedValueNeeded> {
            fixed_value: Some(value),
            attributes: self.attributes,
            phantom: PhantomData,
        }
    }
}

impl<T: H5Type + Clone> NexusDatasetBuilder<T, NoAttributesNeeded, NoFixedValueNeeded> {
    pub(crate) fn finish<A: AttributesOption + 'static, F: FixedValueOption + 'static>(
        self,
        name: &str,
        register: RcDatasetRegister
    ) -> Rc<Mutex<NexusDataset<T, A, F>>> {
        let rc = Rc::new(Mutex::new(NexusDataset {
            name: name.to_owned(),
            fixed_value: self.fixed_value,
            attributes: self.attributes,
            dataset: None,
            phantom: PhantomData::<(F, A)>,
        }));
        register.lock().expect("Lock Exists").push(rc.clone());
        rc
    }
}


pub(crate) trait NxDataset {
    fn create(&mut self, parent: &Group);
    fn open(&mut self, parent: &Group);
    fn close(&mut self);
}

impl<T: H5Type + Clone, A: AttributesOption, F: FixedValueOption> NxDataset for NexusDataset<T, A, F> {
    fn create(&mut self, parent: &Group) {
        if let Some(fixed_value) = &self.fixed_value {
            match parent
                .new_dataset_builder()
                .with_data(&[fixed_value.clone()])
                .create(self.name.as_str())
            {
                Ok(dataset) => {
                    self.dataset = Some(dataset);
                }
                Err(e) => panic!("{e}"),
            }
        }
    }

    fn open(&mut self, parent: &Group) {
        if self.dataset.is_some() {
            panic!("{} group already open", self.name)
        } else {
            match parent.dataset(&self.name) {
                Ok(dataset) => {
                    self.dataset = Some(dataset);
                }
                Err(e) => panic!("{e}"),
            }
        }
    }

    fn close(&mut self) {
        if self.dataset.is_none() {
            panic!("{} dataset already closed", self.name)
        } else {
            self.dataset = None
        }
    }
}
