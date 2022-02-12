use crate::classes::generated::{
    Bit, Long, Longlong, Longstr, Octet, Short, Shortstr, Table, Timestamp,
};
use crate::error::TransError;
use std::io;
use std::io::Write;

fn error(e: io::Error) -> TransError {
    TransError::Other(e.into())
}

pub fn octet<W: Write>(value: Octet, writer: &mut W) -> Result<(), TransError> {
    writer.write_all(&[value])?;
    Ok(())
}

pub fn short<W: Write>(value: Short, writer: &mut W) -> Result<(), TransError> {
    todo!()
}

pub fn long<W: Write>(value: Long, writer: &mut W) -> Result<(), TransError> {
    todo!()
}

pub fn longlong<W: Write>(value: Longlong, writer: &mut W) -> Result<(), TransError> {
    todo!()
}

pub fn bit<W: Write>(value: Vec<Bit>, writer: &mut W) -> Result<(), TransError> {
    todo!()
}

pub fn shortstr<W: Write>(value: Shortstr, writer: &mut W) -> Result<(), TransError> {
    todo!()
}

pub fn longstr<W: Write>(value: Longstr, writer: &mut W) -> Result<(), TransError> {
    todo!()
}

pub fn timestamp<W: Write>(value: Timestamp, writer: &mut W) -> Result<(), TransError> {
    todo!()
}

pub fn table<W: Write>(value: Table, writer: &mut W) -> Result<(), TransError> {
    todo!()
}
