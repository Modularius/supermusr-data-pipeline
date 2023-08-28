use std::collections::VecDeque;

use crate::Real;

use crate::window::Window;

use crate::tracedata::Stats;

#[derive(Default,Clone)]
pub struct SmoothingWindow {
    value: Real,
    sum: Real,
    sum_of_squares: Real,
    size: Real,
    size_m1: Real,
    window: VecDeque<Real>,
}
impl SmoothingWindow {
    pub fn new(size: usize) -> Self {
        if size < 2 {
            panic!("Size must be >= 2");
        }
        SmoothingWindow {
            window: VecDeque::<Real>::with_capacity(size),
            size: size as Real,
            size_m1: size as Real - 1.,
            ..Default::default()
        }
    }
    pub fn is_full(&self) -> bool {
        self.window.len() == self.window.capacity()
    }
    #[cfg(test)]
    pub fn test_mean(&self) -> Real {
        self.window.iter().sum::<f64>() / self.size
    }
    #[cfg(test)]
    pub fn test_variance(&self) -> Real {
        let mean = self.test_mean();
        self.window.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / self.size_m1
    }
}
impl Window for SmoothingWindow {
    type TimeType = Real;
    type InputType = Real;
    type OutputType = Stats;

    fn push(&mut self, value: Real) -> bool {
        self.value = value;
        if self.is_full() {
            let old = self.window.pop_front().unwrap_or_default();
            self.sum -= old;
            self.sum_of_squares -= old.powi(2);
        }
        self.sum += value;
        self.sum_of_squares += value.powi(2);
        self.window.push_back(value);
        self.is_full()
    }
    fn stats(&self) -> Option<Stats> {
        if self.is_full() {
            Some(Stats {
                value: self.value,
                mean: self.sum / self.size,
                variance: (self.sum_of_squares - self.sum.powi(2) / self.size) / self.size_m1,
            })
        } else {
            None
        }
    }
    fn apply_time_shift(&self, time : Real) -> Real { time } //time - (self.size - 1.)/2.0 }
}

#[cfg(test)]
mod tests {
    use crate::processing;

    use common::Intensity;
    use super::super::iter::WindowFilter;
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    #[should_panic]
    fn test_window_size_zero() {
        let data = [4, 3, 2, 5, 6, 1, 5, 7, 2, 4];
        data.iter()
            .enumerate()
            .map(processing::make_enumerate_real)
            .window(SmoothingWindow::new(0));
    }

    #[test]
    #[should_panic]
    fn test_window_size_one() {
        let data = [4, 3, 2, 5, 6, 1, 5, 7, 2, 4];
        data.iter()
            .enumerate()
            .map(processing::make_enumerate_real)
            .window(SmoothingWindow::new(1));
    }

    #[test]
    fn test_window_size_two() {
        let data = [4, 3, 2, 5, 6, 1, 5, 7, 2, 4];
        assert!(data
            .iter()
            .enumerate()
            .map(processing::make_enumerate_real)
            .window(SmoothingWindow::new(2))
            .next()
            .is_some());
    }

    #[test]
    fn test_no_data() {
        let data = [];
        assert!(data
            .iter()
            .enumerate()
            .map(processing::make_enumerate_real)
            .window(SmoothingWindow::new(3))
            .next()
            .is_none());
    }
    #[test]
    fn test_insufficient_data() {
        let data = [4, 3];
        assert!(data
            .iter()
            .enumerate()
            .map(processing::make_enumerate_real)
            .window(SmoothingWindow::new(3))
            .next()
            .is_none());
    }
    #[test]
    fn test_minimal() {
        let data = [4, 3];
        let (i, stats) = data
            .iter()
            .enumerate()
            .map(processing::make_enumerate_real)
            .window(SmoothingWindow::new(2))
            .next()
            .unwrap();
        assert_eq!(i, 1.);
        assert_eq!(stats.value, 3.);
        assert_approx_eq!(stats.mean, 7. / 2.);
        assert_approx_eq!(
            stats.variance,
            ((4. as Real - 7. / 2.).powi(2) + (3. as Real - 7. / 2.).powi(2)) / (2. - 1.)
        );
    }
    #[test]
    fn test_three_data() {
        let data = [4, 3, 1];
        let (i, stats) = data
            .iter()
            .enumerate()
            .map(processing::make_enumerate_real)
            .window(SmoothingWindow::new(2))
            .skip(1)
            .next()
            .unwrap();
        assert_eq!(i, 2.);
        assert_eq!(stats.value, 1.);
        assert_approx_eq!(stats.mean, 2.);
        assert_approx_eq!(
            stats.variance,
            ((3. as Real - 2.).powi(2) + (1. as Real - 2.).powi(2)) / (2. - 1.)
        );
    }

