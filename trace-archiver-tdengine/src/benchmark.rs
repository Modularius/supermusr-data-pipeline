use std::time::{Instant, Duration};

type BenchmarkRecord = (u128, u128, u128, u128);


pub(crate) struct BenchmarkData {
    times: Vec<BenchmarkRecord>,
    messages_to_benchmark: usize,
    current_time: Instant,
    processing_time: Instant,
    posting_time: Instant,
    binding_time: Instant,
    processing_duration: Duration,
    posting_duration: Duration,
    binding_duration: Duration,
    batch_size: usize,
}

impl BenchmarkData {
    pub(crate) fn new(messages_to_benchmark: usize, batch_size: usize) -> Self {
        BenchmarkData {
            times: Vec::with_capacity(messages_to_benchmark),
            messages_to_benchmark,
            current_time: Instant::now(),
            processing_time: Instant::now(),
            binding_time: Instant::now(),
            posting_time: Instant::now(),
            processing_duration: Duration::default(),
            posting_duration: Duration::default(),
            binding_duration: Duration::default(),
            batch_size,
        }
    }

    pub(crate) fn begin_processing_timer(&mut self) {
        if self.messages_to_benchmark > 0 {
            self.processing_time = Instant::now();
        }
    }
    pub(crate) fn end_processing_timer(&mut self) {
        if self.messages_to_benchmark > 0 {
            self.processing_duration = self.processing_time.elapsed();
        }
    }

    pub(crate) fn begin_binding_timer(&mut self) {
        if self.messages_to_benchmark > 0 {
            self.binding_time = Instant::now();
        }
    }
    pub(crate) fn end_binding_timer(&mut self) {
        if self.messages_to_benchmark > 0 {
            self.binding_duration = self.binding_time.elapsed();
        }
    }

    pub(crate) fn begin_posting_timer(&mut self) {
        if self.messages_to_benchmark > 0 {
            self.posting_time = Instant::now();
        }
    }
    pub(crate) fn end_posting_timer(&mut self) {
        if self.messages_to_benchmark > 0 {
            self.posting_duration = self.posting_time.elapsed();
        }
    }
    pub(crate) fn end_timers(&mut self) {
        if self.messages_to_benchmark > 0 {
            let duration = (Instant::now() - self.current_time).as_micros();
            self.current_time = Instant::now();
            self.messages_to_benchmark -= 1;
            self.times.push((self.processing_duration.as_micros(), self.binding_duration.as_micros(), self.posting_duration.as_micros(),duration));
        }
    }

    pub(crate) fn print_times(&mut self) {
        if self.messages_to_benchmark == 0 {
            let (mut mean_process, mut mean_binding, mut mean_post, mut mean_interval) = (0, 0, 0, 0);
            for (index,(process, binding, post, interval)) in self.times.iter().enumerate() {
                // println!("Message took {interval} us, taking {process} us to process and {post} us to post.");
                mean_process += process;
                mean_binding += binding;
                mean_post += post;
                mean_interval += interval;
                if index % self.batch_size == (self.batch_size - 1) {
                    mean_process /= self.batch_size as u128;
                    mean_binding /= self.batch_size as u128;
                    mean_post /= self.batch_size as u128;
                    mean_interval /= self.batch_size as u128;
                    println!("Batch sent with MEAN PROCESS = {mean_process} us, MEAN BIND = {mean_binding} us, MEAN POST = {mean_post} us, MEAN INTERVAL = {mean_interval} us");
                    (mean_process, mean_binding, mean_post, mean_interval) = (0, 0, 0, 0);
                }
            }
            self.times.clear();
        }
    }
}
