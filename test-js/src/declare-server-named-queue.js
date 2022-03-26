/*
This test creates a queue with an empty name and asserts that the generated
queue name is not empty.
 */

import { assert, connectAmqp } from './utils/utils.js';

const connection = await connectAmqp();

const channel = await connection.createChannel();

const reply = await channel.assertQueue('');

assert(reply.messageCount === 0, 'Message found in queue');
assert(reply.consumerCount === 0, 'Consumer listening on queue');
assert(reply.queue !== '', 'Wrong queue name returned');

console.log(`created queue '${reply.queue}'`);

await channel.close();
await connection.close();
