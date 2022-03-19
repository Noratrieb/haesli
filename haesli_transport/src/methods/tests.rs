// create random methods to test the ser/de code together. if they diverge, we have a bug
// this is not perfect, if they both have the same bug it won't be found, but that's an ok tradeoff

use std::collections::HashMap;

use rand::SeedableRng;

use crate::methods::{FieldValue, Method, RandomMethod};

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
    const ITERATIONS: usize = 10000;
    let mut rng = rand::rngs::StdRng::from_seed([0; 32]);

    for _ in 0..ITERATIONS {
        let method = Method::random(&mut rng);
        let mut bytes = Vec::new();

        if let Err(err) = super::write::write_method(&method, &mut bytes) {
            eprintln!("{method:#?}");
            eprintln!("{err:?}");
            panic!("Failed to serialize");
        }

        match super::parse_method(&bytes) {
            Ok(parsed) => {
                if method != parsed {
                    eprintln!("{method:#?}");
                    eprintln!("{bytes:?}");
                    eprintln!("{parsed:?}");
                    panic!("Not equal!");
                }
            }
            Err(err) => {
                eprintln!("{method:#?}");
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
        "A".to_owned(),
        FieldValue::FieldTable(HashMap::from([("B".to_owned(), FieldValue::Boolean(true))])),
    )]);
    eprintln!("{table:?}");

    let mut bytes = Vec::new();
    crate::methods::write_helper::table(&table, &mut bytes).unwrap();
    eprintln!("{bytes:?}");

    let (rest, parsed_table) = crate::methods::parse_helper::table(&bytes).unwrap();

    assert!(rest.is_empty());
    assert_eq!(table, parsed_table);
}
