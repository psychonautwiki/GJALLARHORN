import * as kafka from 'kafka-node';

import * as bluebird from 'bluebird';
import * as _request from 'request';
import * as redis from 'redis';
import * as _ from 'lodash';

const request = bluebird.promisify(_request.defaults({
    // proxy: 'http://localhost:6152',
    // strictSSL: false
}));

// bluebird.promisifyAll(redis.RedisClient.prototype);

const getJSON = async url => {
    try {
        const xx = await request(url);
        return JSON.parse((xx).body);
    } catch(err) {
        console.error(err);

        return null;
    }
}

const redisClient = bluebird.promisifyAll(redis.createClient()) as any;

const wait = ms => new Promise(res =>
    setTimeout(res, ms)
);

const __KAFKA_CHANNEL__ = 'redis-test-1';

let lastCommit = 0;
let totalCommitted = 0;

const start = process.hrtime();

setInterval(() => {
    const sinceLastCommit = totalCommitted - lastCommit;

    lastCommit = totalCommitted;
    
    console.log('[%s] [%s] [∆ %s] committed', process.hrtime(start).join('; '), totalCommitted, sinceLastCommit);
}, 1000);

(async () => {
    const client = new kafka.KafkaClient({
        kafkaHost: '127.0.0.1:9092'
        // kafkaHost: '35.204.177.64:9092'
    });

    client.on('ready', async () => {
        const producer = new kafka.Producer(client);

        // const consumer = new kafka.Consumer(client, [{
        //     topic: __KAFKA_CHANNEL__
        // }], {});
    
        // consumer.on('message', msg => {
        //     console.log(msg.topic, msg.value);
        // });

        while(true) {
            // const data = _.get(await getJSON('https://www.reddit.com/r/all.json?limit=120&' + Math.random()), 'data.children');
            const data = _.get(await getJSON('https://www.reddit.com/r/all/comments/.json?limit=100&' + Math.random()), 'data.children');
    
            // parseInt(entry_data.id, 36),

            Promise.all(
                data.map(async item => {
                    if ( await redisClient.getAsync( item.data.name ) !== null ) {
                        return;
                    }

                    await redisClient.setAsync( item.data.name, 1, 'EX', 86400 );

                    producer.send([{
                        topic: __KAFKA_CHANNEL__,
                        messages: [
                            JSON.stringify(item)
                        ]
                    }], async error => {
                        if ( error ) {
                            await redisClient.delAsync( item.data.name );
                        }

                        ++totalCommitted;
                    });
                })
            );
            
            await wait(1000);
        }
    });
})();