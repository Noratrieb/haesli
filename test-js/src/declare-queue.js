/*
This test declares a new queue and expects it to be empty.
 */

import { assert, connectAmqp } from './utils/utils.js';

const QUEUE = 'test-queue-124';

const connection = await connectAmqp();

const channel = await connection.createChannel();

const reply = await channel.assertQueue(QUEUE);

assert(reply.messageCount === 0, 'Message found in queue');
assert(reply.consumerCount === 0, 'Consumer listening on queue');
assert(reply.queue === QUEUE, 'Wrong queue name returned');

console.log(`created queue '${QUEUE}'`);

await channel.close();
await connection.close();
