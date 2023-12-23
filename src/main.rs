use thiserror::Error;
use trace::Trace::{self, Err, Ok};

/// Demonstrate the example from https://ziglang.org/documentation/master/#Error-Return-Traces
pub fn main() -> Trace<(), BarError> {
    Ok(foo(12)?)
}

fn foo(x: i32) -> Trace<(), BarError> {
    if x >= 5 {
        Ok(bar()?)
    } else {
        Ok(bang2()?)
    }
}

#[derive(Debug, Error)]
pub enum BarError {
    #[error("{0}")]
    FileNotFound(#[from] FileNotFound),
    #[error("{0}")]
    PermissionDenied(#[from] PermissionDenied),
}

fn bar() -> Trace<(), BarError> {
    match baz() {
        Ok(()) => Ok(quux()?),
        Err(e, t) => match e {
            FileNotFound => Ok(hello().caused_by(t)?),
        },
    }
}

fn baz() -> Trace<(), FileNotFound> {
    Ok(bang1()?)
}

fn quux() -> Trace<(), PermissionDenied> {
    Ok(bang2()?)
}

fn hello() -> Trace<(), PermissionDenied> {
    Ok(bang2()?)
}

#[derive(Debug, Error)]
#[error("File not found")]
pub struct FileNotFound;

fn bang1() -> Trace<(), FileNotFound> {
    Trace::err_here(FileNotFound)
}

#[derive(Debug, Error)]
#[error("Permission denied")]
pub struct PermissionDenied;

fn bang2() -> Trace<(), PermissionDenied> {
    Trace::err_here(PermissionDenied)
}
