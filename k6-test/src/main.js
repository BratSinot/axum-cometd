import {
    HOST,
    LOG_ENABLED,
    PORT,
    MAX_VUS,
    MAX_ITERATIONS,
} from "./envs.js";
import {check} from 'k6';
import http from 'k6/http';

function log(msg) {
    if (LOG_ENABLED == true) {
        console.log(`[${__VU}]: ${msg}`);
    }
}

export let options = {
    vus: MAX_VUS,
    iterations: MAX_ITERATIONS,
};

let ID = 0;

export function setup() {
    const CLIENT_IDS = [];
    const url = `http://${HOST}:${PORT}/notifications`;

    log(`Got url ${url}`);

    for (let rcx = 1; rcx <= MAX_VUS; ++rcx) {
        const handshake_response = http.post(
            `${url}/handshake/`,
            JSON.stringify([{
                "id": (ID++).toString(),
                "version": "1.0",
                "minimumVersion": "1.0",
                "channel": "/meta/handshake",
                "supportedConnectionTypes": ["long-polling"],
                "advice": {
                    "timeout": 60000,
                    "interval": 0
                }
            }]),
            {
                headers: {
                    'Content-Type': 'application/json',
                },
            }
        );
        log(`Got handshake_response: \`${handshake_response}\``);
        check(handshake_response, {
            'status is 200': (r) => r.status === 200,
            'successful is true': (r) => r.body && JSON.parse(r.body)[0]["successful"] == true,
        });

        const client_id = JSON.parse(handshake_response.body)[0]["clientId"];

        const subscribe_response = http.post(
            `${url}/`,
            JSON.stringify([{
                "id": (ID++).toString(),
                "channel": "/meta/subscribe",
                "subscription": `/topic${rcx % 2}`,
                "clientId": client_id
            }]),
            {
                headers: {
                    'Content-Type': 'application/json',
                },
            }
        );
        log(`Got subscribe_response: \`${JSON.stringify(subscribe_response)}\``);
        check(subscribe_response, {
            'status is 200': (r) => r.status === 200,
            'successful is true': (r) => r.body && JSON.parse(r.body)[0]["successful"] == true,
        });

        CLIENT_IDS[rcx] = client_id;
    }

    return [url, CLIENT_IDS];
}

export default ([url, CLIENT_IDS]) => {
    const id = (ID++).toString();

    const response = http.post(
        `${url}/connect/`,
        JSON.stringify([{
            "id": id,
            "channel": "/meta/connect",
            "connectionType": "long-polling",
            "advice": {"timeout": 2000},
            "clientId": CLIENT_IDS[__VU]
        }]),
        {
            headers: {
                'Content-Type': 'application/json',
            },
        }
    );

    log(`Got connect_response: \`${JSON.stringify(response)}\``);

    check(response, {
        'status is 200': (r) => r.status === 200,
        'successful is true': (r) => r.body && JSON.parse(r.body).find((json) => json["successful"])["successful"] == true,
        'check channel': (r) => r.body && JSON.parse(r.body).find((json) => json["data"])["data"]["channel"] == `/topic${__VU % 2}`,
        'check msg': (r) => r.body && JSON.parse(r.body).find((json) => json["data"])["data"]["msg"] == `Hello from /topic${__VU % 2}`,
    });
}
