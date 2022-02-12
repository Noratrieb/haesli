use crate::classes::generated::parse::IResult;
use crate::classes::generated::{
    Bit, Long, Longlong, Longstr, Octet, Short, Shortstr, Table, Timestamp,
};
use crate::error::{ConException, ProtocolError, TransError};
use nom::error::ErrorKind;

impl nom::error::ParseError<&[u8]> for TransError {
    fn from_error_kind(_input: &[u8], _kind: ErrorKind) -> Self {
        ProtocolError::ConException(ConException::SyntaxError).into()
    }

    fn append(_input: &[u8], _kind: ErrorKind, other: Self) -> Self {
        other
    }
}

#[macro_export]
macro_rules! fail {
    () => {
        return Err(nom::Err::Failure(
            crate::error::ProtocolError::ConException(crate::error::ConException::SyntaxError)
                .into(),
        ))
    };
}

pub use fail;

pub fn octet(input: &[u8]) -> IResult<Octet> {
    todo!()
}
pub fn short(input: &[u8]) -> IResult<Short> {
    todo!()
}
pub fn long(input: &[u8]) -> IResult<Long> {
    todo!()
}
pub fn longlong(input: &[u8]) -> IResult<Longlong> {
    todo!()
}
pub fn bit(input: &[u8], amount: u8) -> IResult<Vec<Bit>> {
    todo!()
}
pub fn shortstr(input: &[u8]) -> IResult<Shortstr> {
    todo!()
}
pub fn longstr(input: &[u8]) -> IResult<Longstr> {
    todo!()
}
pub fn timestamp(input: &[u8]) -> IResult<Timestamp> {
    todo!()
}
pub fn table(input: &[u8]) -> IResult<Table> {
    todo!()
}
