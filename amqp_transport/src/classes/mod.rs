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
pub fn parse_method(payload: &[u8]) -> Result<generated::Class, TransError> {
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

#[cfg(test)]
mod tests {
    #[test]
    fn pack_few_bits() {
        let bits = [true, false, true];

        let mut buffer = [0u8; 2];
        super::write_helper::bit(&bits, &mut buffer.as_mut_slice()).unwrap();

        let (_, parsed_bits) = super::parse_helper::bit(&buffer, 3).unwrap();
        assert_eq!(bits.as_slice(), parsed_bits.as_slice());
    }

    #[test]
    fn pack_many_bits() {
        let bits = [
            /* first 8 */
            true, true, true, true, false, false, false, false, /* second 4 */
            true, false, true, true,
        ];
        let mut buffer = [0u8; 2];
        super::write_helper::bit(&bits, &mut buffer.as_mut_slice()).unwrap();

        let (_, parsed_bits) = super::parse_helper::bit(&buffer, 12).unwrap();
        assert_eq!(bits.as_slice(), parsed_bits.as_slice());
    }
}
