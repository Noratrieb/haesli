import { assert, connectAmqp } from './utils/utils.js';

const queueName = 'test-queue-124';

const connection = await connectAmqp();

const channel = await connection.createChannel();

const reply = await channel.assertQueue(queueName);

assert(reply.messageCount === 0, 'Message found in queue');
assert(reply.consumerCount === 0, 'Consumer listening on queue');
assert(reply.queue === queueName, 'Wrong queue name returned');

console.log(`created queue '${queueName}'`);

await channel.close();
await connection.close();
