import { connect } from 'amqplib';

export const connectAmqp = async () => {
  return connect(
    {
      protocol: 'amqp',
      hostname: 'localhost',
      port: 5672,
      username: 'admin',
      password: '',
      frameMax: 0,
      channelMax: 1000,
    },
    {}
  );
};

export const assert = (cond, msg) => {
  if (!cond) {
    throw new Error(`Assertion failed: ${msg}`);
  }
};

export const waitForMessage = (channel, queue, message) =>
  new Promise((resolve) => {
    channel
      .consume(queue, (msg) => {
        if (msg.content.toString() === message) {
          console.log(`Received '${message}'!`);
          resolve();
        }
      })
      .then((response) =>
        console.log(
          `Registered consumer, consumerTag: "${response.consumerTag}"`
        )
      );
  });
