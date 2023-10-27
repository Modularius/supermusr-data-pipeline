use std::{
    env,
    fmt::Debug,
    fs::File,
    io::{Error, ErrorKind, Read, Seek, SeekFrom},
    mem::size_of,
    usize,
};

use common::Intensity;

#[derive(Default, Debug)]
pub struct TraceFileHeader {
    pub prog_version: String,
    pub run_descript: String,
    pub resolution: i32,
    pub number_of_channels: i32,
    pub channel_enabled: Vec<bool>,
    pub volts_scale_factor: Vec<f64>,
    pub channel_offset_volts: Vec<f64>,
    pub sample_time: f64,
    pub number_of_samples: i32,
    pub trigger_enabled: Vec<bool>,
    pub ex_trigger_enabled: bool,
    pub trigger_level: Vec<f64>,
    pub ex_trigger_level: f64,
    pub trigger_slope: Vec<i32>,
    pub ex_trigger_slope: i32,
    total_bytes: usize,
}

impl TraceFileHeader {
    pub fn load(file: &mut File) -> Result<Self, Error> {
        let mut total_bytes = usize::default();
        let prog_version = load_string(file, &mut total_bytes)?;
        let run_descript = load_string(file, &mut total_bytes)?;
        let resolution = load_i32(file, &mut total_bytes)?;
        let number_of_channels = load_i32(file, &mut total_bytes)?;
        Ok(TraceFileHeader {
            prog_version,
            run_descript,
            resolution,
            number_of_channels,
            channel_enabled: load_bool_vec(file, number_of_channels as usize, &mut total_bytes)?,
            volts_scale_factor: load_f64_vec(file, number_of_channels as usize, &mut total_bytes)?,
            channel_offset_volts: load_f64_vec(
                file,
                number_of_channels as usize,
                &mut total_bytes,
            )?,
            sample_time: load_f64(file, &mut total_bytes)?,
            number_of_samples: load_i32(file, &mut total_bytes)?,
            trigger_enabled: load_bool_vec(file, number_of_channels as usize, &mut total_bytes)?,
            ex_trigger_enabled: load_bool(file, &mut total_bytes)?,
            trigger_level: load_f64_vec(file, number_of_channels as usize, &mut total_bytes)?,
            ex_trigger_level: load_f64(file, &mut total_bytes)?,
            trigger_slope: load_i32_vec(file, number_of_channels as usize, &mut total_bytes)?,
            ex_trigger_slope: load_i32(file, &mut total_bytes)?,
            total_bytes,
        })
    }
}

impl TraceFileHeader {
    fn get_total_bytes(&self) -> usize {
        self.total_bytes
    }
    fn get_size(&self) -> usize {
        size_of::<i32>() + self.prog_version.len() + //pub prog_version : String,
        size_of::<i32>() + self.run_descript.len() + //pub run_descript : String,
        size_of::<i32>() + //pub resolution : i32,
        size_of::<i32>() + //pub number_of_channels : i32,
        size_of::<bool>()*self.number_of_channels as usize +//pub channel_enabled : Vec<bool>,
        size_of::<f64>()*self.number_of_channels as usize + //pub volts_scale_factor : Vec<f64>,
        size_of::<f64>()*self.number_of_channels as usize + //pub channel_offset_volts : Vec<f64>,
        size_of::<f64>() + //pub sample_time : f64,
        size_of::<i32>() + //pub number_of_samples : i32,
        size_of::<bool>()*self.number_of_channels as usize + //pub trigger_enabled : Vec<bool>,
        size_of::<bool>() + //pub ex_trigger_enabled : bool,
        size_of::<f64>()*self.number_of_channels as usize + //pub trigger_level : Vec<f64>,
        size_of::<f64>() + //pub ex_trigger_level : f64,
        size_of::<i32>()*self.number_of_channels as usize + //pub trigger_slope : Vec<i32>,
        size_of::<i32>() //pub ex_trigger_slope : i32,
    }
    fn get_event_size(&self) -> usize {
        TraceFileEvent::get_size(
            self.number_of_channels as usize,
            self.number_of_samples as usize,
        )
    }
    fn get_event(&self, file: &mut File) -> Result<TraceFileEvent, Error> {
        Ok(TraceFileEvent::load_raw(
            file,
            self.number_of_channels as usize,
            self.number_of_samples as usize,
            &self.volts_scale_factor,
            &self.channel_offset_volts,
        )?)
    }
}

