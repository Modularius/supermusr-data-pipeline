use std::{
    env,
    fmt::Display,
    fs::File,
    io::{Error, Write},
};

//use tdengine::utils::log_then_panic_t;

use crate::{
    tracedata::TraceData,
    EventIter, detectors::FeedbackDetector, log_then_panic_t, events::{Event, EventData}
};

use super::iter::{TraceIterType, TraceIter};


fn create_file(name: &str) -> File {
    let cd = env::current_dir()
        .unwrap_or_else(|e| log_then_panic_t(format!("Cannot obtain current directory : {e}")));
    File::create(cd.join(name))
        .unwrap_or_else(|e| log_then_panic_t(format!("Cannot create {name} file : {e}")))
}


pub trait SaveToFile<I> where I: Iterator,
{
    fn save_to_file(self, name: &str) -> Result<(), Error>;
}

impl<I> SaveToFile<I> for I where
    I: Iterator,
    I::Item : TraceData,
    <I::Item as TraceData>::TimeType : Display,
    <I::Item as TraceData>::ValueType : Display,
{
    fn save_to_file(self, name: &str) -> Result<(), Error> {
        let mut file = create_file(name);
        for trace in self {
            writeln!(&mut file, "{0},{1}",trace.get_time(),trace.get_value())
                .unwrap_or_else(|e| log_then_panic_t(format!("Cannot write to {name} file : {e}")))
        }
        Ok(())
    }
}
/* 
impl<I,D> SaveToFile<I> for I where
    I: Iterator<Item = Event<D>>,
    D : EventData,
{
    fn save_to_file(self, name: &str) -> Result<(), Error> {
        let mut file = create_file(name);
        for event in self.source {
            writeln!(&mut file, "{event}")
                .unwrap_or_else(|e| log_then_panic_t(format!("Cannot write to {name} file : {e}")))
        }
        Ok(())
    }
}
*/