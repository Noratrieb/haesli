use crate::classes::generated::parse::IResult;
use crate::classes::generated::{
    Bit, Long, Longlong, Longstr, Octet, Short, Shortstr, Table, Timestamp,
};
use crate::error::{ConException, ProtocolError, TransError};
use nom::branch::alt;
use nom::bytes::complete::{tag, take};
use nom::error::ErrorKind;
use nom::multi::{count, many0};
use nom::number::complete::{f32, f64, i16, i32, i64, i8, u16, u32, u64, u8};
use nom::number::Endianness::Big;
use nom::Err;
use std::collections::HashMap;

// todo: remove the debug machinery or change it in a way that actually does what it should lmao
// I'm probably misusing nom hard

impl<T> nom::error::ParseError<T> for TransError {
    fn from_error_kind(_input: T, _kind: ErrorKind) -> Self {
        ProtocolError::ConException(ConException::SyntaxError(vec![])).into()
    }

    fn append(_input: T, _kind: ErrorKind, other: Self) -> Self {
        other
    }
}

// todo: make this into fail_err to avoid useless allocations
pub fn err<S: Into<String>>(msg: S) -> impl FnOnce(Err<TransError>) -> Err<TransError> {
    move |err| {
        let error_level = if matches!(err, nom::Err::Failure(_)) {
            Err::Failure
        } else {
            Err::Error
        };

        let msg = msg.into();
        let stack = match err {
            Err::Error(e) | Err::Failure(e) => match e {
                TransError::Invalid(ProtocolError::ConException(ConException::SyntaxError(
                    mut stack,
                ))) => {
                    stack.push(msg);
                    stack
                }
                _ => vec![msg],
            },
            _ => vec![msg],
        };
        error_level(ProtocolError::ConException(ConException::SyntaxError(stack)).into())
    }
}
pub fn err_other<E, S: Into<String>>(msg: S) -> impl FnOnce(E) -> Err<TransError> {
    move |_| {
        Err::Error(ProtocolError::ConException(ConException::SyntaxError(vec![msg.into()])).into())
    }
}

pub fn failure<E>(err: Err<E>) -> Err<E> {
    match err {
        Err::Incomplete(needed) => Err::Incomplete(needed),
        Err::Error(e) => Err::Failure(e),
        Err::Failure(e) => Err::Failure(e),
    }
}

#[macro_export]
macro_rules! fail {
    ($cause:expr) => {
        return Err(nom::Err::Failure(
            crate::error::ProtocolError::ConException(crate::error::ConException::SyntaxError(
                vec![String::from($cause)],
            ))
            .into(),
        ))
    };
}

use crate::classes::{FieldValue, TableFieldName};
pub use fail;

pub fn octet(input: &[u8]) -> IResult<Octet> {
    u8(input)
}

pub fn short(input: &[u8]) -> IResult<Short> {
    u16(Big)(input)
}

pub fn long(input: &[u8]) -> IResult<Long> {
    u32(Big)(input)
}

pub fn longlong(input: &[u8]) -> IResult<Longlong> {
    u64(Big)(input)
}

pub fn bit(input: &[u8], amount: usize) -> IResult<Vec<Bit>> {
    let octets = (amount + 7) / 8;
    let (input, bytes) = take(octets)(input)?;

    let mut vec = Vec::new();
    let mut byte_index = 0;
    let mut total_index = 0;

    for &byte in bytes {
        while byte_index < 8 && total_index < amount {
            let next_bit = 1 & (byte >> byte_index);
            let bit_bool = match next_bit {
                0 => false,
                1 => true,
                _ => unreachable!(),
            };
            vec.push(bit_bool);
            byte_index += 1;
            total_index += 1;
        }
        byte_index = 0;
    }

    Ok((input, vec))
}

pub fn shortstr(input: &[u8]) -> IResult<Shortstr> {
    let (input, len) = u8(input)?;
    let (input, str_data) = take(usize::from(len))(input)?;
    let data = String::from_utf8(str_data.into()).map_err(err_other("shortstr"))?;
    Ok((input, data))
}

pub fn longstr(input: &[u8]) -> IResult<Longstr> {
    let (input, len) = u32(Big)(input)?;
    let (input, str_data) = take(usize::try_from(len).unwrap())(input)?;
    let data = str_data.into();
    Ok((input, data))
}

