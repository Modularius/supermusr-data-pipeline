use super::{Real, RealArray, Window};
use std::collections::VecDeque;

#[derive(Default, Debug, Clone)]
pub(crate) struct NumericalDerivative {
    coefficients: Vec<Real>,
    values: VecDeque<Real>,
    diff: RealArray<2>,
    midpoint: usize,
}

fn prod(from: i32, to: i32) -> Real {
    (from..=to).fold(1, i32::saturating_mul) as Real
}

fn nonzero_coef(p: i32, n: i32) -> Real {
    // 1 2 3 4 ... (n - |p| - 1) (n - |p|) (n - |p| + 1) ... (n - 1) n (n + 1) ... (n + |p| - 1) (n + |p|)
    (-1_f64).powi(p + 1)*
    prod(n - (p.abs() - 1), n)
    /(prod(n + 1, n + p.abs()) * p  as f64)
}

impl NumericalDerivative {
    pub(crate) fn new(radius: usize) -> Self {
        NumericalDerivative {
            values: VecDeque::<Real>::with_capacity(2 * radius + 1),
            coefficients: (-(radius as i32)..=radius as i32)
                .map(|p| (p != 0)
                    .then(||nonzero_coef(p, radius as i32))
                    .unwrap_or_default()
                )
                .rev() // We reverse the order of the coeffients due to how the temp values are stored.
                .collect(),
            diff: RealArray::new([Real::default(); 2]),
            midpoint: radius as usize
        }
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
                    .get(self.midpoint)
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
        time - self.midpoint as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pulse_detection::window::WindowFilter;
    use assert_approx_eq::assert_approx_eq;
    use std::ops::Range;
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

    #[test]
    fn findif_coef_accuracy() {
        assert_approx_eq!(nonzero_coef(-1, 1), -0.5);
        assert_approx_eq!(nonzero_coef(1, 1), 0.5);

        assert_approx_eq!(nonzero_coef(-2, 2), 1.0/12.0);
        assert_approx_eq!(nonzero_coef(-1, 2), -2.0/3.0);
        assert_approx_eq!(nonzero_coef(1, 2), 2.0/3.0);
        assert_approx_eq!(nonzero_coef(2, 2), -1.0/12.0);

        assert_approx_eq!(nonzero_coef(-3, 3), -1.0/60.0);
        assert_approx_eq!(nonzero_coef(-2, 3), 3.0/20.0);
        assert_approx_eq!(nonzero_coef(-1, 3), -3.0/4.0);
        assert_approx_eq!(nonzero_coef(1, 3), 3.0/4.0);
        assert_approx_eq!(nonzero_coef(2, 3), -3.0/20.0);
        assert_approx_eq!(nonzero_coef(3, 3), 1.0/60.0);

        assert_approx_eq!(nonzero_coef(-4, 4), 1.0/280.0);
        assert_approx_eq!(nonzero_coef(-3, 4), -4.0/105.0);
        assert_approx_eq!(nonzero_coef(-2, 4), 1.0/5.0);
        assert_approx_eq!(nonzero_coef(-1, 4), -4.0/5.0);
        assert_approx_eq!(nonzero_coef(1, 4), 4.0/5.0);
        assert_approx_eq!(nonzero_coef(2, 4), -1.0/5.0);
        assert_approx_eq!(nonzero_coef(3, 4), 4.0/105.0);
        assert_approx_eq!(nonzero_coef(4, 4), -1.0/280.0);
    }

    fn derivative_accuracy(size: usize, radius_bounds: Range<usize>, f: impl Fn(Real) -> Real, df_dx: impl Fn(Real) -> Real) {
        let x = (0..size).map(|x|x as Real);
        let y = x.clone().map(f).collect::<Vec<_>>();
        let dy_dx = x.map(df_dx).collect::<Vec<_>>();

        for radius in radius_bounds {
            let dy_dx_exact = dy_dx.iter()
                .enumerate()
                .take(size - radius)
                .skip(radius)
                .collect::<Vec<_>>();

            let window_fn = NumericalDerivative::new(radius);
            let dy_dx_approx = y.iter()
                .enumerate()
                .map(|(i, v)| (i as Real, *v as Real))
                .window(window_fn)
                .map(|x| (x.0, (x.1[0], x.1[1])))
                .collect::<Vec<_>>();

            // Both should be of the same size now.
            assert_eq!(dy_dx_exact.len(), dy_dx_approx.len());

            let y_trunc = y.iter()
                .take(size - radius)
                .skip(radius)
                .collect::<Vec<_>>();
            
            // Both should be of the same size now.
            assert_eq!(y_trunc.len(), dy_dx_approx.len());
            for (&y1, &(_, (y2, _))) in Iterator::zip(y_trunc.iter(),dy_dx_approx.iter()) {
                // Check the y values agree.
                assert_approx_eq!(y1, y2);
            }

            for ((i_exact, &d_exact), (i_approx, (_, d_approx))) in Iterator::zip(dy_dx_exact.into_iter(),dy_dx_approx.into_iter()) {
                // Check the indices agree.
                assert_eq!(i_exact as i32, i_approx as i32);
                // The derivatives should be approximately equal to within 1e-6.
                assert_approx_eq!(d_exact, d_approx);
            }
        }
    }

    #[test]
    fn polynomial_derivative_accuracy() {
        let f = |x: Real|x.powi(3) + 3.0*x.powi(2);
        let df_dx = |x: Real|3.0*x.powi(2) + 6.0*x;
        derivative_accuracy(120, 2..9, f, df_dx);
    }

    #[test]
    fn sine_derivative_accuracy() {
        let f = |x: Real|Real::sin(x/10.0);
        let df_dx = |x: Real|Real::cos(x/10.0)/10.0;
        derivative_accuracy(100, 2..9, f, df_dx);
    }
}
