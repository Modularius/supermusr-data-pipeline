use std::time::{Instant, Duration};

#[derive(Default)]
pub(crate) struct DescStats {
    num: usize,
    mean: f64,
    stddev: f64,
    skewness: f64,
    median: f64,
    iq_range: f64,
    min: u128,
    max: u128,
}

impl DescStats {
    pub(crate) fn print(&self) {
        println!("Stats for {0} data times", self.num);
        println!("|{0:>10}|{1:>10}|{2:>10}|{3:>10}|{4:>10}|{5:>10}|{6:>10}|",
            "Mean",
            "StdDev",
            "Skewness",
            "Median",
            "IQ Range",
            "Min",
            "Max"
        );
        println!("|{0:10.3}|{1:10.3}|{2:10.3}|{3:10.3}|{4:10.3}|{5:10.3}|{6:10.3}|",
            self.mean,
            self.stddev,
            self.skewness,
            self.median,
            self.iq_range,
            self.min,
            self.max
        );
    }
}
#[derive(Default, Clone)]
pub(crate) struct Record<D : Default + Clone> {
    duration: Duration,
    full_duration: Duration,
    data: D,
}
#[derive(Default, Clone)]
pub(crate) struct StatTimer<D : Default + Clone> {
    // Parameters
    warm_up: usize,
    num: usize,

    begin_time: Option<Instant>,
    full_time: Option<Instant>,
    record: Vec<Record<D>>,
}

fn moment<I : Iterator>(m : usize, num : usize, mean : f64, iter : I) -> f64 where I : Iterator<Item = u128> {
    iter.map(|x|f64::powi(x as f64 - mean,m as i32)).sum::<f64>()/num as f64
}
fn calculate_stats_from<I>(num : usize, iter : I) -> DescStats where I : Iterator<Item=u128> + Clone {
    let mean = iter.clone().map(|x|x as f64).sum::<f64>()/num as f64;
    let stddev = f64::sqrt(moment(2, num - 1, mean, iter.clone()));
    let skewness = moment(3, num, mean, iter.clone())/f64::powi(stddev,3);
    let (median, iq_range) = {
        let mut vec = iter.clone().collect::<Vec<_>>();
        vec.sort();
        (vec[vec.len()/2] as f64, (vec[9*vec.len()/10] - vec[vec.len()/10]) as f64)
    };

    DescStats {
        num, mean, stddev, skewness, median, iq_range,
        max: iter.clone().max().unwrap_or_default(),
        min: iter.min().unwrap_or_default(),
    }
}

impl<D : Default + Clone> StatTimer<D> {
    pub(crate) fn new(
        warm_up: usize,
        num: usize,
    ) -> Self {
        Self { warm_up, num, record: Vec::with_capacity(num + warm_up), ..Default::default() }
    }

    pub(crate) fn has_finished(&self) -> bool {
        self.record.len() == self.record.capacity()
    }

    pub(crate) fn begin_record(&mut self) {
        if self.begin_time.is_none() {
            self.begin_time = Some(Instant::now());
        }
    }
    pub(crate) fn end_record(&mut self, data : D) {
        if let Some(begin_time) = self.begin_time {
            let now = Instant::now();
            self.record.push( Record { data,
                duration: now - begin_time,
                full_duration: self.full_time.map(|full_time| now - full_time).unwrap_or_default()
            } );
            self.begin_time = None;
            self.full_time = Some(now);
        } else {
            panic!("Timer ended before beginning!")
        }
    }
    pub(crate) fn calculate_stats(&self) -> (DescStats,DescStats) {
        let iter1 = self.record.iter().skip(self.warm_up).map(|r|r.duration.as_micros());
        let iter2 = self.record.iter().skip(self.warm_up).map(|r|r.full_duration.as_micros());
        (calculate_stats_from(self.num, iter1),calculate_stats_from(self.num, iter2))
    }
}


#[derive(Default, Clone)]
pub(crate) struct Data {
    pub(crate) bytes_in: usize,
    pub(crate) bytes_out: usize,
}