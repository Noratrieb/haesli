import { connect } from 'amqplib';
import { sleep } from './utils.js';

(async () => {
  const connection = await connect('amqp://localhost');

  const channel = await connection.createChannel();

  console.log('Successfully opened channel');

  await sleep(100_000);

  await channel.close();
  await connection.close();

  console.log('Successfully shut down connection');
})();
