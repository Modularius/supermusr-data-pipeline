use std::fmt::{Debug, Display};

use crate::Real;

use super::{FeedbackDetector, EventValuedDetector};

mod pulse_detector;
mod biexponential;

pub use pulse_detector::PulseEvent;
pub use pulse_detector::PulseDetector;
pub use pulse_detector::Gaussian;
pub use biexponential::Biexponential;

pub trait PulseModel : Default + Debug + Display + Clone {
    fn get_value_at(&self, time: Real) -> Real;
    fn get_derivative_at(&self, time: Real) -> Real;
    fn get_second_derivative_at(&self, time: Real) -> Real;
    
    fn get_effective_interval(&self, bound : Real) -> (Real, Real);

    fn from_data(peak_time: Real, peak_value: Real, area_under_curve: Real) -> Self;
    fn from_data2(data : Vec<Real>, start: Real, peak : Real) -> Self { Self::default() }
    fn from_basic(mean: Real, amplitude: Real,) -> Self;
}

