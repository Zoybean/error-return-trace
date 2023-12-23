#![feature(try_trait_v2)]
#![feature(never_type)]
use core::ops::ControlFlow;
use core::ops::FromResidual;
use core::ops::Try;
use std::process::Termination;

#[derive(Debug)]
pub struct Location {
    pub file: String,
    pub line: u32,
    pub column: u32,
}
impl From<std::panic::Location<'_>> for Location {
    fn from(l: std::panic::Location<'_>) -> Self {
        Self {
            file: l.file().to_owned(),
            line: l.line(),
            column: l.column(),
        }
    }
}

#[derive(Default)]
pub struct ReturnTrace(Vec<Location>);
impl ReturnTrace {
    #[track_caller]
    pub fn push_trace(&mut self) {
        let l = *std::panic::Location::caller();
        self.0.push(l.into())
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
    pub fn err(e: E) -> Self {
        Self::Err(e, Default::default())
    }

    fn as_result(self) -> Result<T, Traced<E>> {
        match self {
            Trace::Ok(o) => Ok(o),
            Trace::Err(e, t) => Err(Traced(e, t)),
        }
    }
    pub fn caused_by(mut self, t: ReturnTrace) -> Self {
        t.caused(&mut self);
        self
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
                //
                t.push_trace();
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
