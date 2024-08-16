use crate::{
    data::{Accumulate, DigitiserData},
    TIMESTAMP_FORMAT,
};
use std::{collections::HashMap, fmt::Debug, time::Duration};
use supermusr_common::{
    spanned::{FindSpan, SpannedAggregator},
    DigitizerId,
};

#[cfg(not(test))]
use supermusr_common::spanned::Spanned;
use supermusr_streaming_types::FrameMetadata;

use super::{partial::PartialFrame, AggregatedFrame};

pub(crate) struct FrameCache<D: Debug> {
    ttl: Duration,
    expected_digitisers: Vec<DigitizerId>,

    frames: HashMap<FrameMetadata, PartialFrame<D>>,
}

impl<D: Debug> FrameCache<D>
where
    DigitiserData<D>: Accumulate<D>,
{
    pub(crate) fn new(ttl: Duration, expected_digitisers: Vec<DigitizerId>) -> Self {
        Self {
            ttl,
            expected_digitisers,
            frames: Default::default(),
        }
    }

    #[tracing::instrument(skip_all, target = "otel")]
    fn existing_frame_found(_frame: &mut PartialFrame<D>) {
        // _frame is used here as in test mode, this parameter is not used
        // In test mode, the frame.span() are not initialised
        #[cfg(not(test))]
        tracing::Span::current().follows_from(_frame.span().get().expect("Span should exist"));
    }

    #[tracing::instrument(skip_all, target = "otel")]
    fn new_frame(ttl: Duration, metadata: FrameMetadata) -> PartialFrame<D> {
        let mut frame = PartialFrame::<D>::new(ttl, metadata);
        frame.span_init(); // Initialise the span field
        frame
    }

    #[tracing::instrument(skip_all, fields(
        digitiser_id = digitiser_id,
        metadata_timestamp = metadata.timestamp.format(TIMESTAMP_FORMAT).to_string(),
        metadata_frame_number = metadata.frame_number,
        metadata_period_number = metadata.period_number,
        metadata_veto_flags = metadata.veto_flags,
        metadata_protons_per_pulse = metadata.protons_per_pulse,
        metadata_running = metadata.running
    ))]
    pub(crate) fn push<'a>(
        &'a mut self,
        digitiser_id: DigitizerId,
        metadata: &FrameMetadata,
        data: D,
    ) -> &'a impl SpannedAggregator {
        let frame = self
            .frames
            .entry(metadata.clone()) // Find the frame with the given metadata
            .and_modify(Self::existing_frame_found) // If it exists, apply the associated existing_frame_found function to it
            .or_insert_with(|| Self::new_frame(self.ttl, metadata.clone())); // Otherwise create a new PartialFrame

        frame.push(digitiser_id, data);
        frame.push_veto_flags(metadata.veto_flags);
        frame
    }

    pub(crate) fn poll(&mut self) -> Option<AggregatedFrame<D>> {
        // Find a frame which is completed
        let metadata = self
            .frames
            .keys()
            .find(|metadata| {
                let frame = self
                    .frames
                    .get(metadata)
                    .expect("Frame with metadata should exist");
                frame.is_complete(&self.expected_digitisers) | frame.is_expired()
            })
            .cloned();

        // If such a frame is found, then remove it from the hashmap and return as aggregated frame
        metadata
            .and_then(|metadata| self.frames.remove(&metadata))
            .map(|frame| {
                frame.end_span();
                frame.into()
            })
    }
    pub(crate) fn get_num_partial_frames(&self) -> usize {
        self.frames.len()
    }
}

impl<'a, D: Debug> FindSpan<'a> for FrameCache<D> {
    type Key = FrameMetadata;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::data::EventData;
    use chrono::Utc;

