use crate::classes::FieldValue;
use crate::frame::FrameType;
use crate::{classes, frame};
use std::collections::HashMap;

#[tokio::test]
async fn write_start_frame() {
    let mut payload = Vec::new();
    let method = classes::Class::Connection(classes::Connection::Start {
        version_major: 0,
        version_minor: 9,
        server_properties: HashMap::from([(
            "version".to_string(),
            FieldValue::ShortString("0.1.0".to_string()),
        )]),
        mechanisms: vec![],
        locales: vec![],
    });

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
        /* type, octet */
        1u8, // = method
        /* channel, short */
        0, 0,
        /* size, long */
        /* count all the bytes in the payload, 33 here */
        0, 0, 0, 33,
        /* payload */

        /* class-id, short */
        0, 10, // connection
        /* method-id, short */
        0, 10, // start
        /* version-major, octet */
        0,
        /* version-minor, octet */
        9,
        /* server-properties, table */
          /* table-size, long (actual byte size) */
          0, 0, 0, 15,
          /* table-items */
          /* name ("version"), shortstr */
          /* len (7) ; bytes */
          7, b'v', b'e', b'r', b's', b'i', b'o', b'n',
          /* value, a shortstr ("0.1.0") here */
             /* tag (s) ; len (5) ; data */
             b's', 5, b'0', b'.', b'1', b'.', b'0',
        /* mechanisms, longstr */
        /* str-len, long ; data (none here) */
        0, 0, 0, 0,  
        /* locales, longstr */
        /* str-len, long ; data (none here) */
        0, 0, 0, 0,

        /* frame-end */
        0xCE,
    ];

    assert_eq!(expected.as_slice(), output.as_slice());
}

#[tokio::test]
async fn read_start_ok_payload() {
    // comes from a python pika amqp client - can assumed to be valid
    // annotated manually
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
          80, 105, 107, 97, 32, 80, 121, 116, 104, 111, 110, 32, 67, 108, 105, 101, 110, 116, 32, 76, 105, 98,
          114, 97, 114, 121,
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
          /* unsure after this */
          /* sixth key has length 11 "information" */
          11, 105, 110, 102, 111, 114, 109, 97, 116, 105, 111, 110,
          /* value of type 83, "S" long-str ; len 24 */
          83, 0, 0, 0, 24,
/* it gets very very confusing and possibly wrong on my side here */
          /* data "See http://pika.rtf\n\x00.or" */
          83, 101, 101, 32, 104, 116, 116, 112, 58, 47, 47, 112, 105, 107, 97, 46, 114, 116, 102, 10, 0, 46, 111, 114, 103, 7, 118, 101, 114, 115, 105, 111, 110, 83, 0, 0, 0, 5, 49, 46, 49, 46,

        /* table should only end here */

        48, 5, 80, 76, 65, 73, 78, 0, 0, 0, 7, 0, 97, 100, 109, 105, 110, 0,
        /* locale, shortstr, len 5 "en_US" */
        5, 101, 110, 95, 85, 83,
    ];

    classes::parse_method(&raw_data).unwrap();
}
