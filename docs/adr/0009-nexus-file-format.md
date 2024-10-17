# 9. nexus_file_format

Date: 2024-10-15

## Status

Accepted

## Context

A format .

## Decision

The Nexus File created by the Nexus Writer will adhere to the following format. The group structure is given in the next diagram.

- {filename} [`NXclass = NXroot`]

   - Attribute: `file_name: float64`
   - Attribute: `file_time: ISO 8601`
   - Attribute: `HDF_version: string`
   - Attribute: `HDF5_version: string`
   - Attribute: `XML_version: string`

   - `raw_data_1` [`NXclass = NXentry`]

      - Dataset: `collection_time: float64` [`Units = seconds`]

         - Duration of data collection, taking out periods when collection was suspended (e.g. because of a beam off or run control veto)

      - Dataset: `definition: string`

         - A template (DTD name) on which an extension to the base definition is based

         - Attribute: `version: string`

            - DTD version number

         - Attribute: `url: string`

            - URL of XML DTD or schema appropriate for file

      - Dataset: `definition_local: string`

         - A template (DTD name) on which an extension to the base definition is based

      - Dataset: `duration: uint32` (`Units = ?`)

         - Duration of measurement i.e. (end-start)

      - Dataset: `end_time: ISO 8601`

      - Dataset: `experiment_identifier: string`

         - Experiment number, for ISIS, the RB number

      - Dataset: `good_frames: uint32`

         - Number of proton pulses used (not vetoed)

      - Dataset: `idf_version: uint32`

         - Version of IDF that NeXus file confirms to

      - Dataset: `notes: string`

         - log of useful stuff about the experiment, supplied by the user

      - Dataset: `program_name: string = SECI | MCS | CONVERT_NEXUS | SUPERMUSR_NEXUS_WRITER`

         - Name of creating program

      - Dataset: `proton_charge: float64`

         - Attribute: `Units = uA/hr`

      - Dataset: `raw_frames: uint32`

         - Number of proton pulses to target

      - Dataset: `run_cycle: string`

         - ISIS cycle

      - Dataset: `run_number: uint32`

         - Run number

      - Dataset: `start_time: ISO 8601`

      - Dataset: `title : string`

         - Extended title for the entry, e.g. string containing sample, temperature and field

      - Dataset: `total_counts: uint32`

         - Total number of detector events

   - `detector_1` [`NXclass = NXevent_data`]

      - Dataset: `event_id: uint32`
      - Dataset: `event_index: uint64`
      - Dataset: `event_period_number: uint64`
      - Dataset: `event_pulse_height: float64`
      - Dataset: `event_time_offset: uint32` [`Units = ns`]
      - Dataset: `event_time_zero: uint64` [`Units = ns`]

         - Offset of each frame
         - Attribute: `Start: ISO 8601`

            - The date/time of the first entry of `event_time_zero`

   - `instrument`

      - `source`
      - `beamline`
      - `detector_1`
      - `dae`

   - `periods` [`NXclass = NXperiod`]

      - `number: uint32`
      - `type: uint32[period] = 1|2`

         - function of `period`: `1: DAQ`, `2: DWELL`

      - `good_frames: uint32[period]`

         - function of `period`: number of good frames for that period

      - `raw_frames: uint32[period]`

         - function of `period`: number of total frames for that period

   - `runlog` [`NXclass = NXrunlog`]

     This group contains an arbitrary

   - `sample` [`NXclass = NXsample`]

      - `geometry` [`NXclass = NXgeometry`]

      - `environment` [`NXclass = NXenvironment`]

   - `selog` [`NXclass = NXselog`]

   - `user_1` [`NXclass = NXuser`]

|NXclass|
|-------|
|NXroot|

|Attribute|Type|Description|
|-------|----|-----------|
|file_name|float64| |
|file_time|ISO 8601 string| |
|HDF_version|string| |
|HDF5_version|string| |
|XML_version|string| |

- `raw_data_1`

  |NXclass|
  |-------|
  |NXentry|

  |Dataset|Type|Description|Units|
  |-------|----|-----------|-----|
  |collection_time|float64|Duration of data collection, taking out periods when collection was suspended (e.g. because of a beam off or run control veto)|seconds|
  |definition|string|A template (DTD name) on which an extension to the base definition is based|
  |definition_local|string|A template (DTD name) on which an extension to the base definition is based|
  |duration|uint32|Duration of measurement i.e. (end-start)|?|
  |end_time|ISO 8601| |
  |experiment_identifier|string|Experiment number, for ISIS, the RB number|
  |good_frames|uint32|Number of proton pulses used (not vetoed)|
  |idf_version|uint32|Version of IDF that NeXus file confirms to|
  |notes|string|log of useful stuff about the experiment, supplied by the user|
  |program_name|string|Name of creating program (`SECI` \| `MCS` \| `CONVERT_NEXUS`)|
  |proton_charge|float64| |uA/hr|
  |raw_frames|uint32|Number of proton pulses to target|
  |run_cycle|string|ISIS cycle|
  |run_number|uint32|Run number|
  |start_time|ISO 8601| |
  |title|string|extended title for the entry, e.g. string containing sample, temperature and field|
  |total_counts|uint32|Total number of detector events| |

  |For Dataset|Attribute|Type|Description|
  |-----------|---------|----|-----------|
  |definition|version|string|DTD version number|
  |definition|url|string|URL of XML DTD or schema appropriate for file|
  |definition_local|version|string|DTD version number|
  |definition_local|url|string|URL of XML DTD or schema appropriate for file|
  |program_name|version|string|version of creating program|
  |program_name|configuration|string|Configuration of software e.g. SECI configuration|

   - `detector_1`

     |NXclass|
     |-------|
     |NXevent_data|

     |Dataset|Type|Description|Units|
     |-------|----|-----------|-----|
     |event_id|uint32| |
     |event_index|uint64| |
     |event_period_number|uint64| |
     |event_pulse_height|float64| |
     |event_time_offset|uint32| |ns|
     |event_time_zero|uint64| |ns|

     |For Dataset|Attribute|Type|Description|
     |-----------|---------|----|-----------|
     |event_time_zero|Start|string| |

   - `instrument`

      - `source`
      - `beamline`
      - `detector_1`
      - `dae`

   - `periods`

     |NXclass|
     |-------|
     |NXperiod|

     |Dataset|Type|Description|Units|
     |-------|----|-----------|-----|
     |number|uint32| |
     |type|uint32[`period`]|function of `period`: `1: DAQ`, `2: DWELL`|
     |good_frames|uint32[`period`]|function of `period`: number of good frames for that period|
     |raw_frames|uint32[`period`]|function of `period`: number of total frames for that period|

   - `runlog`

     |NXclass|
     |-------|
     |NXrunlog|

     This group contains an arbitrary

   - `sample`

     |NXclass|
     |-------|
     |NXsample|

      - `geometry`

        |NXclass|
        |-------|
        |NXgeometry|

      - `environment`

        |NXclass|
        |-------|
        |NXenvironment|

   - `selog`

     |NXclass|
     |-------|
     |NXselog|

   - `user_1`

     |NXclass|
     |-------|
     |NXuser|

## Consequences

What becomes easier or more difficult to do and any risks introduced by the change that will need to be mitigated.
