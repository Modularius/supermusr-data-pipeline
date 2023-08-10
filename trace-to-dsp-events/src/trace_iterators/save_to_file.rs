use std::{
    env,
    fmt::Display,
    fs::File,
    io::{Error, Write},
};

use tdengine::utils::log_then_panic_t;

use crate::{Detector, EventIter, detectors::FeedbackDetector};

use super::TraceData;

pub trait SaveToFile<I> where I: Iterator,
{
    fn save_to_file(self, name: &str) -> Result<(), Error>;
}

impl<I, T: Display, V: Display> SaveToFile<I> for I where
    I: Iterator,
    I::Item : TraceData<TimeType = T, ValueType = V>,
{
    fn save_to_file(self, name: &str) -> Result<(), Error> {
        let cd = env::current_dir()
            .unwrap_or_else(|e| log_then_panic_t(format!("Cannot obtain current directory : {e}")));
        let mut file = File::create(cd.join(name))
            .unwrap_or_else(|e| log_then_panic_t(format!("Cannot create {name} file : {e}")));
        for trace in self {
            writeln!(&mut file, "{0},{1}",trace.get_time(),trace.get_value())
                .unwrap_or_else(|e| log_then_panic_t(format!("Cannot write to {name} file : {e}")))
        }
        Ok(())
    }
}

impl<I, D> SaveToFile<I> for EventIter<I, D> where
    I: Iterator,
    I::Item : TraceData<TimeType = D::TimeType, ValueType = D::ValueType, ParameterType = D::ParameterType>,
    D: FeedbackDetector,
{
    fn save_to_file(self, name: &str) -> Result<(), Error> {
        let cd = env::current_dir()
            .unwrap_or_else(|e| log_then_panic_t(format!("Cannot obtain current directory : {e}")));
        let mut file = File::create(cd.join(name))
            .unwrap_or_else(|e| log_then_panic_t(format!("Cannot create {name} file : {e}")));
        for event in self {
            writeln!(&mut file, "{event}")
                .unwrap_or_else(|e| log_then_panic_t(format!("Cannot write to {name} file : {e}")))
        }
        Ok(())
    }
}
