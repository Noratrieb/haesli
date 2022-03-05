import { connectAmqp } from './utils/utils.js';

const connection = await connectAmqp();
const channel = await connection.createChannel();

await channel.assertQueue('consume-queue-1415');

const consumePromise = new Promise((resolve) => {
  channel
    .consume('consume-queue-1415', (msg) => {
      if (msg.content.toString() === 'STOP') {
        resolve();
      }
    })
    .then((response) =>
      console.log(`Registered consumer, consumerTag: "${response.consumerTag}"`)
    );
});

await channel.sendToQueue('consume-queue-1415', Buffer.from('STOP'));
console.log('Sent STOP message to queue');

await consumePromise;

console.log('Received STOP!');

await channel.close();
await connection.close();
