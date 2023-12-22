#![feature(try_trait_v2)]
#![feature(never_type)]
use core::ops::ControlFlow;
use core::ops::FromResidual;
use core::ops::Try;
use std::process::Termination;

// just temporary, bc these details matter the least for this draft
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

#[derive(Default, Debug)]
pub struct ReturnTrace(Vec<Location>);
#[derive(Debug)]
pub struct Traced<E>(E, ReturnTrace);

impl ReturnTrace {
    #[track_caller]
    pub fn append_trace(&mut self) {
        let l = *std::panic::Location::caller();
        self.0.push(l.into())
    }
}

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
        Ok(self?)
    }
    pub fn with_trace(self, t: ReturnTrace) -> Self {
        match self {
            Trace::Ok(o) => Trace::Ok(o),
            Trace::Err(e, mut t2) => {
                t2.0.extend(t.0);
                Trace::Err(e, t2)
            }
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
            Trace::Ok(never) => match never {}, // satisfy the compiler that it is definitely exhaustive
            Trace::Err(e, mut t) => {
                t.append_trace();
                Self::Err(e.into(), t)
            }
        }
    }
}
impl<T, E, F: From<E>> FromResidual<Trace<!, E>> for Result<T, Traced<F>> {
    #[track_caller]
    fn from_residual(r: Trace<!, E>) -> Self {
        match r {
            Trace::Ok(never) => match never {}, // satisfy the compiler that it is definitely exhaustive
            Trace::Err(e, mut t) => {
                t.append_trace();
                Self::Err(Traced(e.into(), t))
            }
        }
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
