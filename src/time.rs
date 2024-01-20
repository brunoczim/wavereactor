use std::{cmp::Ordering, rc::Rc, sync::Arc};

use thiserror::Error;

pub type TimeCompatible = f32;

pub type Time = TimeCompatible;

pub type DynTimeFn<'t, A> = dyn TimeFn<Output = A> + 't + Send + Sync;

pub type UnsyncDynTimeFn<'t, A> = dyn TimeFn<Output = A> + 't;

pub trait TimeFn {
    type Output;

    fn at(&self, seconds: Time) -> Self::Output;

    fn into_dyn<'t>(self) -> Arc<DynTimeFn<'t, Self::Output>>
    where
        Self: Sized + Send + Sync + 't,
    {
        Arc::new(self)
    }

    fn into_unsync_dyn<'t>(self) -> Rc<UnsyncDynTimeFn<'t, Self::Output>>
    where
        Self: Sized + 't,
    {
        Rc::new(self)
    }

    fn proxy<F>(self, time_proxy: F) -> Proxy<Self, F>
    where
        Self: Sized,
        F: Fn(Time) -> Time,
    {
        Proxy { inner: self, time_proxy }
    }

    fn map<F, A>(self, mapper: F) -> Map<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Output) -> A,
    {
        Map { inner: self, mapper }
    }

    fn compose<T>(self, inner: T) -> Compose<Self, T>
    where
        Self: Sized,
        T: TimeFn<Output = Time>,
    {
        Compose { outer: self, inner }
    }

    fn with_speed(self, speed: TimeCompatible) -> WithSpeed<Self>
    where
        Self: Sized,
    {
        WithSpeed { inner: self, speed }
    }

    fn step_at<U>(self, at_seconds: Time, after: U) -> StepAt<Self, U>
    where
        Self: Sized,
        U: TimeFn<Output = Self::Output>,
    {
        StepAt { before: self, at_seconds, after }
    }
}

impl<'t, T> TimeFn for &'t T
where
    T: TimeFn + ?Sized,
{
    type Output = T::Output;

    fn at(&self, time: Time) -> Self::Output {
        (**self).at(time)
    }
}

impl<'t, T> TimeFn for &'t mut T
where
    T: TimeFn + ?Sized,
{
    type Output = T::Output;

    fn at(&self, time: Time) -> Self::Output {
        (**self).at(time)
    }
}

impl<T> TimeFn for Box<T>
where
    T: TimeFn + ?Sized,
{
    type Output = T::Output;

    fn at(&self, time: Time) -> Self::Output {
        (**self).at(time)
    }
}

impl<T> TimeFn for Rc<T>
where
    T: TimeFn + ?Sized,
{
    type Output = T::Output;

    fn at(&self, time: Time) -> Self::Output {
        (**self).at(time)
    }
}

