/*
This test just sends a message to a new queue.
 */
import { connectAmqp } from './utils/utils.js';

const connection = await connectAmqp();
const channel = await connection.createChannel();

const QUEUE = 'send-queue-352';

await channel.assertQueue(QUEUE);

channel.publish('', QUEUE, Buffer.from('hello'));

console.log('Published message');

await channel.close();
await connection.close();

console.log('Successfully shut down connection');
