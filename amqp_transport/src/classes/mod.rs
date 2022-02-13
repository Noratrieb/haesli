use crate::classes::generated::Class;
use crate::error::{ConException, ProtocolError, TransError};
use std::collections::HashMap;

mod generated;
mod parse_helper;
mod write_helper;

pub type TableFieldName = String;

pub type Table = HashMap<TableFieldName, FieldValue>;

#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    Boolean(bool),
    ShortShortInt(i8),
    ShortShortUInt(u8),
    ShortInt(i16),
    ShortUInt(u16),
    LongInt(i32),
    LongUInt(u32),
    LongLongInt(i64),
    LongLongUInt(u64),
    Float(f32),
    Double(f64),
    DecimalValue(u8, u32),
    ShortString(Shortstr),
    LongString(Longstr),
    FieldArray(Vec<FieldValue>),
    Timestamp(u64),
    FieldTable(Table),
    Void,
}

pub use generated::*;

/// Parses the payload of a method frame into the class/method
pub fn parse_method(payload: &[u8]) -> Result<Class, TransError> {
    let nom_result = generated::parse::parse_method(payload);

    match nom_result {
        Ok(([], class)) => Ok(class),
        Ok((_, _)) => Err(ProtocolError::ConException(ConException::SyntaxError).into()),
        Err(nom::Err::Incomplete(_)) => {
            Err(ProtocolError::ConException(ConException::SyntaxError).into())
        }
        Err(nom::Err::Failure(err) | nom::Err::Error(err)) => Err(err),
    }
}
