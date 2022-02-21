import {connectAmqp, sleep} from './utils/utils.js';

const connection = await connectAmqp();

const channel = await connection.createChannel();

console.log('Successfully opened channel');

await sleep(100_000);

await channel.close();
await connection.close();

console.log('Successfully shut down connection');
