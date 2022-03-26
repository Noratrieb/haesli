/*
This test binds a queue to a fanout exchange and sends a message to that exchange.
It expects the message to arrive at the queue.
 */

import { connectAmqp, waitForMessage } from './utils/utils.js';

const connection = await connectAmqp();
const channel = await connection.createChannel();

const QUEUE = 'exchange-bind-queue-2352';
const EXCHANGE = 'exchange-bind-3152';

await channel.assertQueue(QUEUE);
await channel.assertExchange(EXCHANGE, 'fanout');

await channel.bindQueue(QUEUE, EXCHANGE, '');

await channel.publish(EXCHANGE, '', Buffer.from('STOP'));
console.log('Sent STOP message to queue');

await waitForMessage(channel, QUEUE, 'STOP');

await channel.close();
await connection.close();
