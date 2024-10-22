use chrono::{DateTime, Utc};
use environment::Environment;
use geometry::Geometry;
//use environment::Environment;
//use geometry::Geometry;
use hdf5::{Dataset, Group};
use supermusr_streaming_types::ecs_pl72_run_start_generated::RunStart;

use crate::{
    elements::{
        attribute::{NexusAttribute, NexusAttributeMut},
        dataset::{NexusDataset, NexusDatasetMut, NexusDatasetResize},
        group::NexusGroup,
        traits::{
            NexusAppendableDataHolder, NexusDataHolderScalarMutable, NexusDataHolderStringMutable,
            NexusDatasetDef, NexusDatasetDefUnitsOnly, NexusGroupDef, NexusHandleMessage,
            NexusPushMessage,
        },
        NexusUnits,
    },
    error::NexusPushError,
    nexus::NexusSettings,
    schematic::{nexus_class, H5String},
};

mod environment;
mod geometry;

/*
    Dataset: Thickness
*/

#[derive(Default, Clone)]
struct Thickness;

impl NexusDatasetDefUnitsOnly for Thickness {
    const UNITS: NexusUnits = NexusUnits::Millimeters;
}

/*
    Dataset: Mass
*/

#[derive(Default, Clone)]
struct Mass;

impl NexusDatasetDefUnitsOnly for Mass {
    const UNITS: NexusUnits = NexusUnits::Milligrams;
}

/*
    Dataset: Density
*/

#[derive(Default, Clone)]
struct Density;

impl NexusDatasetDefUnitsOnly for Density {
    const UNITS: NexusUnits = NexusUnits::MilligramsPerCm3;
}

/*
    Dataset: Temperature
*/

#[derive(Clone)]
struct Temperature {
    /// function of temperature (`control` | `sample`)
    role: NexusAttributeMut<H5String>,
    /// `nominal` | `derived`
    value: NexusAttributeMut<H5String>,
}

impl NexusDatasetDef for Temperature {
    const UNITS: Option<NexusUnits> = Some(NexusUnits::Kelvin);

    fn new() -> Self {
        Self {
            role: NexusAttribute::new_with_default("role"),
            value: NexusAttribute::new_with_default("value"),
        }
    }
}

impl<'a> NexusHandleMessage<RunStart<'a>, Dataset> for Temperature {
    fn handle_message(
        &mut self,
        _message: &RunStart<'a>,
        parent: &Dataset,
    ) -> Result<(), NexusPushError> {
        self.role.write_string(parent, "sample")?;
        self.value.write_string(parent, "nominal")?;
        Ok(())
    }
}

/*
    Dataset: MagneticField
*/

#[derive(Clone)]
struct MagneticField {
    /// field status (`active` | `inactive`)
    role: NexusAttributeMut<H5String>,
    /// field direction (`x` | `y` | `z`)
    direction: NexusAttributeMut<H5String>,
}

impl NexusDatasetDef for MagneticField {
    const UNITS: Option<NexusUnits> = Some(NexusUnits::Gauss);

    fn new() -> Self {
        Self {
            role: NexusAttribute::new_with_default("role"),
            direction: NexusAttribute::new_with_default("value"),
        }
    }
}

impl<'a> NexusHandleMessage<RunStart<'a>, Dataset> for MagneticField {
    fn handle_message(
        &mut self,
        _message: &RunStart<'a>,
        parent: &Dataset,
    ) -> Result<(), NexusPushError> {
        self.role.write_string(parent, "inactive")?;
        self.direction.write_string(parent, "x")?;
        Ok(())
    }
}

/*
    Group: Sample
*/

pub(super) struct Sample {
    /// sample name
    name: NexusDatasetMut<H5String>,
    /// element symbols to be arranged in ‘Hill System’ order: C, H, then other elements alphabetically
    chemical_formula: NexusDatasetMut<H5String>,
    /// description of sample
    description: NexusDatasetMut<H5String>,
    /// type of sample
    sample_type: NexusDatasetMut<H5String>,
    /// `atmosphere` | `vacuum``
    situation: NexusDatasetMut<H5String>,
    /// additional information about sample shape
    shape: NexusDatasetMut<H5String>,
    /// ate of preparation of sample
    preparation_date: NexusDatasetMut<H5String>,
    /// description of sample holder
    sample_holder: NexusDatasetMut<H5String>,
    /// flypast (suspended sample) mounting: `0` - No, `1`` - Yes
    flypast: NexusDatasetMut<H5String>,
    /// sample size
    geometry: NexusGroup<Geometry>,
    /// name of each sample componenet
    sample_component: NexusDatasetResize<H5String>,
    /// sample thickness, may be multiple components
    thickness: NexusDatasetResize<u32, Thickness>,
    /// sample mass, may be multiple components
    mass: NexusDatasetResize<u32, Mass>,
    /// sample density, may be multiple components
    density: NexusDatasetResize<u32, Density>,
    /// linked to most representative sample temperature (to help cataloguing programs)
    temperature: NexusDatasetMut<u32>,
    /// linked to most representative magnetic field (to help cataloguing programs)
    magnetic_field: NexusDatasetResize<f64>,
    /// current field operating mode (`LF` | `TF`` | `ZF`)
    magnetic_field_state: NexusDatasetMut<H5String>,
    /// temperature
    temperature_x: Vec<NexusDatasetMut<u32, Temperature>>,
    /// details of associated hardware
    temperature_x_env: Vec<NexusGroup<Environment>>,
    // /// temperature log
    // temperature_x_log: Vec<NexusGroup<Log>>,
    /// magnetic field, may be a vector
    magnetic_field_x: Vec<NexusDatasetResize<u32, MagneticField>>,
    /// details of associated hardware
    magnetic_field_x_env: Vec<NexusGroup<Environment>>,
    // /// log of field values  –extension to NeXus to enable log of vector quantities
    // magnetic_field_x_log: Vec<NexusGroup<Log>>,
}

