use crate::error::{ConException, TransError};
use rand::Rng;
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

/// Parses the payload of a method frame into the method
pub fn parse_method(payload: &[u8]) -> Result<generated::Method, TransError> {
    let nom_result = generated::parse::parse_method(payload);

    match nom_result {
        Ok(([], method)) => Ok(method),
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

/// Allows the creation of a random instance of that type
pub trait RandomMethod<R: Rng> {
    fn random(rng: &mut R) -> Self;
}

impl<R: Rng> RandomMethod<R> for String {
    fn random(rng: &mut R) -> Self {
        let n = rng.gen_range(0_u16..9999);
        format!("string{n}")
    }
}

impl<R: Rng, T: RandomMethod<R>> RandomMethod<R> for Vec<T> {
    fn random(rng: &mut R) -> Self {
        let len = rng.gen_range(1_usize..10);
        let mut vec = Vec::with_capacity(len);
        (0..len).for_each(|_| vec.push(RandomMethod::random(rng)));
        vec
    }
}

macro_rules! rand_random_method {
    ($($ty:ty),+) => {
        $(
             impl<R: Rng> RandomMethod<R> for $ty {
             fn random(rng: &mut R) -> Self {
                rng.gen()
            }
        })+
    };
}

rand_random_method!(bool, u8, i8, u16, i16, u32, i32, u64, i64, f32, f64);

impl<R: Rng> RandomMethod<R> for HashMap<String, FieldValue> {
    fn random(rng: &mut R) -> Self {
        let len = rng.gen_range(0..3);
        HashMap::from_iter((0..len).map(|_| (String::random(rng), FieldValue::random(rng))))
    }
}

impl<R: Rng> RandomMethod<R> for FieldValue {
    fn random(rng: &mut R) -> Self {
        let index = rng.gen_range(0_u32..17);
        match index {
            0 => FieldValue::Boolean(RandomMethod::random(rng)),
            1 => FieldValue::ShortShortInt(RandomMethod::random(rng)),
            2 => FieldValue::ShortShortUInt(RandomMethod::random(rng)),
            3 => FieldValue::ShortInt(RandomMethod::random(rng)),
            4 => FieldValue::ShortUInt(RandomMethod::random(rng)),
            5 => FieldValue::LongInt(RandomMethod::random(rng)),
            6 => FieldValue::LongUInt(RandomMethod::random(rng)),
            7 => FieldValue::LongLongInt(RandomMethod::random(rng)),
            8 => FieldValue::LongLongUInt(RandomMethod::random(rng)),
            9 => FieldValue::Float(RandomMethod::random(rng)),
            10 => FieldValue::Double(RandomMethod::random(rng)),
            11 => FieldValue::ShortString(RandomMethod::random(rng)),
            12 => FieldValue::LongString(RandomMethod::random(rng)),
            13 => FieldValue::FieldArray(RandomMethod::random(rng)),
            14 => FieldValue::Timestamp(RandomMethod::random(rng)),
            15 => FieldValue::FieldTable(RandomMethod::random(rng)),
            16 => FieldValue::Void,
            _ => unreachable!(),
        }
    }
}
