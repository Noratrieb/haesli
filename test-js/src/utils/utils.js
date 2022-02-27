import { connect } from 'amqplib';

export const sleep = (ms) => new Promise((res) => setTimeout(res, ms));

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
