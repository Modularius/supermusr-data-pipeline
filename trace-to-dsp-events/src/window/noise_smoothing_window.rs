use crate::Real;

use crate::window::{smoothing_window::SmoothingWindow, Window};

use crate::tagged::{
    SNRSign,
    Stats
};

#[derive(Default,Clone)]
pub struct NoiseSmoothingWindow {
    threshold: Real,
    _influence: Real, //  Maybe we should do something with this?
    position: Real,
    window: SmoothingWindow,
}
impl NoiseSmoothingWindow {
    pub fn new(size: usize, threshold: Real, _influence: Real) -> Self {
        NoiseSmoothingWindow {
            threshold,
            _influence,
            window: SmoothingWindow::new(size),
            ..Default::default()
        }
    }
}
impl Window for NoiseSmoothingWindow {
    type TimeType = Real;
    type InputType = Real;
    type OutputType = Stats;

    fn push(&mut self, value: Real) -> bool {
        if let Some(mut stats) = self.window.stats() {
            stats.value = value - self.position;
            if let SNRSign::Zero = stats.signal_over_noise_sign(self.threshold) {
                self.window.push(value - self.position)
            } else {
                self.position = value - stats.value;
                true
            }
        } else {
            self.window.push(value)
        }
    }
    fn stats(&self) -> Option<Self::OutputType> {
        let mut stats = self.window.stats()?;
        stats.shift(self.position);
        Some(stats)
    }
    fn apply_time_shift(&self, time : Real ) -> Real { self.window.apply_time_shift(time) }
}
