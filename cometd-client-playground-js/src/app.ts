import {adapt} from 'cometd-nodejs-client';
import {CometD, Message} from 'cometd';

const json = function (data: any): string {
    //return JSON.stringify(data, null, 2);
    return JSON.stringify(data);
};

adapt();
const cometd = new CometD();

cometd.configure({
    url: 'http://[::1]:1025/notifications',
});

cometd.handshake(function (message: Message) {
    console.log("Try to do handshake.")
    if (message.successful) {
        console.log("Got handshake: `", json(message), "`.");

        // Subscribe to receive messages from the server.
        cometd.subscribe('/*', function (message: Message) {
            console.log("Got message: `", json(message), "`.");
        });
    }
});
