use crate::data::DigitiserData;
use std::time::Duration;
use supermusr_common::spanned::{SpanOnce, Spanned};
use supermusr_common::DigitizerId;
use supermusr_streaming_types::FrameMetadata;
use tokio::time::Instant;
use tracing::Span;

pub(super) struct PartialFrame<D> {
    span: SpanOnce,
    expiry: Instant,

    pub(super) metadata: FrameMetadata,
    pub(super) digitiser_data: DigitiserData<D>,
}

impl<D> PartialFrame<D> {
    pub(super) fn new(ttl: Duration, metadata: FrameMetadata) -> Self {
        let expiry = Instant::now() + ttl;

        Self {
            span: SpanOnce::default(),
            expiry,
            metadata,
            digitiser_data: Default::default(),
        }
    }
    pub(super) fn digitiser_ids(&self) -> Vec<DigitizerId> {
        let mut cache_digitiser_ids: Vec<DigitizerId> =
            self.digitiser_data.iter().map(|i| i.0).collect();
        cache_digitiser_ids.sort();
        cache_digitiser_ids
    }

    pub(super) fn push(&mut self, digitiser_id: DigitizerId, data: D) {
        self.digitiser_data.push((digitiser_id, data));
    }

    pub(super) fn is_complete(&self, expected_digitisers: &[DigitizerId]) -> bool {
        self.digitiser_ids() == expected_digitisers
    }

    pub(super) fn is_expired(&self) -> bool {
        Instant::now() > self.expiry
    }
}

impl<D> Spanned for PartialFrame<D> {
    fn init_span(&mut self, span: Span) {
        self.span = match self.span {
            SpanOnce::Waiting => SpanOnce::Spanned(span),
            _ => panic!(),
        };
    }

    fn get_span(&self) -> &Span {
        match &self.span {
            SpanOnce::Spanned(span) => span,
            _ => panic!(),
        }
    }

    fn inherit_span(&mut self) -> SpanOnce {
        let span = match &self.span {
            SpanOnce::Spanned(span) => span.clone(),
            _ => panic!(),
        };
        self.span = SpanOnce::Spent;
        SpanOnce::Spanned(span)
    }
}
