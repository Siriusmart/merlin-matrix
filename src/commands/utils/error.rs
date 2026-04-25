use std::{error::Error, fmt::Display};

#[allow(unused)]
#[derive(Debug)]
pub struct ErrorMsg(pub String);

impl Display for ErrorMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{self:?}"))
    }
}

impl Error for ErrorMsg {}
