use crate::{error::TransError, methods::FieldValue};
use amqp_core::methods::{Bit, Long, Longlong, Longstr, Octet, Short, Shortstr, Table, Timestamp};
use anyhow::Context;
use std::io::Write;

pub fn octet<W: Write>(value: Octet, writer: &mut W) -> Result<(), TransError> {
    writer.write_all(&[value])?;
    Ok(())
}

pub fn short<W: Write>(value: Short, writer: &mut W) -> Result<(), TransError> {
    writer.write_all(&value.to_be_bytes())?;
    Ok(())
}

pub fn long<W: Write>(value: Long, writer: &mut W) -> Result<(), TransError> {
    writer.write_all(&value.to_be_bytes())?;
    Ok(())
}

pub fn longlong<W: Write>(value: Longlong, writer: &mut W) -> Result<(), TransError> {
    writer.write_all(&value.to_be_bytes())?;
    Ok(())
}

pub fn bit<W: Write>(value: &[Bit], writer: &mut W) -> Result<(), TransError> {
    // accumulate bits into bytes, starting from the least significant bit in each byte

    // how many bits have already been packed into `current_buf`
    let mut already_filled = 0;
    let mut current_buf = 0u8;

    for &bit in value {
        if already_filled >= 8 {
            writer.write_all(&[current_buf])?;
            current_buf = 0;
            already_filled = 0;
        }

        let new_bit = (u8::from(bit)) << already_filled;
        current_buf |= new_bit;
        already_filled += 1;
    }

    if already_filled > 0 {
        writer.write_all(&[current_buf])?;
    }

    Ok(())
}

pub fn shortstr<W: Write>(value: Shortstr, writer: &mut W) -> Result<(), TransError> {
    let len = u8::try_from(value.len()).context("shortstr too long")?;
    writer.write_all(&[len])?;
    writer.write_all(value.as_bytes())?;

    Ok(())
}

pub fn longstr<W: Write>(value: Longstr, writer: &mut W) -> Result<(), TransError> {
    let len = u32::try_from(value.len()).context("longstr too long")?;
    writer.write_all(&len.to_be_bytes())?;
    writer.write_all(value.as_slice())?;

    Ok(())
}

// this appears to be unused right now, but it could be used in `Basic` things?
#[allow(dead_code)]
pub fn timestamp<W: Write>(value: Timestamp, writer: &mut W) -> Result<(), TransError> {
    writer.write_all(&value.to_be_bytes())?;
    Ok(())
}

pub fn table<W: Write>(table: Table, writer: &mut W) -> Result<(), TransError> {
    let mut table_buf = Vec::new();

    for (field_name, value) in table {
        shortstr(field_name, &mut table_buf)?;
        field_value(value, &mut table_buf)?;
    }

    let len = u32::try_from(table_buf.len()).context("table too big")?;
    writer.write_all(&len.to_be_bytes())?;
    writer.write_all(&table_buf)?;

    Ok(())
}

fn field_value<W: Write>(value: FieldValue, writer: &mut W) -> Result<(), TransError> {
    match value {
        FieldValue::Boolean(bool) => {
            writer.write_all(&[b't', u8::from(bool)])?;
        }
        FieldValue::ShortShortInt(int) => {
            writer.write_all(b"b")?;
            writer.write_all(&int.to_be_bytes())?;
        }
        FieldValue::ShortShortUInt(int) => {
            writer.write_all(&[b'B', int])?;
        }
        FieldValue::ShortInt(int) => {
            writer.write_all(b"U")?;
            writer.write_all(&int.to_be_bytes())?;
        }
        FieldValue::ShortUInt(int) => {
            writer.write_all(b"u")?;
            writer.write_all(&int.to_be_bytes())?;
        }
        FieldValue::LongInt(int) => {
            writer.write_all(b"I")?;
            writer.write_all(&int.to_be_bytes())?;
        }
        FieldValue::LongUInt(int) => {
            writer.write_all(b"i")?;
            writer.write_all(&int.to_be_bytes())?;
        }
        FieldValue::LongLongInt(int) => {
            writer.write_all(b"L")?;
            writer.write_all(&int.to_be_bytes())?;
        }
        FieldValue::LongLongUInt(int) => {
            writer.write_all(b"l")?;
            writer.write_all(&int.to_be_bytes())?;
        }
        FieldValue::Float(float) => {
            writer.write_all(b"f")?;
            writer.write_all(&float.to_be_bytes())?;
        }
        FieldValue::Double(float) => {
            writer.write_all(b"d")?;
            writer.write_all(&float.to_be_bytes())?;
        }
        FieldValue::DecimalValue(scale, long) => {
            writer.write_all(&[b'D', scale])?;
            writer.write_all(&long.to_be_bytes())?;
        }
        FieldValue::ShortString(str) => {
            writer.write_all(b"s")?;
            shortstr(str, writer)?;
        }
        FieldValue::LongString(str) => {
            writer.write_all(b"S")?;
            longstr(str, writer)?;
        }
        FieldValue::FieldArray(array) => {
            writer.write_all(b"A")?;
            let len = u32::try_from(array.len()).context("array too long")?;
            writer.write_all(&len.to_be_bytes())?;

            for element in array {
                field_value(element, writer)?;
            }
        }
        FieldValue::Timestamp(time) => {
            writer.write_all(b"T")?;
            writer.write_all(&time.to_be_bytes())?;
        }
        FieldValue::FieldTable(value) => {
            writer.write_all(b"F")?;
            table(value, writer)?;
        }
        FieldValue::Void => {
            writer.write_all(b"V")?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn pack_few_bits() {
        let bits = [true, false, true];

        let mut buffer = [0u8; 1];
        super::bit(&bits, &mut buffer.as_mut_slice()).unwrap();

        assert_eq!(buffer, [0b00000101])
    }

    #[test]
    fn pack_many_bits() {
        let bits = [
            /* first 8 */
            true, true, true, true, false, false, false, false, /* second 4 */
            true, false, true, true,
        ];

        let mut buffer = [0u8; 2];
        super::bit(&bits, &mut buffer.as_mut_slice()).unwrap();

        assert_eq!(buffer, [0b00001111, 0b00001101]);
    }
}
