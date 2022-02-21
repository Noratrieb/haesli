//! (Very) partial implementation of SASL Authentication (see [RFC 4422](https://datatracker.ietf.org/doc/html/rfc4422))
//!
//! Currently only supports PLAIN (see [RFC 4616](https://datatracker.ietf.org/doc/html/rfc4616))

use crate::error::{ConException, Result};

pub struct PlainUser {
    pub authorization_identity: String,
    pub authentication_identity: String,
    pub password: String,
}

pub fn parse_sasl_plain_response(response: &[u8]) -> Result<PlainUser> {
    let mut parts = response
        .split(|&n| n == 0)
        .map(|bytes| String::from_utf8(bytes.into()).map_err(|_| ConException::Todo.into_trans()));

    let authorization_identity = parts
        .next()
        .ok_or_else(|| ConException::Todo.into_trans())??;
    let authentication_identity = parts
        .next()
        .ok_or_else(|| ConException::Todo.into_trans())??;
    let password = parts
        .next()
        .ok_or_else(|| ConException::Todo.into_trans())??;

    Ok(PlainUser {
        authorization_identity,
        authentication_identity,
        password,
    })
}
