use super::{Real, RealArray, Window};
use std::collections::VecDeque;

#[derive(Default, Clone)]
pub(crate) struct NumericalDerivative {
    coefficients: Vec<Real>,
    values: VecDeque<Real>,
    diff: RealArray<2>,
}

fn factorial(n: i32) -> i32 {
    let mut f = n;
    for i in 1..n {
        f = f*i;
    }
    f
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
                        (-1.0_f64).powi(p)*factorial(radius).pow(2) as f64/(p*factorial(radius - p)*factorial(radius + p)) as f64
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
    fn sample_data() {
        let input = (0..100)
            .map(|x| {
                b2bexp(x as Real, 1000.0, 3.5, 20.0, 3.5, 2.25)
                    + b2bexp(x as Real, 1000.0, 3.5, 54.0, 4.5, 5.5)
                    + b2bexp(x as Real, 1000.0, 3.5, 81.0, 1.5, 3.25)
            })
            .collect::<Vec<_>>();
        let output = input
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as Real, v as Real))
            .window(NumericalDerivative::new(3))
            .collect::<Vec<_>>();
        println!("{output:?}");
    }
}
