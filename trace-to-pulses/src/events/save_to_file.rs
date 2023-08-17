use std::{
    env,
    fs::File,
    io::{Error, Write},
};

//use tdengine::utils::log_then_panic_t;

use crate::{events::{
    EventData,
    event::Event
}, log_then_panic_t};

pub trait SaveEventsToFile<I,D> where
    I: Iterator<Item = Event<D>>,
    D : EventData
{
    fn save_to_file(self, name: &str) -> Result<(), Error>;
}

impl<I, D> SaveEventsToFile<I, D> for I where
    I: Iterator<Item = Event<D>>,
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
