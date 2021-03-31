
export default class ConnectionHandler {

    constructor(wsConn) {
        this.ws = wsConn;
    }

    joinServer(username) {
        this.ws.send({
            type: "join",
            payload: {
                username: username,
            }
        })
    }

    sendMessage(message) {
        this.ws.send({
            type: "post",
            payload: {
                message: message,
            }
        })
    }
}