impl NexusGroupDef for Sample {
    const CLASS_NAME: &'static str = nexus_class::SAMPLE;
    type Settings = NexusSettings;

    fn new(settings: &NexusSettings) -> Self {
        Self {
            name: NexusDataset::new_with_default("name"),
            chemical_formula: NexusDataset::new_with_default("chemical_formula"),
            description: NexusDataset::new_with_default("description"),
            sample_type: NexusDataset::new_with_default("sample_type"),
            situation: NexusDataset::new_with_default("situation"),
            shape: NexusDataset::new_with_default("shape"),
            preparation_date: NexusDataset::new_with_default("preparation_date"),
            sample_holder: NexusDataset::new_with_default("sample_holder"),
            flypast: NexusDataset::new_with_default("flypast"),
            geometry: NexusGroup::new("geometry", settings),
            sample_component: NexusDataset::new_appendable_with_default(
                "sample_component",
                settings.components_chunk_size,
            ),
            thickness: NexusDataset::new_appendable_with_default(
                "thickness",
                settings.components_chunk_size,
            ),
            mass: NexusDataset::new_appendable_with_default("mass", settings.components_chunk_size),
            density: NexusDataset::new_appendable_with_default(
                "density",
                settings.components_chunk_size,
            ),
            temperature: NexusDataset::new_with_default("temperature"),
            magnetic_field: NexusDataset::new_appendable_with_default(
                "magnetic_field",
                settings.dimensional_chunk_size,
            ),
            magnetic_field_state: NexusDataset::new_with_default("magnetic_field_state"),
            temperature_x: vec![NexusDataset::new_with_default("temperature_1")],
            temperature_x_env: vec![NexusGroup::new("temperature_1_env", settings)],
            //temperature_x_log: vec![NexusGroup::new("temperature_1_log", settings)],
            magnetic_field_x: vec![NexusDataset::new_appendable_with_default(
                "magnetic_field_1",
                settings.dimensional_chunk_size,
            )],
            magnetic_field_x_env: vec![NexusGroup::new("magnetic_field_1_env", settings)],
            //magnetic_field_x_log: vec![NexusGroup::new("magnetic_field_1_log", settings)],
        }
    }
}

impl<'a> NexusHandleMessage<RunStart<'a>> for Sample {
    fn handle_message(
        &mut self,
        message: &RunStart<'a>,
        parent: &Group,
    ) -> Result<(), NexusPushError> {
        self.name.write_string(parent, "Sample Name")?;
        self.chemical_formula.write_string(parent, "Sample Name")?;
        self.description.write_string(parent, "Sample Formula")?;
        self.sample_type.write_string(parent, "Sample Type")?;
        self.situation.write_string(parent, "atmosphere")?;
        self.shape.write_string(parent, "Sample Shape")?;
        self.preparation_date
            .write_string(parent, DateTime::<Utc>::default().to_rfc3339().as_str())?;
        self.sample_holder.write_string(parent, "Sample Name")?;
        self.flypast.write_string(parent, "Sample Name")?;
        self.geometry.push_message(message, parent)?;
        self.sample_component
            .append(parent, &["Sample Name".parse().expect("")])?;
        self.thickness.append(parent, &[0])?;
        self.mass.append(parent, &[0])?;
        self.density.append(parent, &[0])?;
        self.temperature.write_scalar(parent, 0)?;
        self.magnetic_field.append(parent, &[0.0, 0.0, 0.0])?;
        self.magnetic_field_state
            .write_string(parent, "Sample Name")?;
        self.temperature_x
            .iter()
            .try_for_each(|temperature| temperature.write_scalar(parent, 0))?;
        self.temperature_x_env
            .iter_mut()
            .try_for_each(|temperature_env| temperature_env.push_message(message, parent))?;
        //self.temperature_x_log.iter_mut().map(|temperature_log|temperature_log.push_message(message, parent)).collect::<Result<_,_>>()?;
        self.magnetic_field_x
            .iter()
            .try_for_each(|temperature| temperature.append(parent, &[0, 0, 0]))?;
        self.magnetic_field_x_env
            .iter_mut()
            .try_for_each(|temperature_env| temperature_env.push_message(message, parent))?;
        //self.magnetic_field_x_log.iter_mut().map(|temperature_log|temperature_log.push_message(message, parent)).collect::<Result<_,_>>()?;
        Ok(())
    }
}
