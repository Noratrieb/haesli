use crate::classes::{FieldValue, Method};
use crate::frame::FrameType;
use crate::{classes, frame};
use std::collections::HashMap;

#[tokio::test]
async fn write_start_ok_frame() {
    let mut payload = Vec::new();
    let method = Method::ConnectionStart {
        version_major: 0,
        version_minor: 9,
        server_properties: HashMap::from([(
            "product".to_string(),
            FieldValue::LongString("no name yet".into()),
        )]),
        mechanisms: "PLAIN".into(),
        locales: "en_US".into(),
    };

    classes::write::write_method(method, &mut payload).unwrap();

    let frame = frame::Frame {
        kind: FrameType::Method,
        channel: 0,
        payload,
    };

    let mut output = Vec::new();

    frame::write_frame(&frame, &mut output).await.unwrap();

    #[rustfmt::skip]
    let expected = [
        /* type, octet, method */
        1u8,
        /* channel, short */
        0, 0,
        /* size, long */
        /* count all the bytes in the payload, 33 here */
        0, 0, 0, 52,
        /* payload */
        /* class-id, short, connection */
        0, 10,
        /* method-id, short, start */
        0, 10,
        /* version-major, octet */
        0,
        /* version-minor, octet */
        9,
        /* server-properties, table */
          /* table-size, long (actual byte size) */
          0, 0, 0, 24,
          /* table-items */
          /* name ("product"), shortstr */
          /* len (7) ; bytes */
          7, b'p', b'r', b'o', b'd', b'u', b'c', b't',
          /* value, a shortstr ("no name yet") here */
             /* tag (s) ; len (11) ; data */
             b'S', 0, 0, 0, 11, b'n', b'o', b' ', b'n', b'a', b'm', b'e', b' ', b'y', b'e', b't',
        /* mechanisms, longstr */
        /* str-len, long ; len 5 ; data ("PLAIN") */
        0, 0, 0, 5,
        b'P', b'L', b'A', b'I', b'N',
        /* locales, longstr */
        /* str-len, long ; len 5 ; data ("en_US") */
        0, 0, 0, 5,
        b'e', b'n', b'_', b'U', b'S',
        /* frame-end */
        0xCE,
    ];

    assert_eq!(expected.as_slice(), output.as_slice());
}

#[test]
fn read_start_ok_payload() {
    #[rustfmt::skip]
    let raw_data = [
        /* Connection.Start-Ok */
        0, 10, 0, 11,
        /* field client-properties */
        /* byte size of the table */
        0, 0, 0, 254,
          /* first key of len 7, "product"*/
          7, 112, 114, 111, 100, 117, 99, 116,
          /* value is of type 83 ("S"), long-string */
          /* has length 26 "Pika Python Client Library" */
          83, 0, 0, 0, 26,
          80, 105, 107, 97, 32, 80, 121, 116, 104, 111, 110, 32, 67, 108, 105, 101, 110, 116, 32, 76, 105, 98, 114, 97, 114, 121,
          /* second key of len 8, "platform" */
          8, 112, 108, 97, 116, 102, 111, 114, 109,
          /* value is of type 83("S"), long-string */
          /* has length 13, "Python 3.8.10" */
          83, 0, 0, 0, 13,
          80, 121, 116, 104, 111, 110, 32, 51, 46, 56, 46, 49, 48,
          /* third key has len 12 "capabilities" */
          12, 99, 97, 112, 97, 98, 105, 108, 105, 116, 105, 101, 115,
          /* type is 70 F (table), with byte-len of 111 */
          70, 0, 0, 0, 111,
            /* first key has length 28, "authentication_failure_close" */
            28, 97, 117, 116, 104, 101, 110, 116, 105, 99, 97, 116, 105, 111, 110, 95, 102, 97, 105, 108, 117, 114, 101, 95, 99, 108, 111, 115, 101,
            /* value of type 116, "t", boolean, true */
            116, 1,
            /* second key has length 10, "basic.nack" */
            10, 98, 97, 115, 105, 99, 46, 110, 97, 99, 107,
            /* value of type 116, "t", boolean, true */
            116, 1,
            /* third key has length 18 "connection.blocked" */
            18, 99, 111, 110, 110, 101, 99, 116, 105, 111, 110, 46, 98, 108, 111, 99, 107, 101, 100,
            /* value of type 116, "t", boolean, true */
            116, 1,
            /* fourth key has length 22 "consumer_cancel_notify" */
            22, 99, 111, 110, 115, 117, 109, 101, 114, 95, 99, 97, 110, 99, 101, 108, 95, 110, 111, 116, 105, 102, 121,
            /* value of type 116, "t", boolean, true */
            116, 1,
            /* fifth key has length 18 "publisher_confirms" */
            18, 112, 117, 98, 108, 105, 115, 104, 101, 114, 95, 99, 111, 110, 102, 105, 114, 109, 115,
            /* value of type 116, "t", boolean, true */
            116, 1,
          /* sixth key has length 11 "information" */
          11, 105, 110, 102, 111, 114, 109, 97, 116, 105, 111, 110,
          /* value of type 83, "S" long-str ; len 24 ; data "See http://pika.rtfd.org" */
          83, 0, 0, 0, 24,
          83, 101, 101, 32, 104, 116, 116, 112, 58, 47, 47, 112, 105, 107, 97, 46, 114, 116, 102, 100, 46, 111, 114, 103,
          /* seventh key has length 7, "version" */
          7, 118, 101, 114, 115, 105, 111, 110,
          /* value of type 83, "S" long-str ; length 5 ; "1.1.0" */
          83, 0, 0, 0, 5,
          49, 46, 49, 46, 48,
        /* client-properties table ends here */
        /* field mechanism, length 5, "PLAIN" */
        5, 80, 76, 65, 73, 78,
        /* field response, longstr, length 7, "\x00admin\x00" */
        0, 0, 0, 7, 0, 97, 100, 109, 105, 110, 0,
        /* locale, shortstr, len 5 "en_US" */
        5, 101, 110, 95, 85, 83,
    ];

    let method = classes::parse_method(&raw_data).unwrap();

    assert_eq!(
        method,
        Method::ConnectionStartOk {
            client_properties: HashMap::from([
                (
                    "product".to_string(),
                    FieldValue::LongString("Pika Python Client Library".into())
                ),
                (
                    "platform".to_string(),
                    FieldValue::LongString("Python 3.8.10".into())
                ),
                (
                    "capabilities".to_string(),
                    FieldValue::FieldTable(HashMap::from([
                        (
                            "authentication_failure_close".to_string(),
                            FieldValue::Boolean(true)
                        ),
                        ("basic.nack".to_string(), FieldValue::Boolean(true)),
                        ("connection.blocked".to_string(), FieldValue::Boolean(true)),
                        (
                            "consumer_cancel_notify".to_string(),
                            FieldValue::Boolean(true)
                        ),
                        ("publisher_confirms".to_string(), FieldValue::Boolean(true)),
                    ]))
                ),
                (
                    "information".to_string(),
                    FieldValue::LongString("See http://pika.rtfd.org".into())
                ),
                (
                    "version".to_string(),
                    FieldValue::LongString("1.1.0".into())
                )
            ]),
            mechanism: "PLAIN".to_string(),
            response: "\x00admin\x00".into(),
            locale: "en_US".to_string()
        }
    );
}
