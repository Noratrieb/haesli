use haesli_core::{
    amqp_todo,
    connection::Channel,
    methods::{ExchangeDeclare, Method},
};

use crate::Result;

pub fn declare(_channel: Channel, _exchange_declare: ExchangeDeclare) -> Result<Method> {
    amqp_todo!()
}
