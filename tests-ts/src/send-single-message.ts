import {connect} from 'amqplib';

(async () => {
    const connection = await connect('amqp://localhost');

    const channel = await connection.createChannel();

    channel.publish('exchange-1', 'queue-1', Buffer.from('hello'));

    console.log("Hello world!");
})()
