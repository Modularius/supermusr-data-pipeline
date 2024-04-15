use chrono::{DateTime, Utc};
use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Exp, Normal};
use serde::Deserialize;
use std::ops::RangeInclusive;
use supermusr_common::{Channel, DigitizerId, FrameNumber, Time};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", untagged)]
pub(crate) enum Expression {
    Fixed(f64),
    FrameTransform(Transformation<f64>),
}

impl Expression {
    fn value(&self, frame_index: usize) -> f64 {
        match self {
            Expression::Fixed(v) => *v,
            Expression::FrameTransform(frame_function) => {
                frame_function.transform(frame_index as f64)
            }
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", untagged)]
pub(crate) enum RandomDistribution {
    Constant(Expression),
    Uniform { min: Expression, max: Expression },
    Normal { mean: Expression, sd: Expression },
    Exponential { lifetime: Expression },
}

impl RandomDistribution {
    pub(crate) fn sample(&self, frame_index: usize) -> f64 {
        match self {
            Self::Constant(t) => t.value(frame_index),
            Self::Uniform { min, max } => {
                rand::rngs::StdRng::seed_from_u64(Utc::now().timestamp_subsec_nanos() as u64)
                    .gen_range(min.value(frame_index)..max.value(frame_index))
            }
            Self::Normal { mean, sd } => {
                Normal::new(mean.value(frame_index), sd.value(frame_index))
                    .unwrap()
                    .sample(&mut rand::rngs::StdRng::seed_from_u64(
                        Utc::now().timestamp_subsec_nanos() as u64,
                    ))
            }
            Self::Exponential { lifetime } => Exp::new(1.0 / lifetime.value(frame_index))
                .unwrap()
                .sample(&mut rand::rngs::StdRng::seed_from_u64(
                    Utc::now().timestamp_subsec_nanos() as u64,
                )),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Pulse {
    pub(crate) weight: f64,
    pub(crate) attributes: PulseAttributes,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub(crate) enum PulseAttributes {
    Flat {
        start: RandomDistribution,
        width: RandomDistribution,
        height: RandomDistribution,
    },
    Triangular {
        start: RandomDistribution,
        peak_time: RandomDistribution,
        width: RandomDistribution,
        height: RandomDistribution,
    },
    Gaussian {
        height: RandomDistribution,
        peak_time: RandomDistribution,
        sd: RandomDistribution,
    },
    Biexp {
        start: RandomDistribution,
        decay: RandomDistribution,
        rise: RandomDistribution,
        height: RandomDistribution,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct NoiseSource {
    bounds: Interval<Time>,
    attributes: NoiseAttributes,
    smoothing_factor: Expression,
}

impl NoiseSource {
    pub(crate) fn smooth(&self, new_value: f64, old_value: f64, frame_index: usize) -> f64 {
        new_value * (1.0 - self.smoothing_factor.value(frame_index))
            + old_value * self.smoothing_factor.value(frame_index)
    }

    pub(crate) fn sample(&self, time: Time, frame_index: usize) -> f64 {
        if self.bounds.is_in(time) {
            match &self.attributes {
                NoiseAttributes::Uniform(Interval { min, max }) => {
                    (max.value(frame_index) - min.value(frame_index)) * rand::random::<f64>()
                        + min.value(frame_index)
                }
                NoiseAttributes::Gaussian { mean, sd } => {
                    Normal::new(mean.value(frame_index), sd.value(frame_index))
                        .unwrap()
                        .sample(&mut rand::rngs::StdRng::seed_from_u64(
                            Utc::now().timestamp_subsec_nanos() as u64,
                        ))
                }
            }
        } else {
            f64::default()
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub(crate) enum NoiseAttributes {
    Uniform(Interval<Expression>),
    Gaussian { mean: Expression, sd: Expression },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Interval<T> {
    pub(crate) min: T,
    pub(crate) max: T,
}

impl<T: PartialOrd + Copy> Interval<T> {
    fn range_inclusive(&self) -> RangeInclusive<T> {
        self.min..=self.max
    }

    fn is_in(&self, value: T) -> bool {
        self.range_inclusive().contains(&value)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Transformation<T> {
    pub(crate) scale: T,
    pub(crate) translate: T,
}

impl Transformation<f64> {
    pub(crate) fn transform(&self, x: f64) -> f64 {
        x * self.scale + self.translate
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Digitizer {
    pub(crate) id: DigitizerId,
    pub(crate) channels: Interval<Channel>,
}

impl Digitizer {
    pub(crate) fn get_channels(&self) -> RangeInclusive<Channel> {
        self.channels.range_inclusive()
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Timestamp {
    Now,
    From(DateTime<Utc>),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", untagged)]
pub(crate) enum Frames {
    Vector(Vec<FrameNumber>),
    Interval(Interval<FrameNumber>),
}

impl<'a> Frames {
    pub(crate) fn iter(&'a self) -> Box<dyn Iterator<Item = FrameNumber>> {
        match self {
            Frames::Vector(vec) => Box::new(vec.clone().into_iter()),
            Frames::Interval(interval) => Box::new(interval.range_inclusive()),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct TraceMessage {
    pub(crate) time_bins: Time,
    pub(crate) digitizers: Vec<Digitizer>,
    pub(crate) frames: Frames,
    pub(crate) pulses: Vec<Pulse>,
    pub(crate) noises: Vec<NoiseSource>,
    pub(crate) num_pulses: RandomDistribution,
    pub(crate) timestamp: Timestamp,
    pub(crate) sample_rate: Option<u64>,
    pub(crate) frame_delay_us: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Simulation {
    pub(crate) voltage_transformation: Transformation<f64>,
    pub(crate) traces: Vec<TraceMessage>,
}


#[cfg(test)]
mod tests {
    use super::*;

    const JSON_INPUT : &str = r#"
            {
                "voltage-transformation": {"scale": 1, "translate": 0 },
                "traces": [
                    {
                        "digitizers": [ { "id": 0, "channels": { "min": 0, "max": 1 } }],
                        "frames": [1, 2, 3, 4, 5],
                        "sample-rate": 100000000,
                        "pulses": [
                            {
                                "weight": 1,
                                "attributes": {
                                    "type": "biexp",
                                    "height": { "min": {"type": "fixed", 30}, "max": 70 },
                                    "start": { "lifetime": 2200 },
                                    "rise": { "min": 20, "max": 30 },
                                    "decay": { "min": 5, "max": 10 }
                                }
                            },
                            {
                                "weight": 1,
                                "attributes": {
                                    "type": "flat",
                                    "start": { "lifetime": 2200 },
                                    "width": { "min": 20, "max": 50 },
                                    "height": { "min": 30, "max": 70 }
                                }
                            },
                            {
                                "weight": 1,
                                "attributes": {
                                    "type": "triangular",
                                    "start": { "lifetime": 2200 },
                                    "width": { "min": 20, "max": 50 },
                                    "peak_time": { "min": 0.25, "max": 0.75 },
                                    "height": { "min": 30, "max": 70 }
                                }
                            }
                        ],
                        "noises": [
                            {
                                "attributes": { "type" : "gaussian", "mean" : 0, "sd" : 20 },
                                "smoothing-factor" : 0.975,
                                "bounds" : { "min": 0, "max": 30000 }
                            },
                            {
                                "attributes": { "type" : "gaussian", "mean" : 0, "sd" : {"scale": 50, "translate": 50 } },
                                "smoothing-factor" : 0.995,
                                "bounds" : { "min": 0, "max": 30000 }
                            }
                        ],
                        "num-pulses": 500,
                        "frame-extra-pulses" : 0,
                        "time-bins": 30000,
                        "timestamp": "now",
                        "frame-delay-us": 20000
                    }
                ]
            }
    "#;

    #[test]
    fn test() {
        let simulation: Simulation = serde_json::from_str(JSON_INPUT).unwrap();
        assert_eq!(simulation.voltage_transformation.scale,1.0);
        assert_eq!(simulation.voltage_transformation.translate,0.0);

        assert_eq!(simulation.traces.len(),1);
        assert_eq!(simulation.traces[0].digitizers.len(),1);
        assert_eq!(simulation.traces[0].digitizers[0].get_channels().collect::<Vec<Channel>>(),vec![0, 1]);
        assert_eq!(simulation.traces[0].frames.iter().collect::<Vec<Channel>>(),vec![1,2,3,4,5]);
        assert_eq!(simulation.traces[0].sample_rate, Some(100_000_000));
        assert_eq!(simulation.traces[0].pulses.len(), 3);
        assert_eq!(simulation.traces[0].noises.len(), 2);
    }
}