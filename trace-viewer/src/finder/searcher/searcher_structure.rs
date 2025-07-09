use crate::{
    Timestamp,
    finder::searcher::{BackstepIter, BinarySearchIter, ForwardSearchIter},
    messages::{BorrowedMessageError, FBMessage},
};
use miette::{Error, IntoDiagnostic};
use rdkafka::{
    Offset, TopicPartitionList,
    consumer::{Consumer, StreamConsumer},
    error::KafkaError,
};
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, info, instrument, warn};

#[derive(Default, Error, Debug)]
pub(crate) enum SearcherError {
    #[default]
    #[error("Unknown Error")]
    UnknownError,
    #[error("Topic start reached")]
    StartOfTopicReached,
    #[error("Topic end reached")]
    EndOfTopicReached,
    #[error("No valid message found")]
    NoMessageFound(#[from] BorrowedMessageError),
    #[error("Kafka Error {0}")]
    Kafka(#[from] KafkaError),
}

/// Object to search through the broker from a given offset, on a given topic, for messages of type `M`.
pub(crate) struct Searcher<'a, M, C, G> {
    /// Reference to the Kafka consumer.
    pub(super) consumer: &'a C,
    /// Topic to search on.
    pub(super) topic: String,
    /// Current offset.
    pub(super) offset: i64,
    /// Offset function.
    pub(super) offset_fn: G,
    /// Results accumulate here.
    pub(super) results: Vec<M>,
}

impl<'a, M, C: Consumer, G> Searcher<'a, M, C, G> {
    /// Creates a new instance, and assigns the given topic to the broker's consumer.
    ///
    /// # Parameters
    /// - consumer: the broker's consumer to use.
    /// - topic: the topic to search on.
    /// - offset: the offset to search from.
    /// - send_status: send channel, along which status messages should be sent.
    #[instrument(skip_all)]
    pub(crate) fn new(
        consumer: &'a C,
        topic: &str,
        offset: i64,
        offset_fn: G,
    ) -> Result<Self, SearcherError> {
        //consumer.unsubscribe();
        //consumer.subscribe(&[topic])?;
        consumer.unassign()?;
        let mut tpl = TopicPartitionList::with_capacity(1);
        tpl.add_partition_offset(topic, 0, rdkafka::Offset::End)
            .expect("Cannot set offset to end.");
        consumer.assign(&tpl).expect("Cannot assign to consumer.");
        Ok(Self {
            consumer,
            offset,
            offset_fn,
            topic: topic.to_owned(),
            results: Default::default(),
        })
    }

    #[instrument(skip_all)]
    /// Consumer the searcher and create a backstep iterator.
    pub(crate) fn iter_backstep(self) -> BackstepIter<'a, M, C, G> {
        BackstepIter {
            inner: self,
            step_size: None,
        }
    }

    #[instrument(skip_all)]
    /// Consumer the searcher and create a forward iterator.
    pub(crate) fn iter_forward(self) -> ForwardSearchIter<'a, M, C, G> {
        ForwardSearchIter {
            inner: self,
            message: None,
        }
    }

    #[instrument(skip_all)]
    /// Consumer the searcher and create a forward iterator.
    pub(crate) fn iter_binary(self, target: Timestamp) -> BinarySearchIter<'a, M, C, G> {
        BinarySearchIter {
            inner: self,
            bound: Default::default(),
            max_bound: Default::default(),
            target,
        }
    }

    /// Sets the offset.
    pub(super) fn set_offset(&mut self, offset: i64) {
        self.offset = offset;
    }

    /// Gets the offset.
    pub(crate) fn get_offset(&self) -> i64 {
        self.offset
    }
}

/// Extracts the results from the searcher, when the user is finished with it.
impl<'a, M, C, F> From<Searcher<'a, M, C, F>> for Vec<M> {
    #[instrument(skip_all)]
    fn from(value: Searcher<'a, M, C, F>) -> Vec<M> {
        value.results
    }
}

impl<'a, M, F: Fn(i64) -> Offset> Searcher<'a, M, StreamConsumer, F>
where
    M: FBMessage<'a>,
{
    #[instrument(skip_all, fields(offset=offset))]
    pub(crate) async fn message(&mut self, offset: i64) -> Result<Option<M>, Error> {
        self.message_from_raw_offset((self.offset_fn)(offset)).await
    }

    /// This method calls [StreamConsumer::recv] repeatedly until a message of type [M]
    /// is successfully obtained.
    /// 
    /// The maximum number of tries is fixed by `MAX_TRY_FROM` to prevent an infinite loop.
    /// If this maximum is reached, the method returns an error with all individual errors
    /// wrapped together.
    async fn message_until_try_from(&self) -> Result<Option<M>, Error> {
        /// The maximum number of times to attempt to consume a message.
        const MAX_TRY_FROM : usize = 64;
        const RECV_TIMEOUT_MS: u64 = 500;

        let mut err : Option<Error> = None;
        for _ in 0..MAX_TRY_FROM {
            // Introduce a timeout to prevent [StreamConsumer::recv()] blocking infinitely.
            // If the timeout is reached, assume no message is available.
            let recv_timeout = match tokio::time::timeout(Duration::from_millis(RECV_TIMEOUT_MS), self.consumer.recv()).await {
                Ok(recv) => recv,
                Err(_) => {
                    return Ok(None);
                },
            };

            match M::try_from(recv_timeout.into_diagnostic()?) {
                Ok(message) => {
                    return Ok(Some(message));
                },
                Err(new_error) => {
                    warn!("try_from failed: {new_error}");
                    err = match err {
                        Some(inner_error) => Some(inner_error.wrap_err(new_error)),
                        None => Some(Error::from_err(new_error)),
                    }
                },
            }
        };
        Err(err.expect("Error should exist, this should never fail"))
    }

    #[instrument(skip_all)]
    pub(crate) async fn message_from_raw_offset(
        &mut self,
        offset: Offset,
    ) -> Result<Option<M>, Error> {
        self.consumer
            .seek(&self.topic, 0, offset, Duration::from_millis(1)).into_diagnostic()?;

        let msg = self.message_until_try_from().await?;

        match msg.as_ref() {
            Some(msg) => debug!( "Message at offset {offset:?}: timestamp: {0}", msg.timestamp()),
            None => debug!("No message found at offset {offset:?}")
        }
        
        Ok(msg)
    }
}
