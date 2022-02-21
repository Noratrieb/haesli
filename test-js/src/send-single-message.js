import {connectAmqp} from "./utils/utils.js";

const connection = await connectAmqp();

const channel = await connection.createChannel();

channel.publish('exchange-1', 'queue-1', Buffer.from('hello'));

console.log('Published message');

await channel.close();
await connection.close();

console.log('Successfully shut down connection');
