mod generated;

use std::collections::HashMap;

pub use generated::*;

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
