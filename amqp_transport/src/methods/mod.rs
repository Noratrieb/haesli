use crate::error::{ConException, TransError};
use std::collections::HashMap;

mod generated;
mod parse_helper;
#[cfg(test)]
mod tests;
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
pub fn parse_method(payload: &[u8]) -> Result<generated::Method, TransError> {
    let nom_result = generated::parse::parse_method(payload);

    match nom_result {
        Ok(([], class)) => Ok(class),
        Ok((_, _)) => {
            Err(
                ConException::SyntaxError(vec!["could not consume all input".to_string()])
                    .into_trans(),
            )
        }
        Err(nom::Err::Incomplete(_)) => {
            Err(
                ConException::SyntaxError(vec!["there was not enough data".to_string()])
                    .into_trans(),
            )
        }
        Err(nom::Err::Failure(err) | nom::Err::Error(err)) => Err(err),
    }
}