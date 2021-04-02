
import React, { Component } from "react";
import { Redirect } from "react-router-dom";

import Dropdown from 'react-bootstrap/Dropdown';

import Navbar from './Navbar';
import RoomSelect from './RoomSelect';

import '../css/Chat.css';

function GetUTCComponents(utcString) {
    let d = new Date(utcString);

    return [d.getHours(), d.getMinutes(), d.getSeconds()];
}

class ChatView extends Component {

    constructor(props) {
        super(props);

        // Will be set to false if we are ever disconnected / does not manage
        // to connect, and set to true when we manage to connect.
        this.connected = false;

        let username = "";

        // Check if this.props.loation is defined, and if it is set the username
        // to the props username.
        if (this.props.location.state) {
            username = this.props.location.state.username;
        }

        this.state = {
            username,
            redirectToLogin: username === "", // If the username is blank that means the user must login.
            ws: null,
            rooms: [],
            selectedRoom: "", // The ID of the selected room.
            currentMessage: "",
            receivedMessages: [],
            peerClients: [],
        };

        this.checkWsConnection = this.checkWsConnection.bind(this);

        this.handleExitRoom = this.handleExitRoom.bind(this);
        this.handleJoinRoom = this.handleJoinRoom.bind(this);

        this.addToReceivedMessages = this.addToReceivedMessages.bind(this);
        this.handleLogout = this.handleLogout.bind(this);
        this.handleMessageInput = this.handleMessageInput.bind(this);
        this.handleWsMessage = this.handleWsMessage.bind(this);
        this.handleSendMessage = this.handleSendMessage.bind(this);
        this.renderMessages = this.renderMessages.bind(this);
        this.renderPeers = this.renderPeers.bind(this);
    }

    // Initial timeout duration for the WebSocket.
    webSocketTimeout = 250;

    componentDidMount() {
        // If we are about to redirect to the login page there is no need
        // to connect to the WebSocket yet.
        if (!this.state.redirectToLogin) this.connectToWs();
    }

    connectToWs() {
        let ws = new WebSocket("ws://localhost:8080/ws/");
        let that = this;
        var connectInterval;

        ws.onopen = () => {
            this.connected = true;
            this.setState({ ws: ws });

            that.webSocketTimeout = 250;
            clearTimeout(connectInterval);
        }

        ws.onclose = e => {
            console.log(
                `Socket is closed. Reconnect will be attempted in ${Math.min(
                    10000 / 1000,
                    (that.webSocketTimeout + that.webSocketTimeout) / 1000
                )} second.`,
                e.reason
            );

            // Increment retry.
            that.webSocketTimeout = that.webSocketTimeout + that.webSocketTimeout;

            connectInterval = setTimeout(this.checkWsConnection, Math.min(10000, that.webSocketTimeout));
        }

        ws.onerror = err => {

            if (this.connected === true) {
                // Add to user messages that we have been disconnected and are trying to reconnect.
                this.addToReceivedMessages({
                    type: "info",
                    info: "Disconnected from server. Trying to reconnect..."
                });

                this.connected = false;
            }

            console.error(
                "Socket encountered error: ",
                err.message,
                "Closing ocket."
            );

            ws.close();
        }

        ws.onmessage = this.handleWsMessage;
    }

    // Checks whether we are connected to the WebSocket, and if not, try to
    // reconnect.
    checkWsConnection() {
        const { ws } = this.state;
        if (!ws || ws.readyState === WebSocket.CLOSED) this.connectToWs()
    }

    handleLogout(event) {
        event.preventDefault();

        // Remove the onclose event and close the WebSocket connection.
        this.state.ws.onclose = function () { };
        this.state.ws.close();

        this.setState({ redirectToLogin: true, username: "", ws: null });
    }

    handleExitRoom(event) {
        event.preventDefault();

        this.setState({ selectedRoom: "" })
        this.state.ws.send(JSON.stringify({
            type: "leave",
        }));
    }

    handleJoinRoom(roomId) {
        console.log("Joining room: " + roomId);

        this.state.ws.send(JSON.stringify({
            type: "join",
            payload: {
                username: this.state.username,
                room: roomId,
            }
        }));

        this.setState({ selectedRoom: roomId });
    }

    addToReceivedMessages(message) {
        this.setState({
            receivedMessages: [...this.state.receivedMessages, message]
        });
    }

    handleMessageInput(event) {
        event.preventDefault();

        if (event.key === "Enter") {
            // Send the message.
            this.handleSendMessage(event);
        } else {
            this.setState({ currentMessage: event.target.value });
        }
    }

