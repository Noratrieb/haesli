type Octet = u8;
type PeerProperties = ();
type LongStr = String;

pub enum Connection {
    Start {
        version_major: Option<u8>,
        version_minor: Option<u8>,
        server_properties: PeerProperties,
        mechanisms: LongStr,
        locales: LongStr,
    },
    StartOk,
    Secure,
    SecureOk,
    Tune,
    TuneOk,
    Open,
    OpenOk,
    Close,
    CloseOk,
    Blocked,
    Unblocked,
}
