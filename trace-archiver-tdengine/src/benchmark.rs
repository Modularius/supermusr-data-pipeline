use std::time::Instant;

type BenchmarkRecord = (u128, u128, u128);


pub(crate) struct BenchmarkData {
    times: Vec<BenchmarkRecord>,
    messages_to_benchmark: usize,
    current_time: Instant,
    processing_time: Instant,
    posting_time: Instant,
    batch_size: usize,
}

impl BenchmarkData {
    pub(crate) fn new(messages_to_benchmark: usize, batch_size: usize) -> Self {
        BenchmarkData {
            times: Vec::with_capacity(messages_to_benchmark),
            messages_to_benchmark,
            current_time: Instant::now(),
            processing_time: Instant::now(),
            posting_time: Instant::now(),
            batch_size,
        }
    }

    pub(crate) fn begin_processing_timer(&mut self) {
        if self.messages_to_benchmark > 0 {
            self.processing_time = Instant::now();
        }
    }

    pub(crate) fn begin_posting_timer(&mut self) {
        if self.messages_to_benchmark > 0 {
            self.posting_time = Instant::now();
        }
    }
    pub(crate) fn end_timers(&mut self) {
        if self.messages_to_benchmark > 0 {
            let duration1 = self.processing_time.elapsed().as_micros();
            let duration2 = self.posting_time.elapsed().as_micros();
            let duration3 = (Instant::now() - self.current_time).as_micros();
            self.current_time = Instant::now();
            self.messages_to_benchmark -= 1;
            self.times.push((duration1, duration2, duration3));
        }
    }

    pub(crate) fn print_times(&mut self) {
        if self.messages_to_benchmark == 0 {
            let mut (mean_process, mean_post, mean_interval) = (0, 0, 0);
            for (index,(process, post, interval)) in self.times.iter().enumerate() {
                // println!("Message took {interval} us, taking {process} us to process and {post} us to post.");
                mean_process += process;
                mean_post += post;
                mean_interval += interval;
                if index % self.batch_size == 0 {
                    mean_process /= self.batch_size;
                    mean_post /= self.batch_size;
                    mean_interval /= self.batch_size;
                    println!("Batch sent with MEAN PROCESS = {mean_process} us, MEAN POST = {mean_post} us, MEAN INTERVAL = {mean_interval} us");
                    (mean_process, mean_post, mean_interval) = (0, 0, 0);
                }
            }
            self.times.clear();
        }
    }
}