    handleWsMessage(event) {
        try {
            let data = JSON.parse(event.data);

            switch (data.type) {
                case "rooms":

                    data.payload.rooms.sort((a, b) => {
                        a = Number(a.name.replace("Room ", ""))
                        b = Number(b.name.replace("Room ", ""))

                        if (a < b) {
                            return -1;
                        } else if (a > b) {
                            return 1;
                        } else {
                            return 0;
                        }
                    });

                    this.setState({ rooms: data.payload.rooms });
                    console.log(this.state.rooms[0]);
                    break;

                // The client has successfully joined a chatroom.
                case "joined":

                    console.log(data);
                    // Update the state with the peer clients and reset any received messages.
                    this.setState({
                        peerClients: data.payload.others.filter(user => user.name !== this.state.username),
                        receivedMessages: [],
                    });

                    data.payload.messages.forEach(message => {
                        let { createdAt, user, body } = message;
                        let timeComponents = GetUTCComponents(createdAt);
                        let timestamp = `${String(timeComponents[0]).padStart(2, '0')}:${String(timeComponents[1]).padStart(2, '0')}`;
                        let full_timestamp = timestamp + `:${String(timeComponents[2]).padStart(2, '0')}`;

                        this.addToReceivedMessages({
                            type: "message",
                            full_timestamp: full_timestamp,
                            timestamp: timestamp,
                            sender: user.name,
                            message: body,
                        });

                    });

                    this.addToReceivedMessages({
                        type: "info",
                        info: `You have now entered the chatroom. There are ${data.payload.others.length} ${data.payload.others.length > 1 ? "users" : "user"} in the chatroom: ${data.payload.others.map(e => e.name).join(", ")}`,
                    });

                    break;

                // Another client has joined the chatroom.
                case "user-joined":
                    this.addToReceivedMessages({
                        type: "info",
                        info: `${data.payload.user.name} has joined the chatroom!`,
                    });

                    // Add the client to the peer list.
                    this.setState({ peerClients: [...this.state.peerClients, data.payload.user] });
                    break;

                // A message has been received.
                case "posted":
                case "user-posted":
                    let { createdAt, user, body } = data.payload.message;
                    let timeComponents = GetUTCComponents(createdAt);
                    let timestamp = `${String(timeComponents[0]).padStart(2, '0')}:${String(timeComponents[1]).padStart(2, '0')}`;
                    let full_timestamp = timestamp + `:${String(timeComponents[2]).padStart(2, '0')}`;

                    this.addToReceivedMessages({
                        type: "message",
                        full_timestamp: full_timestamp,
                        timestamp: timestamp,
                        sender: user.name,
                        message: body,
                    });
                    break;

                // A user left the chatroom.
                case "user-left":

                    // Username and id of the client who left.
                    let { name, id } = data.payload.user;

                    // Remove the peer from the peer clients list.
                    this.setState({
                        peerClients: this.state.peerClients.filter(client => client.id !== id)
                    });

                    this.addToReceivedMessages({
                        type: "info",
                        info: `${name} left the chatroom.`,
                    })
                    break;

                default:
                    console.log(data);
            }
        } catch (e) {
            console.log(`Failed to parse received data as JSON: ${event.data}`);
        }
    }

    handleSendMessage(event) {
        event.preventDefault();

        // TODO: Handle not connected properly. Maybe add state variable 
        // "connected"?
        if (!this.state.ws) {
            alert("Not connected to WebSocket.");
            return;
        }

        // Send the message over the WebSocket to the server.
        this.state.ws.send(JSON.stringify({
            type: "post",
            payload: {
                sender: this.state.username,
                message: this.state.currentMessage
            }
        }));

        // When we're done, clear the message from the input.
        this.setState({ currentMessage: "" });
    }

    renderMessages() {
        let key = 0;

        return this.state.receivedMessages.map(message => {
            key++;

            if (message.type === "info") {
                return (
                    <p className="message message-info" key={key}>{message.info}</p>
                );
            } else {
                return (
                    <p className="message" key={key}><span title={message.full_timestamp}>[{message.timestamp}]</span> {message.sender}: {message.message}</p>
                );
            }
        });
    }

    renderPeers() {
        let key = 0;

        if (this.state.peerClients.length === 0) {
            return (
                <Dropdown.Item>No other users in your room.</Dropdown.Item>
            )
        } else {
            return this.state.peerClients.map(client => {
                key++;

                return (
                    <Dropdown.Item href="" key={key}>{client.name}</Dropdown.Item>
                );
            })
        }
    }

    render() {
        // Redirect back to the login view if the username isn't defined.
        if (this.state.redirectToLogin) {
            return <Redirect to="/" />
        }

        // Render this if the user has selected a chatroom.
        if (this.state.selectedRoom !== "") {
            return (
                <div className="chat-box">
                    <Navbar username={this.state.username} connected={true} handleLogout={this.handleLogout} renderPeers={this.renderPeers} exitRoom={this.handleExitRoom} />

                    <div id="message-box">
                        {this.renderMessages()}
                    </div>

                    <form id="message-form" className="d-flex">
                        <input id="message-input" className="form-control me-2" type="text" placeholder="Enter message.." aria-label="message" value={this.state.currentMessage} onChange={this.handleMessageInput} />
                        <button id="message-send-btn" className="btn btn-primary" type="submit" onClick={this.handleSendMessage}>Send message</button>
                    </form>
                </div>
            );
        } else {
            return (
                <div className="chat-box">
                    <Navbar username={this.state.username} connected={false} handleLogout={this.handleLogout} renderPeers={this.renderPeers} exitRoom={this.handleExitRoom} />

                    <RoomSelect rooms={this.state.rooms} handleJoinRoom={this.handleJoinRoom} />
                </div>
            );
        }
    }
}

export default ChatView;