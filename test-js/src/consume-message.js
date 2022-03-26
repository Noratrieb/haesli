/*
This test consumes from a queue and then sends a message to it, expecting it to arrive.
 */

import { connectAmqp, waitForMessage } from './utils/utils.js';

const connection = await connectAmqp();
const channel = await connection.createChannel();

const QUEUE = 'consume-queue-1415';

await channel.assertQueue(QUEUE);

const consumer = waitForMessage(channel, QUEUE, 'STOP');

await channel.sendToQueue(QUEUE, Buffer.from('STOP'));

console.log('Sent STOP message to queue');

await consumer;

await channel.close();
await connection.close();
