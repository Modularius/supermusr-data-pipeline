use hdf5::{Dataset, Group};
use supermusr_streaming_types::ecs_pl72_run_start_generated::RunStart;

use crate::{
    elements::{
        attribute::{NexusAttribute, NexusAttributeMut},
        dataset::{NexusDataset, NexusDatasetMut, NexusDatasetResize},
        traits::{
            NexusAppendableDataHolder, NexusDataHolderScalarMutable, NexusDataHolderStringMutable,
            NexusDatasetDef, NexusDatasetDefUnitsOnly, NexusGroupDef, NexusHandleMessage,
        },
        NexusUnits,
    },
    error::NexusPushError,
    nexus::NexusSettings,
    schematic::{nexus_class, H5String},
};

#[derive(Default, Clone)]
struct SourceFrequency;

impl NexusDatasetDefUnitsOnly for SourceFrequency {
    const UNITS: NexusUnits = NexusUnits::Hertz;
}

#[derive(Clone)]
struct SourceFramePattern {
    /// repetition length of frame pattern in terms of frames to target
    rep_len: NexusAttributeMut<i32>,
    /// period of repetition of frame pattern, e.g. 100ms at ISIS target 1, with TS2
    period: NexusAttributeMut<f64>,
    /// number of pulses for each accelerator frame, e.g. ‘2’ at ISIS
    pulses_per_frame: NexusAttributeMut<f64>,
}

impl<'a> NexusHandleMessage<RunStart<'a>, Dataset> for SourceFramePattern {
    fn handle_message(
        &mut self,
        _message: &RunStart<'a>,
        parent: &Dataset,
    ) -> Result<(), NexusPushError> {
        self.rep_len.write_scalar(parent, 1)?;
        self.period.write_scalar(parent, 80.0)?;
        self.pulses_per_frame.write_scalar(parent, 1.0)?;
        Ok(())
    }
}

impl NexusDatasetDef for SourceFramePattern {
    const UNITS: Option<NexusUnits> = Some(NexusUnits::Milliseconds);

    fn new() -> Self {
        Self {
            rep_len: NexusAttribute::new_with_default("rep_len"),
            period: NexusAttribute::new_with_default("period"),
            pulses_per_frame: NexusAttribute::new_with_default("pulses_per_frame"),
        }
    }
}

#[derive(Default, Clone)]
struct SourceEnergy;

impl NexusDatasetDefUnitsOnly for SourceEnergy {
    const UNITS: NexusUnits = NexusUnits::MegaElectronVolts;
}

#[derive(Default, Clone)]
struct SourceCurrent;

impl NexusDatasetDefUnitsOnly for SourceCurrent {
    const UNITS: NexusUnits = NexusUnits::MicroAmps;
}

#[derive(Default, Clone)]
struct SourcePulseWidth;

impl NexusDatasetDefUnitsOnly for SourcePulseWidth {
    const UNITS: NexusUnits = NexusUnits::Nanoseconds; // Todo (is this correct?)
}

#[derive(Default, Clone)]
struct TargetThickness;

impl NexusDatasetDefUnitsOnly for TargetThickness {
    const UNITS: NexusUnits = NexusUnits::Millimeters;
}

#[derive(Default, Clone)]
struct Momentum;

impl NexusDatasetDefUnitsOnly for Momentum {
    const UNITS: NexusUnits = NexusUnits::MegaElectronVoltsOverC;
}

#[derive(Default, Clone)]
struct MuonEnergy;

impl NexusDatasetDefUnitsOnly for MuonEnergy {
    const UNITS: NexusUnits = NexusUnits::ElectronVolts;
}

#[derive(Clone)]
struct MuonPulsePattern {
    /// repetition length of frame pattern in terms of frames to target
    rep_len: NexusAttributeMut<i32>,
    /// period of repetition of frame pattern, e.g. 100ms at ISIS target 1, with TS2
    period: NexusAttributeMut<f64>,
}

impl NexusDatasetDef for MuonPulsePattern {
    const UNITS: Option<NexusUnits> = Some(NexusUnits::Milliseconds);

    fn new() -> Self {
        Self {
            rep_len: NexusAttribute::new_with_default("rep_len"),
            period: NexusAttribute::new_with_default("period"),
        }
    }
}

impl<'a> NexusHandleMessage<RunStart<'a>, Dataset> for MuonPulsePattern {
    fn handle_message(
        &mut self,
        _message: &RunStart<'a>,
        parent: &Dataset,
    ) -> Result<(), NexusPushError> {
        self.rep_len.write_scalar(parent, 1)?;
        self.period.write_scalar(parent, 80.0)?;
        Ok(())
    }
}

#[derive(Default, Clone)]
struct MuonPulseWidth;

impl NexusDatasetDefUnitsOnly for MuonPulseWidth {
    const UNITS: NexusUnits = NexusUnits::Nanoseconds;
}

#[derive(Default, Clone)]
struct MuonPulseSeparation;

impl NexusDatasetDefUnitsOnly for MuonPulseSeparation {
    const UNITS: NexusUnits = NexusUnits::Nanoseconds;
}