    #[test]
    fn one_frame_in_one_frame_out() {
        let mut cache = FrameCache::<EventData>::new(Duration::from_millis(100), vec![0, 1, 4, 8]);

        let frame_1 = FrameMetadata {
            timestamp: Utc::now(),
            period_number: 1,
            protons_per_pulse: 8,
            running: true,
            frame_number: 1728,
            veto_flags: 4,
        };

        assert!(cache.poll().is_none());

        assert_eq!(cache.get_num_partial_frames(), 0);
        cache.push(0, &frame_1, EventData::dummy_data(0, 5, &[0, 1, 2]));
        assert_eq!(cache.get_num_partial_frames(), 1);

        assert!(cache.poll().is_none());

        cache.push(1, &frame_1, EventData::dummy_data(0, 5, &[3, 4, 5]));

        assert!(cache.poll().is_none());

        cache.push(4, &frame_1, EventData::dummy_data(0, 5, &[6, 7, 8]));

        assert!(cache.poll().is_none());

        cache.push(8, &frame_1, EventData::dummy_data(0, 5, &[9, 10, 11]));

        {
            let frame = cache.poll().unwrap();
            assert_eq!(cache.get_num_partial_frames(), 0);

            assert_eq!(frame.metadata, frame_1);

            let mut dids = frame.digitiser_ids;
            dids.sort();
            assert_eq!(dids, &[0, 1, 4, 8]);

            assert_eq!(
                frame.digitiser_data,
                EventData::new(
                    vec![
                        0, 1, 2, 3, 4, 0, 1, 2, 3, 4, 0, 1, 2, 3, 4, 0, 1, 2, 3, 4, 0, 1, 2, 3, 4,
                        0, 1, 2, 3, 4, 0, 1, 2, 3, 4, 0, 1, 2, 3, 4, 0, 1, 2, 3, 4, 0, 1, 2, 3, 4,
                        0, 1, 2, 3, 4, 0, 1, 2, 3, 4
                    ],
                    vec![0; 60],
                    vec![
                        0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4,
                        5, 5, 5, 5, 5, 6, 6, 6, 6, 6, 7, 7, 7, 7, 7, 8, 8, 8, 8, 8, 9, 9, 9, 9, 9,
                        10, 10, 10, 10, 10, 11, 11, 11, 11, 11
                    ],
                )
            );
        }

        assert!(cache.poll().is_none());
    }

    #[tokio::test]
    async fn one_frame_in_one_frame_out_missing_digitiser_timeout() {
        let mut cache = FrameCache::<EventData>::new(Duration::from_millis(100), vec![0, 1, 4, 8]);

        let frame_1 = FrameMetadata {
            timestamp: Utc::now(),
            period_number: 1,
            protons_per_pulse: 8,
            running: true,
            frame_number: 1728,
            veto_flags: 4,
        };

        assert!(cache.poll().is_none());

        cache.push(0, &frame_1, EventData::dummy_data(0, 5, &[0, 1, 2]));

        assert!(cache.poll().is_none());

        cache.push(1, &frame_1, EventData::dummy_data(0, 5, &[3, 4, 5]));

        assert!(cache.poll().is_none());

        cache.push(8, &frame_1, EventData::dummy_data(0, 5, &[9, 10, 11]));

        assert!(cache.poll().is_none());

        tokio::time::sleep(Duration::from_millis(105)).await;

        {
            let frame = cache.poll().unwrap();

            assert_eq!(frame.metadata, frame_1);

            let mut dids = frame.digitiser_ids;
            dids.sort();
            assert_eq!(dids, &[0, 1, 8]);

            assert_eq!(
                frame.digitiser_data,
                EventData::new(
                    vec![
                        0, 1, 2, 3, 4, 0, 1, 2, 3, 4, 0, 1, 2, 3, 4, 0, 1, 2, 3, 4, 0, 1, 2, 3, 4,
                        0, 1, 2, 3, 4, 0, 1, 2, 3, 4, 0, 1, 2, 3, 4, 0, 1, 2, 3, 4,
                    ],
                    vec![0; 45],
                    vec![
                        0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4,
                        5, 5, 5, 5, 5, 9, 9, 9, 9, 9, 10, 10, 10, 10, 10, 11, 11, 11, 11, 11
                    ],
                )
            );
        }

        assert!(cache.poll().is_none());
    }

    #[test]
    fn test_metadata_equality() {
        let mut cache = FrameCache::<EventData>::new(Duration::from_millis(100), vec![1, 2]);

        let timestamp = Utc::now();
        let frame_1 = FrameMetadata {
            timestamp,
            period_number: 1,
            protons_per_pulse: 8,
            running: true,
            frame_number: 1728,
            veto_flags: 4,
        };

        let frame_2 = FrameMetadata {
            timestamp,
            period_number: 1,
            protons_per_pulse: 8,
            running: true,
            frame_number: 1728,
            veto_flags: 5,
        };

        assert_eq!(cache.frames.len(), 0);
        assert!(cache.poll().is_none());

        cache.push(1, &frame_1, EventData::dummy_data(0, 5, &[0, 1, 2]));
        assert_eq!(cache.frames.len(), 1);
        assert!(cache.poll().is_none());

        cache.push(2, &frame_2, EventData::dummy_data(0, 5, &[0, 1, 2]));
        assert_eq!(cache.frames.len(), 1);
        assert!(cache.poll().is_some());
        //let frame = cache.poll().is_some();
    }
}
