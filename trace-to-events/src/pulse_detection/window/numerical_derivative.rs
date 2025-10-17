use super::{Real, RealArray, Window};
use std::collections::VecDeque;

#[derive(Default, Debug, Clone)]
pub(crate) struct NumericalDerivative {
    coefficients: Vec<Real>,
    values: VecDeque<Real>,
    diff: RealArray<2>,
}

fn factorial(n: i32) -> i32 {
    (1..=n).fold(1, i32::saturating_mul)
}

impl NumericalDerivative {
    pub(crate) fn new(radius: i32) -> Self {
        NumericalDerivative {
            values: VecDeque::<Real>::with_capacity(2 * radius as usize + 1),
            coefficients: ((-radius)..=radius)
                .map(|p| {
                    if p == 0 {
                        0.0
                    } else {
                        ((-1_i32).pow(p.unsigned_abs()) * factorial(radius).pow(2)) as f64
                            / (p * factorial(radius - p) * factorial(radius + p)) as f64
                    }
                })
                .collect(),
            diff: RealArray::new([Real::default(); 2]),
        }
    }

    fn midpoint(&self) -> usize {
        self.values.len().div_ceil(2)
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
                *self
                    .values
                    .get(self.midpoint())
                    .expect("Midpoint should exist, this should never fail"),
                self.values
                    .iter()
                    .zip(self.coefficients.iter())
                    .map(|(coef, values)| coef * values)
                    .sum(),
            ]);
            self.values.pop_back();
            true
        }
    }

    fn output(&self) -> Option<Self::OutputType> {
        (self.values.len() + 1 == self.values.capacity()).then_some(self.diff)
    }

    fn apply_time_shift(&self, time: Self::TimeType) -> Self::TimeType {
        time - self.midpoint() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pulse_detection::window::WindowFilter;
    use assert_approx_eq::assert_approx_eq;
    use supermusr_common::Intensity;

    fn b2bexp(
        x: Real,
        ampl: Real,
        spread: Real,
        x0: Real,
        rising: Real,
        falling: Real,
    ) -> Intensity {
        let normalising_factor = ampl * 0.5 * (rising * falling) / (rising + falling);
        let rising_spread = rising * spread.powi(2);
        let falling_spread = falling * spread.powi(2);
        let x_shift = x - x0;
        let rising_exp = Real::exp(rising * 0.5 * (rising_spread + 2.0 * x_shift));
        let rising_erfc = libm::erfc((rising_spread + x_shift) / (Real::sqrt(2.0) * spread));
        let falling_exp = Real::exp(falling * 0.5 * (falling_spread - 2.0 * x_shift));
        let falling_erfc = libm::erfc((falling_spread - x_shift) / (Real::sqrt(2.0) * spread));
        (normalising_factor * (rising_exp * rising_erfc + falling_exp * falling_erfc)) as Intensity
    }

    #[test]
    fn radius_1() {
        let input = (0..30)
            .map(|x| b2bexp(x as Real, 1000.0, 3.5, 15.0, 3.5, 2.25))
            .collect::<Vec<_>>();
        let window_fn = NumericalDerivative::new(1);
        let output = input
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as Real, v as Real))
            .window(window_fn)
            .map(|x| (x.0, x.1[1]))
            .collect::<Vec<_>>();

        assert_eq!(output[5].0, 6.0);
        assert_approx_eq!(output[5].1, 3.0);
        assert_eq!(output[12].0, 13.0);
        assert_approx_eq!(output[12].1, 15.5);
        assert_eq!(output[21].0, 22.0);
        assert_approx_eq!(output[21].1, -9.5);
    }

    #[test]
    fn radius_2() {
        let input = (0..30)
            .map(|x| b2bexp(x as Real, 1000.0, 3.5, 15.0, 3.5, 2.25))
            .collect::<Vec<_>>();
        let window_fn = NumericalDerivative::new(2);
        let output = input
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as Real, v as Real))
            .window(window_fn)
            .map(|x| (x.0, x.1[1]))
            .collect::<Vec<_>>();

        assert_eq!(output[4].0, 6.0);
        assert_approx_eq!(output[4].1, 2.833333333333333);
        assert_eq!(output[11].0, 13.0);
        assert_approx_eq!(output[11].1, 16.0);
        assert_eq!(output[20].0, 22.0);
        assert_approx_eq!(output[20].1, -9.33333333333333);
    }

    #[test]
    fn radius_3() {
        let input = (0..30)
            .map(|x| b2bexp(x as Real, 1000.0, 3.5, 15.0, 3.5, 2.25))
            .collect::<Vec<_>>();
        let window_fn = NumericalDerivative::new(3);
        let output = input
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as Real, v as Real))
            .window(window_fn)
            .map(|x| (x.0, x.1[1]))
            .collect::<Vec<_>>();

        assert_eq!(output[3].0, 6.0);
        assert_approx_eq!(output[3].1, 2.8);
        assert_eq!(output[10].0, 13.0);
        assert_approx_eq!(output[10].1, 16.03333333333333);
        assert_eq!(output[19].0, 22.0);
        assert_approx_eq!(output[19].1, -9.25);
    }

    #[test]
    fn radius_4() {
        let input = (0..30)
            .map(|x| b2bexp(x as Real, 1000.0, 3.5, 15.0, 3.5, 2.25))
            .collect::<Vec<_>>();
        let window_fn = NumericalDerivative::new(4);
        let output = input
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as Real, v as Real))
            .window(window_fn)
            .map(|x| (x.0, x.1[1]))
            .collect::<Vec<_>>();

        assert_eq!(output[2].0, 6.0);
        assert_approx_eq!(output[2].1, 2.7785714285714294);
        assert_eq!(output[9].0, 13.0);
        assert_approx_eq!(output[9].1, 16.040476190476205);
        assert_eq!(output[18].0, 22.0);
        assert_approx_eq!(output[18].1, -9.2);
    }
}
