use super::{add_new_group_to, timeseries_file::TimeSeriesDataSource};
use crate::nexus::{
    hdf5_file::{
        hdf5_writer::create_resizable_dataset,
        run_file_components::timeseries_file::get_dataset_builder,
    },
    nexus_class as NX, NexusSettings,
};
use hdf5::{Group, SimpleExtents};
use ndarray::s;
use supermusr_common::FrameNumber;
use supermusr_streaming_types::{
    aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage,
    ecs_f144_logdata_generated::f144_LogData,
};
use tracing::debug;

#[derive(Debug)]
pub(crate) struct RunLog {
    parent: Group,
}

impl RunLog {
    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn new_runlog(parent: &Group) -> anyhow::Result<Self> {
        let logs = add_new_group_to(parent, "runlog", NX::RUNLOG)?;
        Ok(Self { parent: logs })
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn open_runlog(parent: &Group) -> anyhow::Result<Self> {
        let parent = parent.group("runlog")?;
        Ok(Self { parent })
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn push_logdata_to_runlog(
        &mut self,
        logdata: &f144_LogData,
        nexus_settings: &NexusSettings,
    ) -> anyhow::Result<()> {
        debug!("Type: {0:?}", logdata.value_type());

        let timeseries = self.parent.group(logdata.source_name()).or_else(|err| {
            debug!(
                "Cannot find {0}. Creating new group.",
                logdata.source_name()
            );

            let group = add_new_group_to(&self.parent, logdata.source_name(), NX::LOG)
                .map_err(|e| e.context(err))?;

            let time = create_resizable_dataset::<i32>(
                &group,
                "time",
                0,
                nexus_settings.runloglist_chunk_size,
            )?;
            logdata.write_initial_timestamp(&time)?;
            get_dataset_builder(&logdata.get_hdf5_type()?, &group)?
                .shape(SimpleExtents::resizable(vec![0]))
                .chunk(nexus_settings.runloglist_chunk_size)
                .create("value")?;
            Ok::<_, anyhow::Error>(group)
        })?;
        let timestamps = timeseries.dataset("time")?;
        let values = timeseries.dataset("value")?;

        logdata.write_values_to_dataset(&values)?;
        logdata.write_timestamps_to_dataset(&timestamps, 1)?;
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn push_message_to_logs(
        &mut self,
        message: &FrameAssembledEventListMessage,
        nexus_settings: &NexusSettings,
    ) -> anyhow::Result<()> {
        const FRAME_NUMBER: &str = "SuperMuSRDataPipeline_FrameNumber";
        let timeseries = self.parent.group(FRAME_NUMBER).or_else(|err| {
            debug!("Cannot find {FRAME_NUMBER}. Creating new group.");

            let group = add_new_group_to(&self.parent, FRAME_NUMBER, NX::LOG)
                .map_err(|e| e.context(err))?;

            create_resizable_dataset::<FrameNumber>(
                &group,
                "value",
                0,
                nexus_settings.framelist_chunk_size,
            )?;
            Ok::<_, anyhow::Error>(group)
        })?;
        let values = timeseries.dataset("value")?;

        let position = values.size();
        let slice = s![position..(position + 1)];
        values.resize(position + 1)?;
        values.write_slice(&[message.metadata().frame_number()], slice)?;
        Ok(())
    }
}
