#![feature(try_trait_v2)]
#![feature(never_type)]
use core::ops::ControlFlow;
use core::ops::FromResidual;
use core::ops::Try;
use std::process::Termination;

pub type Location = std::panic::Location<'static>;

#[derive(Default)]
pub struct ReturnTrace(Vec<Location>);
impl ReturnTrace {
    pub fn push(&mut self, location: Location) {
        self.0.push(location)
    }
}
impl std::fmt::Debug for ReturnTrace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self.0)
    }
}

#[derive(Debug)]
pub struct Traced<E>(E, ReturnTrace);

#[derive(Debug)]
pub enum Trace<T, E> {
    Ok(T),
    Err(E, ReturnTrace),
}
impl<T, E> Trace<T, E> {
    /// make a new error with an empty trace
    pub fn err(e: E) -> Self {
        Self::Err(e, Default::default())
    }

    /// make a new error, recording the instantiation site
    #[track_caller]
    pub fn err_here(e: E) -> Self {
        Trace::err(e)?
    }

    /// convert to the equivalent result, but prevent future tracing
    fn as_result(self) -> Result<T, Traced<E>> {
        self.into()
    }

    /// add a cause trace to an existing error
    pub fn caused_by(mut self, t: ReturnTrace) -> Self {
        t.caused(&mut self);
        self
    }
}

impl<T, E> From<Trace<T, E>> for Result<T, Traced<E>> {
    fn from(value: Trace<T, E>) -> Self {
        match value {
            Trace::Ok(o) => Ok(o),
            Trace::Err(e, t) => Err(Traced(e, t)),
        }
    }
}

trait Caused<T> {
    fn caused(self, other: &mut T);
}
impl Caused<Self> for ReturnTrace {
    fn caused(mut self, other: &mut Self) {
        // put the cause first
        std::mem::swap(&mut self, other);
        // put the rest of the trace after
        other.0.extend(self.0);
    }
}
impl<T, E> Caused<Trace<T, E>> for ReturnTrace {
    fn caused(self, other: &mut Trace<T, E>) {
        if let Trace::Err(_, ref mut t) = other {
            self.caused(t);
        }
    }
}

impl<T, E> Try for Trace<T, E> {
    type Output = T;
    type Residual = Trace<!, E>;
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Self::Ok(o) => ControlFlow::Continue(o),
            Self::Err(e, t) => ControlFlow::Break(Trace::Err(e, t)),
        }
    }
    fn from_output(o: Self::Output) -> Self {
        Self::Ok(o)
    }
}

impl<T, E, F: From<E>> FromResidual<Trace<!, E>> for Trace<T, F> {
    #[track_caller]
    fn from_residual(r: Trace<!, E>) -> Self {
        match r {
            Trace::Err(e, mut t) => {
                // trace the ?-return of the error
                t.push(*Location::caller());
                Self::Err(e.into(), t)
            }
            // satisfy the compiler that the match is definitely exhaustive
            Trace::Ok(never) => match never {},
        }
    }
}

impl<T, E, F: From<E>> FromResidual<Trace<!, E>> for Result<T, Traced<F>> {
    #[track_caller]
    fn from_residual(r: Trace<!, E>) -> Self {
        Trace::from_residual(r).as_result()
    }
}

impl<T, E> Termination for Trace<T, E>
where
    T: Termination,
    E: std::fmt::Debug,
{
    fn report(self) -> std::process::ExitCode {
        self.as_result().report()
    }
}