pub(super) struct Source {
    /// facility name
    name: NexusDatasetMut<H5String>,
    /// `pulsed muon source` | `low energy muon source`
    source_type: NexusDatasetMut<H5String>,
    /// `positive muons` | `negative muons`
    probe: NexusDatasetMut<H5String>,
    /// accelerator frequency, note that some framesmay be 'missing' at target
    source_frequency: NexusDatasetMut<f64, SourceFrequency>,
    // /// log of source frequency during run
    //source_frequency_log: NexusGroup<Log>,
    /// frame pattern: `1` frame to target, `0` frame missing at `frequency`
    /// e.g. ISIS target 1 with TS2: `1,1,1,1,0`,
    /// with a `rep_len` of `5` and `period` 100ms
    source_frame_pattern: NexusDatasetResize<u32, SourceFramePattern>,
    /// source energy at target
    source_energy: NexusDatasetMut<f64, SourceEnergy>,
    /// source current - this could be an average source current for the run, or logged values
    source_current: NexusDatasetMut<f64, SourceCurrent>,
    // /// log of source current during run
    //source_current_log: NexusGroup<Log>,
    /// source pulse width
    source_pulse_width: NexusDatasetMut<f64, SourcePulseWidth>,
    /// e.g. ‘carbon’
    target_material: NexusDatasetMut<H5String>,
    /// thickness of target
    target_thickness: NexusDatasetMut<f64, TargetThickness>,
    /// pion momentum
    pion_momentum: NexusDatasetMut<f64, Momentum>,
    /// muon energy
    muon_energy: NexusDatasetMut<f64, MuonEnergy>,
    /// muon momentum
    muon_momentum: NexusDatasetMut<f64, Momentum>,
    /// pulse pattern – `n` number of pulses to instrument each frame, e.g. ISIS target 1 with TS2: `2,2,2,2,0`, with a `rep_len` of `5` and `period` 100ms, assuming no muon kicker
    muon_pulse_pattern: NexusDatasetResize<f64, MuonPulsePattern>,
    /// pulse width for each pulse in frame, e.g. 80ns at ISIS
    muon_pulse_width: NexusDatasetResize<f64, MuonPulseWidth>,
    /// separation of consecutive pulses in frame, e.g. 300ns at ISIS
    muon_pulse_separation: NexusDatasetResize<f64, MuonPulseSeparation>,
    /// source related messages or announcements, e.g. MCR messages
    notes: NexusDatasetMut<H5String>,
}

impl NexusGroupDef for Source {
    const CLASS_NAME: &'static str = nexus_class::SOURCE;
    type Settings = NexusSettings;

    fn new(settings: &NexusSettings) -> Self {
        Self {
            name: NexusDataset::new_with_default("name"),
            source_type: NexusDataset::new_with_default("source_type"),
            probe: NexusDataset::new_with_default("probe"),
            source_frequency: NexusDataset::new_with_default("source_frequency"),
            //source_frequency_log: NexusGroup::new("source_frequency_log", settings),
            source_frame_pattern: NexusDataset::new_appendable_with_default(
                "source_frame_pattern",
                settings.dimensional_chunk_size,
            ),
            source_energy: NexusDataset::new_with_default("source_energy"),
            source_current: NexusDataset::new_with_default("tarsource_currentget_thickness"),
            //source_current_log: NexusGroup::new("source_current_log", settings),
            source_pulse_width: NexusDataset::new_with_default("source_pulse_width"),
            target_material: NexusDataset::new_with_default("target_material"),
            target_thickness: NexusDataset::new_with_default("target_thickness"),
            pion_momentum: NexusDataset::new_with_default("pion_momentum"),
            muon_energy: NexusDataset::new_with_default("muon_energy"),
            muon_momentum: NexusDataset::new_with_default("muon_momentum"),
            muon_pulse_pattern: NexusDataset::new_appendable_with_default(
                "muon_pulse_pattern",
                settings.dimensional_chunk_size,
            ),
            muon_pulse_width: NexusDataset::new_appendable_with_default(
                "muon_pulse_width",
                settings.dimensional_chunk_size,
            ),
            muon_pulse_separation: NexusDataset::new_appendable_with_default(
                "muon_pulse_separation",
                settings.dimensional_chunk_size,
            ),
            notes: NexusDataset::new_with_default("notes"),
        }
    }
}

impl<'a> NexusHandleMessage<RunStart<'a>> for Source {
    fn handle_message(
        &mut self,
        _message: &RunStart<'a>,
        parent: &Group,
    ) -> Result<(), NexusPushError> {
        self.name
            .write_string(parent, "Isis Neutron and Muon Source")?;
        self.source_type
            .write_string(parent, "Isis Neutron and Muon Source")?;
        self.probe.write_string(parent, "Positive Muons")?;
        self.source_frequency.write_scalar(parent, 50.0)?;
        self.source_frame_pattern.append(parent, &[0])?;
        self.source_energy.write_scalar(parent, 0.0)?;
        self.source_current.write_scalar(parent, 0.0)?;
        //self.source_current_log.push_message(message, parent)?;
        self.source_pulse_width.write_scalar(parent, 0.0)?;
        self.target_material.write_string(parent, "carbon")?;
        self.target_thickness.write_scalar(parent, 0.0)?;
        self.pion_momentum.write_scalar(parent, 1.0)?;
        self.muon_energy.write_scalar(parent, 1.0)?;
        self.muon_momentum.write_scalar(parent, 1.0)?;
        self.muon_pulse_pattern.append(parent, &[1.0])?;
        self.muon_pulse_width.append(parent, &[1.0])?;
        self.muon_pulse_separation.append(parent, &[1.0])?;
        self.notes.write_string(parent, "")?;
        Ok(())
    }
}
