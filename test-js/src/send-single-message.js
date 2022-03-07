import { connectAmqp } from './utils/utils.js';

const connection = await connectAmqp();
const channel = await connection.createChannel();

await channel.assertQueue('send-queue-352');

channel.publish('', 'send-queue-352', Buffer.from('hello'));

console.log('Published message');

await channel.close();
await connection.close();

console.log('Successfully shut down connection');