impl<T> TimeFn for Arc<T>
where
    T: TimeFn + ?Sized,
{
    type Output = T::Output;

    fn at(&self, time: Time) -> Self::Output {
        (**self).at(time)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TimeClosureFn<F> {
    closure: F,
}

impl<F, A> TimeFn for TimeClosureFn<F>
where
    F: Fn(Time) -> A,
{
    type Output = A;

    fn at(&self, seconds: Time) -> Self::Output {
        (self.closure)(seconds)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Proxy<T, F> {
    inner: T,
    time_proxy: F,
}

impl<T, F> TimeFn for Proxy<T, F>
where
    T: TimeFn,
    F: Fn(Time) -> Time,
{
    type Output = T::Output;

    fn at(&self, seconds: Time) -> Self::Output {
        self.inner.at((self.time_proxy)(seconds))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Map<T, F> {
    inner: T,
    mapper: F,
}

impl<T, F, A> TimeFn for Map<T, F>
where
    T: TimeFn,
    F: Fn(T::Output) -> A,
{
    type Output = A;

    fn at(&self, seconds: Time) -> Self::Output {
        (self.mapper)(self.inner.at(seconds))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WithSpeed<T> {
    inner: T,
    speed: TimeCompatible,
}

impl<T> TimeFn for WithSpeed<T>
where
    T: TimeFn,
{
    type Output = T::Output;

    fn at(&self, seconds: Time) -> Self::Output {
        self.inner.at(seconds * self.speed)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Compose<U, T> {
    outer: U,
    inner: T,
}

impl<U, T> TimeFn for Compose<U, T>
where
    U: TimeFn,
    T: TimeFn<Output = Time>,
{
    type Output = U::Output;

    fn at(&self, seconds: Time) -> Self::Output {
        self.outer.at(self.inner.at(seconds))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StepAt<T, U> {
    before: T,
    at_seconds: Time,
    after: U,
}

impl<T, U> TimeFn for StepAt<T, U>
where
    T: TimeFn,
    U: TimeFn<Output = T::Output>,
{
    type Output = T::Output;

    fn at(&self, seconds: Time) -> Self::Output {
        if seconds >= self.at_seconds {
            self.after.at(seconds)
        } else {
            self.before.at(seconds)
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Seconds;

impl TimeFn for Seconds {
    type Output = TimeCompatible;

    fn at(&self, seconds: Time) -> Self::Output {
        seconds
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Const<A> {
    value: A,
}

impl<A> Const<A>
where
    A: Clone,
{
    pub fn new(value: A) -> Self {
        Self { value }
    }
}

impl<A> TimeFn for Const<A>
where
    A: Clone,
{
    type Output = A;

    fn at(&self, _seconds: Time) -> Self::Output {
        self.value.clone()
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Sin;

impl TimeFn for Sin {
    type Output = TimeCompatible;

    fn at(&self, seconds: Time) -> Self::Output {
        seconds.sin()
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Cos;

impl TimeFn for Cos {
    type Output = TimeCompatible;

    fn at(&self, seconds: Time) -> Self::Output {
        seconds.cos()
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Tan;

impl TimeFn for Tan {
    type Output = TimeCompatible;

    fn at(&self, seconds: Time) -> Self::Output {
        seconds.tan()
    }
}
#[derive(Debug, Clone, Copy, Default)]
pub struct Ln;

impl TimeFn for Ln {
    type Output = TimeCompatible;

    fn at(&self, seconds: Time) -> Self::Output {
        seconds.ln()
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Log2;

impl TimeFn for Log2 {
    type Output = TimeCompatible;

    fn at(&self, seconds: Time) -> Self::Output {
        seconds.log2()
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Log10;

impl TimeFn for Log10 {
    type Output = TimeCompatible;

    fn at(&self, seconds: Time) -> Self::Output {
        seconds.log10()
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Log {
    base: TimeCompatible,
}

impl Log {
    pub fn of_base(base: TimeCompatible) -> Self {
        Self { base }
    }
}

impl TimeFn for Log {
    type Output = TimeCompatible;

    fn at(&self, seconds: Time) -> Self::Output {
        seconds.log(self.base)
    }
}

#[derive(Debug, Clone, Error)]
pub enum BadSwitchStep {
    #[error("step point seconds cannot be NaN")]
    Nan,
    #[error("duplicated step point: {0} seconds")]
    Duplicated(Time),
}

#[derive(Debug, Clone)]
pub struct Switch<T> {
    initial_step: T,
    switching_steps: Vec<(Time, T)>,
}

impl<T> Switch<T>
where
    T: TimeFn,
{
    pub fn new(initial_step: T) -> Self {
        Self { initial_step, switching_steps: Vec::new() }
    }

    pub fn new_with_capacity(initial_step: T, capacity: usize) -> Self {
        Self { initial_step, switching_steps: Vec::with_capacity(capacity) }
    }

    pub fn try_step_at(
        mut self,
        at_seconds: Time,
        after: T,
    ) -> Result<Self, BadSwitchStep> {
        if at_seconds.is_nan() {
            Err(BadSwitchStep::Nan)
        } else {
            match self.search(at_seconds) {
                Ok(_) => Err(BadSwitchStep::Duplicated(at_seconds)),
                Err(index) => {
                    self.switching_steps.insert(index, (at_seconds, after));
                    Ok(self)
                },
            }
        }
    }

    pub fn step_at(self, at_seconds: Time, after: T) -> Self {
        self.try_step_at(at_seconds, after).expect("bad step point seconds")
    }

    fn search(&self, seconds: Time) -> Result<usize, usize> {
        self.switching_steps.binary_search_by(|(step_at, _)| {
            step_at.partial_cmp(&seconds).unwrap_or(Ordering::Greater)
        })
    }
}

impl<T> TimeFn for Switch<T>
where
    T: TimeFn,
{
    type Output = T::Output;

    fn at(&self, seconds: Time) -> Self::Output {
        match self.search(seconds) {
            Err(0) => self.initial_step.at(seconds),
            Err(i) => {
                let (_, step) = &self.switching_steps[i - 1];
                step.at(seconds)
            },
            Ok(i) => {
                let (_, step) = &self.switching_steps[i];
                step.at(seconds)
            },
        }
    }
}

pub fn time_fn<F, A>(closure: F) -> TimeClosureFn<F>
where
    F: Fn(Time) -> A,
{
    TimeClosureFn { closure }
}
