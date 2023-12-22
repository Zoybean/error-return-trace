use thiserror::Error;
use trace::*;

pub fn main() -> Trace<(), Error> {
    let f = foo(12)?;
    Trace::Ok(f)
}

fn foo(x: i32) -> Trace<(), Error> {
    if x >= 5 {
        Trace::Ok(bar()?)
    } else {
        Trace::Ok(bang2()?)
    }
}

#[derive(Debug, Error)]
#[error("File not found")]
pub struct FileNotFound;
#[derive(Debug, Error)]
#[error("Permission denied")]
pub struct PermissionDenied;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    FileNotFound(#[from] FileNotFound),
    #[error("{0}")]
    PermissionDenied(#[from] PermissionDenied),
}
fn bar() -> Trace<(), Error> {
    match baz() {
        Trace::Ok(()) => Trace::Ok(quux()?),
        Trace::Err(e, t) => match e {
            FileNotFound => Trace::Ok(hello().with_trace(t)?),
        },
    }
}

fn baz() -> Trace<(), FileNotFound> {
    Trace::Ok(bang1()?)
}

fn quux() -> Trace<(), Error> {
    Trace::Ok(bang2()?)
}

fn hello() -> Trace<(), Error> {
    Trace::Ok(bang2()?)
}

fn bang1() -> Trace<(), FileNotFound> {
    Trace::err(FileNotFound)
}

fn bang2() -> Trace<(), PermissionDenied> {
    Trace::err(PermissionDenied)
}
