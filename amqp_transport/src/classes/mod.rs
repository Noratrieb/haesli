mod connection;

use crate::classes::connection::Connection;

pub enum Class {
    Connection(Connection),
}
