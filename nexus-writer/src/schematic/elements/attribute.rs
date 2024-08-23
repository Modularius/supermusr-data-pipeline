use hdf5::{types::{TypeDescriptor, VarLenAscii}, Attribute, Dataset, H5Type, Location};

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
    fn create(dataset: &Dataset);
    fn open(dataset: &Dataset);
    fn close();
}


pub(crate) struct ExperimentalNexusAttribute<T : H5Type> {
    name: String,
    fixed_value: Option<T>,
    attribute: Option<Attribute>,
}

impl<T : H5Type> ExperimentalNexusAttribute<T> {
    pub(crate) fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            fixed_value: None,
            attribute: None
        }
    }
    
    pub(crate) fn new_fixed(name: &str,) -> Self {
        Self {
            name: name.to_string(),
            fixed_value: None,
            attribute: None
        }
    }
}

impl ExperimentalNexusAttribute<VarLenAscii> {
    pub(crate) fn units(units: NexusUnits) -> Self {
        Self {
            name: "units".to_string(),
            fixed_value: Some(VarLenAscii::from_ascii(&units.to_string()).expect("")),
            attribute: None,
        }
    }
}

#[derive(Clone)]
pub(crate) struct NexusAttribute {
    name: String,
    type_descriptor: TypeDescriptor,
    fixed_value: Option<String>,
    attribute: Option<Attribute>,
}

impl NexusAttribute {
    pub(crate) fn new(name: &str, type_descriptor: TypeDescriptor) -> Self {
        Self {
            name: name.to_owned(),
            type_descriptor,
            fixed_value: None,
            attribute: None,
        }
    }

    pub(crate) fn units(units: NexusUnits) -> Self {
        Self {
            name: "Units".to_owned(),
            type_descriptor: TypeDescriptor::VarLenAscii,
            fixed_value: Some(units.to_string()),
            attribute: None,
        }
    }

    pub(crate) fn get_name(&self) -> &str {
        &self.name
    }

    pub(crate) fn get_type(&self) -> &TypeDescriptor {
        &self.type_descriptor
    }

    pub(crate) fn create(&mut self, location: &Location) {
        if let Some(fixed_value) = &self.fixed_value {
            self.attribute = Some(
                location
                    .new_attr_builder()
                    .with_data_as(fixed_value, &self.type_descriptor)
                    .create(self.name.as_str())
                    .expect("Attribute Creates"),
            );
        }
    }

    pub(crate) fn open(&mut self, location: &Location) {
        self.attribute = Some(location.attr(&self.name).expect("Attribute Exists"));
    }

    pub(crate) fn close(&mut self) {
        self.attribute = None;
    }
}
