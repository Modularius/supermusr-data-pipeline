use super::{Real, RealArray, Window};
use std::collections::VecDeque;

#[derive(Default, Clone)]
pub(crate) struct NumericalDerivative {
    coefficients: Vec<Real>,
    values: VecDeque<Real>,
    diff: RealArray<2>,
}

impl NumericalDerivative {
    pub(crate) fn new(radius: i32) -> Self {
        NumericalDerivative {
            values: VecDeque::<Real>::with_capacity(2*radius as usize + 1),
            coefficients: ((-radius)..=radius)
                .map(|p| {
                    if p == 0 {
                        0.0
                    } else {
                        (-1.0_f64).powi(p)*libm::factorial(radius).powi(2) as f64/(p*libm::factorial(radius - p)*libm::factorial(radius + p)) as f64
                    }
                })
                .collect(),
            diff: RealArray::new([Real::default(); 2]),
        }
    }

    fn midpoint(&self) -> usize {
        (self.values.len() + 1)/2
    }
}

impl Window for NumericalDerivative {
    type TimeType = Real;
    type InputType = Real;
    type OutputType = RealArray<2>;

    fn push(&mut self, value: Self::InputType) -> bool {
        if self.values.len() + 1 < self.values.capacity() {
            self.values.push_front(value);
            false
        } else {
            self.values.push_front(value);
            self.diff = RealArray::new([
                *self.values
                    .get(self.midpoint())
                    .expect("Midpoint should exist, this should never fail"),
                self.values
                    .iter()
                    .zip(self.coefficients.iter())
                    .map(|(coef, values)|coef*values)
                    .sum()
            ]);
            self.values.pop_back();
            true
        }
    }

    fn output(&self) -> Option<Self::OutputType> {
        (self.values.len() + 1 < self.values.capacity())
            .then_some(self.diff)
    }

    fn apply_time_shift(&self, time: Self::TimeType) -> Self::TimeType {
        time - self.midpoint() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pulse_detection::window::WindowFilter;
    use supermusr_common::Intensity;

    #[test]
    fn sample_data() {
        let input: Vec<Intensity> = vec![0, 6, 2, 1, 3, 1, 0];
        let mut output = input
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as Real, v as Real))
            .window(NumericalDerivative::new(3))
            .map(|(_, x)| x);

        assert_eq!(output.next(), Some(RealArray::new([2., -4., -10.])));
        assert_eq!(output.next(), Some(RealArray::new([1., -1., 3.])));
        assert_eq!(output.next(), Some(RealArray::new([3., 2., 3.])));
        assert_eq!(output.next(), Some(RealArray::new([1., -2., -4.])));
        assert_eq!(output.next(), Some(RealArray::new([0., -1., 1.])));
        assert!(output.next().is_none());
    }
}
