use std::sync::Arc;

use thiserror::Error;

use crate::time::{Time, TimeCompatible, TimeFn};

pub use rodio::RodioBackend;

mod rodio;

pub type Sample = f32;

#[derive(Debug, Clone, Error)]
#[error("channel multiplexer requires at least a time function")]
pub struct NoChannels;

#[derive(Debug)]
pub struct SampleSource<T> {
    channels: Arc<[T]>,
    curr_channel: usize,
    sample_rate: u32,
    second_sample: u32,
    start: Time,
    end: Time,
}

impl<T> Clone for SampleSource<T> {
    fn clone(&self) -> Self {
        Self {
            channels: self.channels.clone(),
            curr_channel: self.curr_channel,
            sample_rate: self.sample_rate,
            second_sample: 0,
            start: self.start,
            end: self.end,
        }
    }
}

impl<T> Iterator for SampleSource<T>
where
    T: TimeFn<Output = Sample>,
{
    type Item = T::Output;

    fn next(&mut self) -> Option<Self::Item> {
        let offset = (self.second_sample as TimeCompatible)
            / (self.sample_rate as TimeCompatible);
        let curr_time = self.start + offset;

        if curr_time <= self.end {
            let data = self.channels[self.curr_channel].at(curr_time);
            self.curr_channel += 1;
            if self.curr_channel >= self.channels.len() {
                self.second_sample += 1;
                if self.second_sample >= self.sample_rate {
                    self.sample_rate = 0;
                    self.start += 1.0;
                }
                self.curr_channel = 0;
            }
            Some(data)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Player<T, B> {
    channels: Arc<[T]>,
    backend: B,
    sample_rate: u32,
}

impl<T, B> Clone for Player<T, B>
where
    B: Clone,
{
    fn clone(&self) -> Self {
        Self {
            channels: self.channels.clone(),
            backend: self.backend.clone(),
            sample_rate: self.sample_rate,
        }
    }
}

impl<T, B> Player<T, B>
where
    T: TimeFn<Output = Sample>,
    B: Backend,
{
    pub fn new<I>(channels: I, backend: B) -> Result<Self, NoChannels>
    where
        I: IntoIterator<Item = T>,
    {
        let channels: Arc<_> = channels.into_iter().collect();
        if channels.is_empty() {
            Err(NoChannels)
        } else {
            Ok(Player { channels, backend, sample_rate: 48_000 })
        }
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn set_sample_rate(&mut self, value: u32) {
        self.sample_rate = value;
    }

    pub fn play(&mut self, start: Time, end: Time)
    where
        T: Send + Sync + 'static,
    {
        let source = SampleSource {
            channels: self.channels.clone(),
            curr_channel: 0,
            sample_rate: self.sample_rate,
            second_sample: 0,
            start,
            end,
        };

        self.backend.stop();
        self.backend.play(source);
    }

    pub fn stop(&mut self) {
        self.backend.stop();
    }

    pub fn wait(&mut self) {
        self.backend.wait();
    }
}

pub trait Backend {
    fn play<T>(&mut self, source: SampleSource<T>)
    where
        T: TimeFn<Output = Sample> + Send + Sync + 'static;

    fn stop(&mut self);

    fn wait(&mut self);
}