    #[test]
    fn test_three_data_three_window() {
        let data = [4, 3, 1];
        let (i, stats) = data
            .iter()
            .enumerate()
            .map(processing::make_enumerate_real)
            .window(SmoothingWindow::new(3))
            .next()
            .unwrap();
        assert_eq!(i, 2.);
        assert_eq!(stats.value, 1.);
        assert_approx_eq!(stats.mean, 8. / 3.);
        assert_approx_eq!(
            stats.variance,
            ((4. as Real - 8. / 3.).powi(2)
                + (3. as Real - 8. / 3.).powi(2)
                + (1. as Real - 8. / 3.).powi(2))
                / (3. - 1.)
        );
    }

    #[test]
    fn test_five_data_three_window() {
        let data = [4, 3, 1, 5, 3];
        let mut itr = data
            .iter()
            .enumerate()
            .map(processing::make_enumerate_real)
            .window(SmoothingWindow::new(3));
        let (i, stats) = itr.next().unwrap();
        assert_eq!(i, 2.);
        assert_eq!(stats.value, 1.);
        assert_approx_eq!(stats.mean, 8. / 3.);
        assert_approx_eq!(
            stats.variance,
            ((4. as Real - 8. / 3.).powi(2)
                + (3. as Real - 8. / 3.).powi(2)
                + (1. as Real - 8. / 3.).powi(2))
                / (3. - 1.)
        );

        let (i, stats) = itr.next().unwrap();
        assert_eq!(i, 3.);
        assert_eq!(stats.value, 5.);
        assert_approx_eq!(stats.mean, 9. / 3.);
        assert_approx_eq!(
            stats.variance,
            ((3. as Real - 9. / 3.).powi(2)
                + (1. as Real - 9. / 3.).powi(2)
                + (5. as Real - 9. / 3.).powi(2))
                / (3. - 1.)
        );

        let (i, stats) = itr.next().unwrap();
        assert_eq!(i, 4.);
        assert_eq!(stats.value, 3.);
        assert_approx_eq!(stats.mean, 9. / 3.);
        assert_approx_eq!(
            stats.variance,
            ((1. as Real - 9. / 3.).powi(2)
                + (5. as Real - 9. / 3.).powi(2)
                + (3. as Real - 9. / 3.).powi(2))
                / (3. - 1.)
        );
    }

    #[test]
    fn test_variance_accuracy() {
        use rand::random;
        let data: Vec<Intensity> = (0..1000).map(|_| random::<Intensity>()).collect();

        for window_size in 2..100 {
            let smoothing_window = SmoothingWindow::new(window_size);
            let mut itr = data
                .iter()
                .enumerate()
                .map(processing::make_enumerate_real)
                .window(smoothing_window);
            while let Some(stat) = itr.next() {
                assert_approx_eq!(stat.1.mean, itr.get_window().test_mean());
                assert_approx_eq!(stat.1.variance, itr.get_window().test_variance());
            }
        }
    }

    #[test]
    fn test_variance_zero_on_constant_sequence() {
        use rand::random;
        let num = random::<Intensity>();
        let data: Vec<Intensity> = vec![num; 1000];

        for window_size in 2..100 {
            let smoothing_window = SmoothingWindow::new(window_size);
            let mut itr = data
                .iter()
                .enumerate()
                .map(processing::make_enumerate_real)
                .window(smoothing_window);
            while let Some(stat) = itr.next() {
                assert_eq!(stat.1.mean, num as Real);
                assert_eq!(stat.1.variance, 0.);
            }
        }
    }
}