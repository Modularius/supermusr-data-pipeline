use std::marker::PhantomData;
use hdf5::{Attribute, H5Type};

use super::attribute::NexusAttribute;

#[derive(Default)]
pub(crate) enum NexusUnits {
    #[default]NoUnits,
    Second,
    Microsecond,
    Nanoseconds,
    ISO8601,
    Mev,
    MicroAmpHours,
    UAh,
    Counts
}

pub(crate) struct NumAttribs(usize);

pub(crate) enum NexusValue {
    FixedValue,
    VarValue,
}

#[derive(Default)]
pub(crate) struct NexusDataset<T : H5Type,
    const NUM_ATTRIBS : usize = 0,
    const UNITS: NexusUnits = {NexusUnits::NoUnits},
    const VALUE: NexusValue = {NexusValue::VarValue}>
{
    name: String,
    value : Option<T>,
    attributes: [NexusAttribute; NUM_ATTRIBS],
}

impl<T : H5Type, const NUM_ATTRIBS : usize, const UNITS: NexusUnits, const VALUE: NexusValue> NexusDataset<T,NUM_ATTRIBS,UNITS,{NexusValue::FixedValueValue}> {
    pub(crate) fn new(name: &str, value: T, attributes: [NexusAttribute; NUM_ATTRIBS]) -> Self {
        Self {
            name: name.to_owned(),
            value: Some(value),
            attributes
        }
    }
}

impl<T : H5Type,const NUM_ATTRIBS : usize, const UNITS: NexusUnits> NexusDataset<T, NUM_ATTRIBS, UNITS, {NexusValue::VarValue}> {
    pub(crate) fn new(name: &str, attributes: [NexusAttribute; NUM_ATTRIBS]) -> Self {
        Self {
            name: name.to_owned(),
            attributes,
            ..Default::default()
        }
    }
}