pub fn timestamp(input: &[u8]) -> IResult<Timestamp> {
    u64(Big)(input)
}

pub fn table(input: &[u8]) -> IResult<Table> {
    let (input, size) = u32(Big)(input)?;
    let (table_input, rest_input) = input.split_at(size.try_into().unwrap());

    let (input, values) = many0(table_value_pair)(table_input)?;

    if input != &[] {
        fail!(format!(
            "table longer than expected, expected = {size}, remaining = {}",
            input.len()
        ));
    }

    let table = HashMap::from_iter(values.into_iter());
    Ok((rest_input, table))
}

fn table_value_pair(input: &[u8]) -> IResult<(TableFieldName, FieldValue)> {
    let (input, field_name) = shortstr(input)?;
    let (input, field_value) = field_value(input).map_err(err(format!("field {field_name}")))?;
    Ok((input, (field_name, field_value)))
}

fn field_value(input: &[u8]) -> IResult<FieldValue> {
    type R<'a> = IResult<'a, FieldValue>;

    fn boolean(input: &[u8]) -> R {
        let (input, _) = tag(b"t")(input)?;
        let (input, bool_byte) = u8(input)?;
        match bool_byte {
            0 => Ok((input, FieldValue::Boolean(false))),
            1 => Ok((input, FieldValue::Boolean(true))),
            value => fail!(format!("invalid bool value {value}")),
        }
    }

    macro_rules! number {
        ($tag:literal, $name:ident, $comb:expr, $value:ident, $r:path) => {
            fn $name(input: &[u8]) -> $r {
                let (input, _) = tag($tag)(input)?;
                $comb(input).map(|(input, int)| (input, FieldValue::$value(int)))
            }
        };
    }

    number!(b"b", short_short_int, i8, ShortShortInt, R);
    number!(b"B", short_short_uint, u8, ShortShortUInt, R);
    number!(b"U", short_int, i16(Big), ShortInt, R);
    number!(b"u", short_uint, u16(Big), ShortUInt, R);
    number!(b"I", long_int, i32(Big), LongInt, R);
    number!(b"i", long_uint, u32(Big), LongUInt, R);
    number!(b"L", long_long_int, i64(Big), LongLongInt, R);
    number!(b"l", long_long_uint, u64(Big), LongLongUInt, R);
    number!(b"f", float, f32(Big), Float, R);
    number!(b"d", double, f64(Big), Double, R);

    fn decimal(input: &[u8]) -> R {
        let (input, _) = tag("D")(input)?;
        let (input, scale) = u8(input)?;
        let (input, value) = u32(Big)(input)?;
        Ok((input, FieldValue::DecimalValue(scale, value)))
    }

    fn short_str(input: &[u8]) -> R {
        let (input, _) = tag("s")(input)?;
        let (input, str) = shortstr(input)?;
        Ok((input, FieldValue::ShortString(str)))
    }

    fn long_str(input: &[u8]) -> R {
        let (input, _) = tag("S")(input)?;
        let (input, str) = longstr(input)?;
        Ok((input, FieldValue::LongString(str)))
    }

    fn field_array(input: &[u8]) -> R {
        let (input, _) = tag("A")(input)?;
        // todo is it i32?
        let (input, len) = u32(Big)(input)?;
        count(field_value, usize::try_from(len).unwrap())(input)
            .map(|(input, value)| (input, FieldValue::FieldArray(value)))
    }

    number!(b"T", timestamp, u64(Big), Timestamp, R);

    fn field_table(input: &[u8]) -> R {
        let (input, _) = tag("F")(input)?;
        table(input).map(|(input, value)| (input, FieldValue::FieldTable(value)))
    }

    fn void(input: &[u8]) -> R {
        tag("V")(input).map(|(input, _)| (input, FieldValue::Void))
    }

    alt((
        boolean,
        short_short_int,
        short_short_uint,
        short_int,
        short_uint,
        long_int,
        long_uint,
        long_long_int,
        long_long_uint,
        float,
        double,
        decimal,
        short_str,
        long_str,
        field_array,
        timestamp,
        field_table,
        void,
    ))(input)
}