#[derive(Default, Debug)]
pub struct TraceFileEvent {
    pub cur_event: i32,
    pub event_runtime: f64,
    pub number_saved_traces: i32,
    pub saved_channels: Vec<bool>,
    pub trigger_time: f64,
    pub trace: Vec<Vec<f64>>,
    pub raw_trace: Vec<Vec<Intensity>>,
    total_bytes: usize,
}
impl TraceFileEvent {
    //fn get_total_bytes(&self) -> usize { self.total_bytes }
    fn get_size(num_channels: usize, num_samples: usize) -> usize {
        size_of::<i32>() + //pub cur_event : i32,
        size_of::<f64>() + //pub event_runtime : f64,
        size_of::<i32>() + //pub number_saved_traces : i32,
        size_of::<bool>()*num_channels + //pub saved_channels : Vec<bool>,
        size_of::<f64>() + //pub trigger_time : f64,
        size_of::<Intensity>()*num_channels*num_samples //pub raw_trace : Vec<Vec<Intensity>>,
    }
    pub fn clone_channel_trace(&self, channel: usize) -> Vec<f64> {
        self.trace[channel].clone()
    }
    pub fn channel_trace(&self, channel: usize) -> &Vec<f64> {
        &self.trace[channel]
    }
    /*pub fn clone_normalized_channel_trace(&self, channel: usize) -> Vec<f64> {
        self.normalized_trace[channel].clone()
    }
    pub fn normalized_channel_trace(&self, channel: usize) -> &Vec<f64> {
        &self.normalized_trace[channel]
    }*/

    pub fn load(file: &mut File, num_channels: usize, num_samples: usize) -> Result<Self, Error> {
        let mut total_bytes = usize::default();
        Ok(TraceFileEvent {
            cur_event: load_i32(file, &mut total_bytes)?,
            event_runtime: load_f64(file, &mut total_bytes)?,
            number_saved_traces: load_i32(file, &mut total_bytes)?,
            saved_channels: load_bool_vec(file, num_channels, &mut total_bytes)?,
            trigger_time: load_f64(file, &mut total_bytes)?,
            total_bytes,
            ..Default::default()
        })
    }

    pub fn load_raw(file: &mut File, num_channels: usize, num_samples: usize, scale: &[f64], offset : &[f64]) -> Result<Self, Error> {
        let mut total_bytes = usize::default();
        let event = Self::load(file, num_channels, num_samples)?;
        Ok(TraceFileEvent {
            cur_event: event.cur_event,
            event_runtime: event.event_runtime,
            number_saved_traces: event.number_saved_traces,
            saved_channels: event.saved_channels,
            trigger_time: event.trigger_time,
            raw_trace: (0..num_channels)
                .map(|c| load_raw_trace(file, num_samples, &mut total_bytes))
                .collect::<Result<_,_>>()?,
            total_bytes:event.total_bytes + total_bytes,
            ..Default::default()
        })
    }

    pub fn load_real(file: &mut File, num_channels: usize, num_samples: usize, scale: &[f64], offset : &[f64]) -> Result<Self, Error> {
        let mut total_bytes = usize::default();
        let event = Self::load(file, num_channels, num_samples)?;
        Ok(TraceFileEvent {
            cur_event: event.cur_event,
            event_runtime: event.event_runtime,
            number_saved_traces: event.number_saved_traces,
            saved_channels: event.saved_channels,
            trigger_time: event.trigger_time,
            trace: (0..num_channels)
                .map(|c| load_trace(file, num_samples, &mut total_bytes, scale[c], offset[c]))
                .collect::<Result<_,_>>()?,
            total_bytes:event.total_bytes + total_bytes,
            ..Default::default()
        })
    }
}

#[derive(Debug)]
pub struct TraceFile {
    file: File,
    header: TraceFileHeader,
    num_events: usize,
}

impl TraceFile {
    pub fn get_event(&mut self, event: usize) -> Result<TraceFileEvent, Error> {
        if event < self.num_events {
            self.file.seek(SeekFrom::Start(
                (self.header.get_size() + event * self.header.get_event_size()) as u64,
            ))?;
            self.header.get_event(&mut self.file)
        } else {
            Err(Error::new(
                ErrorKind::InvalidInput,
                "Invalid event index: {event} should be less than {num_events}",
            ))
        }
    }
    pub fn get_num_event(&self) -> usize { self.num_events }
    pub fn get_num_channels(&self) -> usize { self.header.number_of_channels as usize }
    pub fn get_num_samples(&self) -> usize { self.header.number_of_samples as usize }
}

