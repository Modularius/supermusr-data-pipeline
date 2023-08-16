pub mod iter;
//pub mod composite;
pub mod gate;
pub mod noise_smoothing_window;
pub mod smoothing_window;
pub mod trivial;

pub use iter::{
    WindowFilter,
    WindowIter
};


pub trait Window {
    type TimeType: Copy;
    type InputType: Copy;
    type OutputType;

    fn push(&mut self, value: Self::InputType) -> bool;
    fn stats(&self) -> Option<Self::OutputType>;
    fn apply_time_shift(&self, time : Self::TimeType) -> Self::TimeType;
}





