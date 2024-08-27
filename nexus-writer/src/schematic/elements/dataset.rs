use std::{any::Any, marker::PhantomData, rc::Rc, sync::Mutex};

use hdf5::{types::VarLenAscii, Dataset, Group, H5Type};

use super::{
    attribute::{self, NexusAttribute, NexusUnits, NxAttribute},
    group::RcGroupContentRegister,
    FixedValueOption, MustEnterFixedValue, NoFixedValueNeeded, NxLivesInGroup,
};

/// NxDataset Trait

pub(crate) type RcNexusDatasetVar<T, D = ()> = Rc<Mutex<NexusDataset<T, D, false>>>;
pub(crate) type RcNexusDatasetFixed<T, D = ()> = Rc<Mutex<NexusDataset<T, D, true>>>;

pub(crate) type RcAttributeRegister = Rc<Mutex<Vec<Rc<Mutex<dyn NxAttribute>>>>>;

pub(crate) trait NxContainerAttributes: Sized {
    const UNITS: Option<NexusUnits> = None;

    fn new(attribute_register: RcAttributeRegister) -> Self;
}

impl NxContainerAttributes for () {
    fn new(_attribute_register: RcAttributeRegister) -> Self {
        ()
    }
}

#[derive(Default)]
pub(crate) struct NexusDataset<T: H5Type, D: NxContainerAttributes = (), const F: bool = false> {
    name: String,
    fixed_value: Option<T>,
    attributes: RcAttributeRegister,
    class: D,
    dataset: Option<Dataset>
}

impl<T, D, const F: bool> NexusDataset<T, D, F>
where
    T: H5Type + Clone,
    D: NxContainerAttributes,
{
    pub(crate) fn begin() -> NexusDatasetBuilder<T, D, F> {
        NexusDatasetBuilder {
            fixed_value: None,
            phantom: PhantomData,
        }
    }
    /*
    pub(crate) fn validate_self(&self) -> bool {
        if let Some(dataset) = &self.dataset {
            dataset.name().cmp(&self.name).is_eq()
                && dataset
                    .attr_names()
                    .expect("Attribute names exist")
                    .iter()
                    .map(|name| {
                        //let attr = dataset.attr(name).expect("Attribute exists");
                        /*if let Some(attributes) = &self.attributes {
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
                        }*/
                        true
                    })
                    .find(|v| *v == false)
                    .is_none()
        } else {
            false
        }
    } */
}

impl<T, D, const F: bool> NxLivesInGroup for NexusDataset<T, D, F>
where
    T: H5Type + Clone,
    D: NxContainerAttributes,
{
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

/// NexusDatasetBuilder
#[derive(Clone)]
pub(crate) struct NexusDatasetBuilder<T, D, const F: bool>
where
    T: H5Type,
    D: NxContainerAttributes,
{
    fixed_value: Option<T>,
    phantom: PhantomData<D>,
}

impl<T, D> NexusDatasetBuilder<T, D, true>
where
    T: H5Type,
    D: NxContainerAttributes,
{
    pub(crate) fn fixed_value(self, value: T) -> NexusDatasetBuilder<T, D, false> {
        NexusDatasetBuilder::<T, D, false> {
            fixed_value: Some(value),
            phantom: PhantomData,
        }
    }
}

impl<T, D> NexusDatasetBuilder<T, D, false>
where
    T: H5Type + Clone,
    D: NxContainerAttributes + Clone + 'static,
{
    pub(crate) fn finish<const F: bool>(
        self,
        name: &str,
        parent_content_register: RcGroupContentRegister,
    ) -> Rc<Mutex<NexusDataset<T, D, F>>> {
        let attributes = RcAttributeRegister::new(Mutex::new(Vec::new()));
        
        if let Some(units) = D::UNITS {
            NexusAttribute::begin()
                .fixed_value(
                    VarLenAscii::from_ascii(&units.to_string()).expect(""),
                )
                .finish::<MustEnterFixedValue>("units", attributes.clone());
        }

        let rc = Rc::new(Mutex::new(NexusDataset {
            name: name.to_owned(),
            fixed_value: self.fixed_value,
            class: D::new(attributes.clone()),
            attributes,
            dataset: None
        }));
        parent_content_register.lock().expect("Lock Exists").push(rc.clone());
        rc
    }
}
