use core::fmt;
use std::time::Duration;

use rodio::{Sink, Source};

use crate::time::TimeFn;

use super::{Backend, Sample, SampleSource};

impl<T> Source for SampleSource<T>
where
    T: TimeFn<Output = Sample>,
{
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.channels
            .len()
            .try_into()
            .expect("non-supported number of channels for rodio")
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

pub struct RodioBackend {
    sink: Sink,
}

impl fmt::Debug for RodioBackend {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "RodioBackend")
    }
}

impl RodioBackend {
    pub fn new(sink: Sink) -> Self {
        Self { sink }
    }
}

impl Backend for RodioBackend {
    fn play<T>(&mut self, source: SampleSource<T>)
    where
        T: TimeFn<Output = Sample> + Send + Sync + 'static,
    {
        self.sink.clear();
        self.sink.append(source);
    }

    fn stop(&mut self) {
        self.sink.stop();
    }

    fn wait(&mut self) {
        self.sink.sleep_until_end();
    }
}
