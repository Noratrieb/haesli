use crate::classes::FieldValue;
use crate::classes::{Bit, Long, Longlong, Longstr, Octet, Short, Shortstr, Table, Timestamp};
use crate::error::Result;
use anyhow::Context;
use std::future::Future;
use std::pin::Pin;
use tokio::io::AsyncWriteExt;

pub async fn octet<W: AsyncWriteExt + Unpin>(value: Octet, writer: &mut W) -> Result<()> {
    writer.write_u8(value).await?;
    Ok(())
}

pub async fn short<W: AsyncWriteExt + Unpin>(value: Short, writer: &mut W) -> Result<()> {
    writer.write_u16(value).await?;
    Ok(())
}

pub async fn long<W: AsyncWriteExt + Unpin>(value: Long, writer: &mut W) -> Result<()> {
    writer.write_u32(value).await?;
    Ok(())
}

pub async fn longlong<W: AsyncWriteExt + Unpin>(value: Longlong, writer: &mut W) -> Result<()> {
    writer.write_u64(value).await?;
    Ok(())
}

pub async fn bit<W: AsyncWriteExt + Unpin>(value: &[Bit], writer: &mut W) -> Result<()> {
    // accumulate bits into bytes, starting from the least significant bit in each byte

    // how many bits have already been packed into `current_buf`
    let mut already_filled = 0;
    let mut current_buf = 0u8;

    for &bit in value {
        if already_filled >= 8 {
            writer.write_u8(current_buf).await?;
            current_buf = 0;
            already_filled = 0;
        }

        let new_bit = (u8::from(bit)) << already_filled;
        current_buf |= new_bit;
        already_filled += 1;
    }

    if already_filled > 0 {
        writer.write_u8(current_buf).await?;
    }

    Ok(())
}

pub async fn shortstr<W: AsyncWriteExt + Unpin>(value: Shortstr, writer: &mut W) -> Result<()> {
    let len = u8::try_from(value.len()).context("shortstr too long")?;
    writer.write_u8(len).await?;
    writer.write_all(value.as_bytes()).await?;

    Ok(())
}

pub async fn longstr<W: AsyncWriteExt + Unpin>(value: Longstr, writer: &mut W) -> Result<()> {
    let len = u32::try_from(value.len()).context("longstr too long")?;
    writer.write_u32(len).await?;
    writer.write_all(value.as_slice()).await?;

    Ok(())
}

pub async fn timestamp<W: AsyncWriteExt + Unpin>(value: Timestamp, writer: &mut W) -> Result<()> {
    writer.write_u64(value).await?;
    Ok(())
}

pub async fn table<W: AsyncWriteExt + Unpin>(table: Table, writer: &mut W) -> Result<()> {
    let len = u32::try_from(table.len()).context("table too big")?;
    writer.write_u32(len).await?;

    for (field_name, value) in table {
        shortstr(field_name, writer).await?;
        field_value(value, writer).await?;
    }

    Ok(())
}

fn field_value<W: AsyncWriteExt + Unpin>(
    value: FieldValue,
    writer: &mut W,
) -> Pin<Box<dyn Future<Output = Result<()>> + '_>> {
    Box::pin(async {
        match value {
            FieldValue::Boolean(bool) => {
                writer.write_all(&[b't', u8::from(bool)]).await?;
            }
            FieldValue::ShortShortInt(int) => {
                writer.write_all(b"b").await?;
                writer.write_all(&int.to_be_bytes()).await?;
            }
            FieldValue::ShortShortUInt(int) => {
                writer.write_all(&[b'B', int]).await?;
            }
            FieldValue::ShortInt(int) => {
                writer.write_all(b"U").await?;
                writer.write_all(&int.to_be_bytes()).await?;
            }
            FieldValue::ShortUInt(int) => {
                writer.write_all(b"u").await?;
                writer.write_all(&int.to_be_bytes()).await?;
            }
            FieldValue::LongInt(int) => {
                writer.write_all(b"I").await?;
                writer.write_all(&int.to_be_bytes()).await?;
            }
            FieldValue::LongUInt(int) => {
                writer.write_all(b"i").await?;
                writer.write_all(&int.to_be_bytes()).await?;
            }
            FieldValue::LongLongInt(int) => {
                writer.write_all(b"L").await?;
                writer.write_all(&int.to_be_bytes()).await?;
            }
            FieldValue::LongLongUInt(int) => {
                writer.write_all(b"l").await?;
                writer.write_all(&int.to_be_bytes()).await?;
            }
            FieldValue::Float(float) => {
                writer.write_all(b"f").await?;
                writer.write_all(&float.to_be_bytes()).await?;
            }
            FieldValue::Double(float) => {
                writer.write_all(b"d").await?;
                writer.write_all(&float.to_be_bytes()).await?;
            }
            FieldValue::DecimalValue(scale, long) => {
                writer.write_all(&[b'D', scale]).await?;
                writer.write_all(&long.to_be_bytes()).await?;
            }
            FieldValue::ShortString(str) => {
                writer.write_all(b"s").await?;
                shortstr(str, writer).await?;
            }
            FieldValue::LongString(str) => {
                writer.write_all(b"S").await?;
                longstr(str, writer).await?;
            }
            FieldValue::FieldArray(array) => {
                writer.write_all(b"A").await?;
                let len = u32::try_from(array.len()).context("array too long")?;
                writer.write_all(&len.to_be_bytes()).await?;

                for element in array {
                    field_value(element, writer).await?;
                }
            }
            FieldValue::Timestamp(time) => {
                writer.write_all(b"T").await?;
                writer.write_all(&time.to_be_bytes()).await?;
            }
            FieldValue::FieldTable(_) => {
                writer.write_all(b"F").await?;
            }
            FieldValue::Void => {
                writer.write_all(b"V").await?;
            }
        }
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn pack_few_bits() {
        let bits = vec![true, false, true];

        let mut buffer = Vec::new();
        super::bit(&bits, &mut buffer).await.unwrap();

        assert_eq!(buffer.as_slice(), &[0b00000101])
    }

    #[tokio::test]
    async fn pack_many_bits() {
        let bits = vec![
            /* first 8 */
            true, true, true, true, false, false, false, false, /* second 4 */
            true, false, true, true,
        ];

        let mut buffer = Vec::new();
        super::bit(&bits, &mut buffer).await.unwrap();

        assert_eq!(buffer, [0b00001111, 0b00001101]);
    }
}
