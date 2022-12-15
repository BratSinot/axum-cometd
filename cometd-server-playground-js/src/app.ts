import * as cometd from 'cometd-nodejs-server';
import * as express from 'express';
import {Server as HttpServer} from 'http';
import * as http from 'http';

const json = function (data: any): string {
    //return JSON.stringify(data, null, 2);
    return JSON.stringify(data);
};

const createServer = (app): HttpServer => {
    app.locals.protocol = 'http';

    let server = http.createServer(app);
    server.keepAliveTimeout = 70000;
    server.headersTimeout = 120000;

    return server;
};

const app = express();

const cometdServer = cometd.createCometDServer({
    timeout: 20000,
    maxInterval: 60000,
    duplicateMetaConnectHttpResponseCode: 409 //GAPI-18907
});

const topic0 = cometdServer.createServerChannel('/topic0');
topic0.addListener('message', function (session, channel, message, callback) {
    /*console.log("Got message: ", {
        "session": session,
        "channel": channel.name,
        "message": message,
    });*/

    // Invoke the callback to signal that handling is complete.
    callback();
});

const topic1 = cometdServer.createServerChannel('/topic1');
topic1.addListener('message', function (session, channel, message, callback) {
    /*console.log("Got message: ", {
        "id": session.id,
        "channel": channel.name,
        "message": message,
    });*/

    // Invoke the callback to signal that handling is complete.
    callback();
});

cometdServer.addListener('sessionAdded', (cometConnection, msg) => {
    console.log(`sessionAdded, (cometConnection: "${json(cometConnection)}", msg: "${msg}")`);
});
cometdServer.addListener('sessionRemoved', (cometConnection, timeout) => {
    console.log(`sessionRemoved, (cometConnection: "${json(cometConnection)}", timeout: ${timeout}`);
});
cometdServer.addListener('channelAdded', (channel) => {
    console.log(`channelAdded, (channel: "${json(channel)}")`);
});

app.use('/notifications', cometdServer.handle);
app.set('port', 1025);

let server = createServer(app);
let port = app.get('port');

server.listen(port, function (): void {
    console.log(`Server running on port ${port}.`);

    //loop();
});

/*function loop() {
    setTimeout(function () {
        console.log("Publish");
        topic0.publish(
            "Vasya",
            {
                "type": "from_server",
                "foo": "bar"
            }
        );
        loop();
    }, 1000)
}*/
