// create random methods to test the ser/de code together. if they diverge, we have a bug
// this is not perfect, if they both have the same bug it won't be found, but tha's an ok tradeoff

use crate::classes::{FieldValue, Method};
use rand::{Rng, SeedableRng};
use std::collections::HashMap;

/// Allows the creation of a random instance of that type
pub(crate) trait RandomMethod<R: Rng> {
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

#[test]
fn random_ser_de() {
    const ITERATIONS: usize = 1000;
    let mut rng = rand::rngs::StdRng::from_seed([0; 32]);

    for _ in 0..ITERATIONS {
        let class = Method::random(&mut rng);
        let mut bytes = Vec::new();

        if let Err(err) = super::write::write_method(class.clone(), &mut bytes) {
            eprintln!("{class:#?}");
            eprintln!("{err:?}");
            panic!("Failed to serialize");
        }

        match super::parse_method(&bytes) {
            Ok(parsed) => {
                assert_eq!(class, parsed);
            }
            Err(err) => {
                eprintln!("{class:#?}");
                eprintln!("{bytes:?}");
                eprintln!("{err:?}");
                panic!("Failed to deserialize");
            }
        }
    }
}

#[test]
fn nested_table() {
    let table = HashMap::from([(
        "A".to_string(),
        FieldValue::FieldTable(HashMap::from([(
            "B".to_string(),
            FieldValue::Boolean(true),
        )])),
    )]);
    eprintln!("{table:?}");

    let mut bytes = Vec::new();
    crate::classes::write_helper::table(table.clone(), &mut bytes).unwrap();
    eprintln!("{bytes:?}");

    let (rest, parsed_table) = crate::classes::parse_helper::table(&bytes).unwrap();

    assert!(rest.is_empty());
    assert_eq!(table, parsed_table);
}
