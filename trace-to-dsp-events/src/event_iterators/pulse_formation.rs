use std::{collections::VecDeque, fmt::Display};

//use fitting::{gaussian::GaussianError, Gaussian, ndarray::Array1};

use itertools::Itertools;

use crate::{
    Real,
    RealArray,
    events::{
        event::Event,
        EventData,
    },
    detectors::{
        composite::{CompositeData, CompositeEvent, CompositeTopOnlyEvent},
        change_detector::{ChangeEvent, ChangeData},
    },
    peak_detector,
};




#[derive(Clone)]
pub struct Gaussian {
    a : Real,   // Peak
    mu : Real,  // Mean
    sigma : Real,   // Standard Deviation
}


impl Gaussian {
    pub fn new(a : Real, mu : Real, sigma : Real) -> Self {
        Gaussian {a, mu, sigma }
    }
    pub fn get_value_at(&self, t : Real) -> Real {
        self.a * (-0.5*((t - self.mu)/self.sigma).powi(2)).exp()
    }
    pub fn get_deriv_at(&self, t : Real) -> Real {
        self.get_value_at(t)*(t - self.mu)/self.sigma.powi(2)
    }
    pub fn get_second_deriv_at(&self, t : Real) -> Real {
        self.get_value_at(t)*(((t - self.mu)/self.sigma).powi(2) - 1.)/self.sigma.powi(2)
    }
}


#[derive(Default, Debug, Clone)]
pub struct PulseData {
    peak_time: Option<Real>,
    peak_intensity: Option<Real>,
    //area_under_curve: Option<Real>,
    radius: Option<Real>,
    //start: Option<Real>,
    //end: Option<Real>,
}

impl PulseData {
    pub fn new(peak_time: Option<Real>, peak_intensity: Option<Real>, radius: Option<Real>) -> Self {
        Self { peak_time, peak_intensity, radius }
    }
    pub fn get_peak_time(&self) -> Option<Real> { self.peak_time }
    pub fn get_peak_intensity(&self) -> Option<Real> { self.peak_intensity }
    pub fn get_radius(&self) -> Option<Real> { self.radius }

    pub fn set_peak_time(&mut self, peak_time: Option<Real>) { self.peak_time = peak_time; }
    pub fn set_peak_intensity(&mut self, peak_intensity: Option<Real>) { self.peak_intensity = peak_intensity; }
    pub fn set_radius(&mut self, radius: Option<Real> ) { self.radius = radius; }
}

impl EventData for PulseData {
    fn has_influence_at(&self, index: Real) -> bool {
        true
    }

    fn get_intensity_at(&self, time: Real) -> Real {
        Gaussian::new(self.peak_intensity.unwrap_or_default(),self.peak_time.unwrap_or_default(),self.radius.unwrap_or_default()).get_value_at(time)
    }
}
impl Display for PulseData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{},{}",//,{},{},{}",
            self.peak_intensity.unwrap_or(-1.),
            self.radius.unwrap_or(-1.),
            //self.half_peak_full_width.unwrap_or(-1.),
            //self.start.unwrap_or(-1.),
            //self.end.unwrap_or(-1.),
        ))?;
        Ok(())
    }
}
pub type PulseEvent = Event<PulseData>;


const N : usize = 3;

//type PulseFormationInputType<const N : usize> = (CompositeTopOnlyEvent<ChangeData,N>,CompositeTopOnlyEvent<ChangeData,N>,CompositeTopOnlyEvent<ChangeData,N>);
type PulseFormationInputType<const N : usize> = CompositeTopOnlyEvent<ChangeData,N>;

#[derive(Clone)]
pub struct PulseFormationIter<I> where
    I: Iterator<Item = PulseFormationInputType<N>>,
{
    source: I,
    past_pulses: VecDeque<Gaussian>,
}

impl<I> PulseFormationIter<I> where
    I: Iterator<Item = PulseFormationInputType<N>>,
{
    pub fn new(source: I) -> Self {
        PulseFormationIter { source, past_pulses: VecDeque::<Gaussian>::default() }
    }
}


impl<I> PulseFormationIter<I> where
    I: Iterator<Item = PulseFormationInputType<N>>,
{
    fn to_gaussian_from_one(&mut self, event : CompositeTopOnlyEvent<ChangeData,N>) -> PulseEvent {
        let t = event.get_time();
        while self.past_pulses.front().map(|gaussian|gaussian.get_value_at(t).abs() < 1e-5).unwrap_or(false) {
            self.past_pulses.pop_front();
        }
        let y0 = event.get_data().get_value()[0] - self.past_pulses.iter().map(|gaussian|gaussian.get_value_at(t)).sum::<Real>();
        let y1 = event.get_data().get_value()[1] - self.past_pulses.iter().map(|gaussian|gaussian.get_deriv_at(t)).sum::<Real>();
        let y2 = event.get_data().get_value()[2] - self.past_pulses.iter().map(|gaussian|gaussian.get_second_deriv_at(t)).sum::<Real>();
        let d = y1.powi(2) - y0*y2;
        if d < 0. {
            return PulseEvent {
                time: t,
                data: PulseData {
                    peak_time: Some(t),
                    peak_intensity: Some(0.),
                    radius: Some(Real::NAN),
                },
            }
        }
        let gaussian = Gaussian {
            a: Real::exp(y0*(0.5*y1.powi(2)/d)),
            mu: t + y0*y1/d,
            sigma: Real::sqrt(0.5*(2.*y0.powi(2)/d)),
        };
        self.past_pulses.push_back(gaussian.clone());
        PulseEvent {
            time: gaussian.mu,
            data: PulseData {
                peak_time: Some(gaussian.mu),
                peak_intensity: Some(gaussian.a),
                radius: Some(gaussian.sigma),
            },
        }
    }

    /*fn to_gaussian_from_three(&mut self, event : PulseFormationInputType<N>) -> PulseEvent {
        PulseEvent {
            time: event.1.get_time(),
            data: PulseData {
                peak_intensity: Some(event.1.get_data().get_value()[0]),
                radius: Some(Real::min(event.1.get_time() - event.0.get_time(),event.2.get_time() - event.1.get_time())),
            },
        }
    }*/
}



impl<I> Iterator for PulseFormationIter<I> where
    I: Iterator<Item = PulseFormationInputType<N>>,
{
    type Item = PulseEvent;

    fn next(&mut self) -> Option<Self::Item> {
        self.source.next().map(|event|self.to_gaussian_from_one(event))
        //self.source.next().map(|events|self.to_gaussian_from_three(events))
    }
}

pub trait PulseFormationFilter<I> where
    I: Iterator<Item = PulseFormationInputType<N>>,
{
    fn to_pulses(self) -> PulseFormationIter<I>;
}

impl<I> PulseFormationFilter<I> for I where
    I: Iterator<Item = PulseFormationInputType<N>>,
{
    fn to_pulses(self) -> PulseFormationIter<I> {
        PulseFormationIter::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::{PulseFormationFilter, Real, RealArray};
    use common::Intensity;

    #[test]
    fn sample_data() {/*
        let input: Vec<Intensity> = vec![0, 6, 2, 1, 3, 1, 0];
        let output: Vec<RealArray<3>> = input
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as Real, v as Real))
            .to_pulses()
            .map(|(_, x)| x)
            .collect();

        assert_eq!(output[0], [2., -4., -10.]);
        assert_eq!(output[1], [1., -1., 3.]);
        assert_eq!(output[2], [3., 2., 3.]);
        assert_eq!(output[3], [1., -2., -4.]);
        assert_eq!(output[4], [0., -1., 1.]);*/
    }
}