pub fn load_trace_file(name: &str) -> Result<TraceFile, Error> {
    let cd = env::current_dir()
        .unwrap_or_else(|e| panic!("Cannot obtain current directory : {e}"));
    let mut file = File::open(cd.join(name))?;
    let header: TraceFileHeader = TraceFileHeader::load(&mut file)?;
    let file_size = file.metadata().unwrap().len() as usize;
    let size_minus_header = file_size - header.get_total_bytes();
    let event_size = header.get_event_size();
    if size_minus_header % event_size != 0 {
        // Error
        Err(Error::new(
            ErrorKind::Other,
            format!("Problem: {0} != 0", size_minus_header % event_size),
        ))
    } else {
        Ok(TraceFile {
            file,
            header,
            num_events: size_minus_header / event_size,
        })
    }
}

fn load_scalar<const B: usize>(
    file: &mut File,
    bytes: &mut [u8],
    total_bytes: &mut usize,
) -> Result<(), Error> {
    let num_bytes = file.read(bytes).unwrap();
    *total_bytes += num_bytes;
    if num_bytes == B {
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::UnexpectedEof,
            format!("Expected {B} bytes, got {num_bytes}."),
        ))
    }
}

pub fn load_i32(file: &mut File, total_bytes: &mut usize) -> Result<i32, Error> {
    let mut bytes = i32::to_le_bytes(0);
    load_scalar::<4>(file, &mut bytes, total_bytes)?;
    Ok(i32::from_le_bytes(bytes))
}

pub fn load_f64(file: &mut File, total_bytes: &mut usize) -> Result<f64, Error> {
    let mut bytes = f64::to_le_bytes(0.);
    load_scalar::<8>(file, &mut bytes, total_bytes)?;
    Ok(f64::from_le_bytes(bytes))
}

pub fn load_bool(file: &mut File, total_bytes: &mut usize) -> Result<bool, Error> {
    let mut bytes = u8::to_le_bytes(0);
    load_scalar::<1>(file, &mut bytes, total_bytes)?;
    Ok(u8::from_le_bytes(bytes) != 0)
}

pub fn load_bool_vec(
    file: &mut File,
    size: usize,
    total_bytes: &mut usize,
) -> Result<Vec<bool>, Error> {
    Ok((0..size)
        .map(|_| load_bool(file, total_bytes).unwrap())
        .collect())
}

pub fn load_f64_vec(
    file: &mut File,
    size: usize,
    total_bytes: &mut usize,
) -> Result<Vec<f64>, Error> {
    Ok((0..size)
        .map(|_| load_f64(file, total_bytes).unwrap())
        .collect())
}

pub fn load_i32_vec(
    file: &mut File,
    size: usize,
    total_bytes: &mut usize,
) -> Result<Vec<i32>, Error> {
    Ok((0..size)
        .map(|_| load_i32(file, total_bytes).unwrap())
        .collect())
}

pub fn load_string(file: &mut File, total_bytes: &mut usize) -> Result<String, Error> {
    let size = load_i32(file, total_bytes).unwrap();
    *total_bytes += size as usize;
    let mut string_bytes = Vec::<u8>::new();
    string_bytes.resize(size as usize, 0);
    file.read_exact(&mut string_bytes).unwrap();
    Ok(String::from_utf8(string_bytes).unwrap())
}
pub fn load_trace(
    file: &mut File,
    size: usize,
    total_bytes: &mut usize,
    scale : f64,
    offset : f64,
) -> Result<Vec<f64>, Error> {
    let mut trace_bytes = Vec::<u8>::new();
    let bytes = (Intensity::BITS / u8::BITS) as usize * size;
    *total_bytes += bytes;

    trace_bytes.resize(bytes, 0);
    file.read_exact(&mut trace_bytes).unwrap();
    Ok((0..size)
        .map(|i| Intensity::from_be_bytes([trace_bytes[2 * i], trace_bytes[2 * i + 1]]))
        .map(|i| i as f64*scale - offset)
        .collect())
}
pub fn load_raw_trace(
    file: &mut File,
    size: usize,
    total_bytes: &mut usize,
) -> Result<Vec<Intensity>, Error> {
    let mut trace_bytes = Vec::<u8>::new();
    let bytes = (Intensity::BITS / u8::BITS) as usize * size;
    *total_bytes += bytes;

    trace_bytes.resize(bytes, 0);
    file.read_exact(&mut trace_bytes).unwrap();
    Ok((0..size)
        .map(|i| Intensity::from_be_bytes([trace_bytes[2 * i], trace_bytes[2 * i + 1]]))
        .collect())
}
