use std::{
    env,
    fs::File,
    io::{Error, Write},
};

//use tdengine::utils::log_then_panic_t;

use crate::{events::Event, log_then_panic_t, tracedata::{EventData, Temporal}, pulse::Pulse};

pub trait SaveEventsToFile<T,I,D> where
    T: Temporal,
    I: Iterator<Item = Event<T,D>>,
    D : EventData
{
    fn save_to_file(self, name: &str) -> Result<(), Error>;
}

impl<T,I,D> SaveEventsToFile<T,I,D> for I where
    T: Temporal,
    I: Iterator<Item = Event<T,D>>,
    D : EventData
{
    fn save_to_file(self, name: &str) -> Result<(), Error> {
        let cd = env::current_dir()
            .unwrap_or_else(|e| log_then_panic_t(format!("Cannot obtain current directory : {e}")));
        let mut file = File::create(cd.join(name))
            .unwrap_or_else(|e| log_then_panic_t(format!("Cannot create {name} file : {e}")));
        for event in self {
            writeln!(&mut file, "{0},{1}",event.get_time(),event.get_data())
                .unwrap_or_else(|e| log_then_panic_t(format!("Cannot write to {name} file : {e}")))
        }
        Ok(())
    }
}

pub trait SavePulsesToFile<I> where
    I: Iterator<Item = Pulse>,
{
    fn save_to_file(self, name: &str) -> Result<(), Error>;
}

impl<I> SavePulsesToFile<I> for I where
    I: Iterator<Item = Pulse>,
{
    fn save_to_file(self, name: &str) -> Result<(), Error> {
        let cd = env::current_dir()
            .unwrap_or_else(|e| log_then_panic_t(format!("Cannot obtain current directory : {e}")));
        let mut file = File::create(cd.join(name))
            .unwrap_or_else(|e| log_then_panic_t(format!("Cannot create {name} file : {e}")));
        for pulse in self {
            writeln!(&mut file, "{pulse}")
                .unwrap_or_else(|e| log_then_panic_t(format!("Cannot write to {name} file : {e}")))
        }
        Ok(())
    }
}
