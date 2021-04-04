
import React, { Component, createRef } from "react";
import { Redirect } from "react-router-dom";

import Dropdown from 'react-bootstrap/Dropdown';

import Navbar from './Navbar';
import RoomSelect from './RoomSelect';

import '../css/Chat.css';

function GetUTCComponents(utcString) {
    let d = new Date(utcString);

    return [d.getHours(), d.getMinutes(), d.getSeconds()];
}

function UtcToTimestamp(utcString, stampType) {
    let components = GetUTCComponents(utcString);
    let timestamp = `${String(components[0]).padStart(2, '0')}:${String(components[1]).padStart(2, '0')}`;

    if (stampType == "full") {
        return timestamp + `:${String(components[2]).padStart(2, '0')}`;
    } else {
        return timestamp;
    }
}

class ChatView extends Component {

    constructor(props) {
        super(props);

        // Will be set to false if we are ever disconnected / does not manage
        // to connect, and set to true when we manage to connect.
        this.connected = false;

        this.messageBox = createRef();

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
            messages: [],
            peerClients: [],
            typingClients: [],
        };

        this.checkWsConnection = this.checkWsConnection.bind(this);

        this.handleExitRoom = this.handleExitRoom.bind(this);
        this.handleJoinRoom = this.handleJoinRoom.bind(this);

        this.addToMessages = this.addToMessages.bind(this);
        this.sendStartedTyping = this.sendStartedTyping.bind(this);
        this.sendStoppedTyping = this.sendStoppedTyping.bind(this);
        this.handleLogout = this.handleLogout.bind(this);
        this.handleMessageInput = this.handleMessageInput.bind(this);
        this.handleWsMessage = this.handleWsMessage.bind(this);
        this.handleSendMessage = this.handleSendMessage.bind(this);
        this.renderMessages = this.renderMessages.bind(this);
        this.renderPeers = this.renderPeers.bind(this);
        this.renderTypingClients = this.renderTypingClients.bind(this);
    }

    // Initial timeout duration for the WebSocket.
    webSocketTimeout = 250;

    componentDidMount() {
        // If we are about to redirect to the login page there is no need
        // to connect to the WebSocket yet.
        if (!this.state.redirectToLogin) this.connectToWs();
    }

    componentDidUpdate() {
        // Scroll the message box to the bottom.
        if (this.messageBox.current) {
            this.messageBox.current.scrollTo({ top: this.messageBox.current.scrollHeight, behavior: 'smooth' })
        }
    }

    connectToWs() {
        let ws = new WebSocket("ws://192.168.1.85:8080/ws/");
        let that = this;
        var connectInterval;

        ws.onopen = () => {
            this.connected = true;
            this.setState({ ws: ws });

            that.webSocketTimeout = 250;
            clearTimeout(connectInterval);

            // If we already have a username and the user has selected a room,
            // we immediately send the join room message and stay on our view.
            if (this.state.username != "" && this.state.selectedRoom != "") {
                this.handleJoinRoom(this.state.selectedRoom);

                // Add reconnected message.
                this.addToMessages({
                    type: "info",
                    info: "Reconnected.",
                })
            }
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
                this.addToMessages({
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

        // Reset the selected room and any received messages in that room.
        this.setState({ selectedRoom: "", messages: [] });

        // Send the leave status to the server.
        this.state.ws.send(JSON.stringify({
            type: "leave",
        }));
    }

    handleJoinRoom(roomId) {
        this.state.ws.send(JSON.stringify({
            type: "join",
            payload: {
                username: this.state.username,
                room: roomId,
            }
        }));

        this.setState({ selectedRoom: roomId });
    }

    addToMessages(message) {
        this.setState({
            messages: [...this.state.messages, message]
        });
    }

    sendStartedTyping() {
        this.state.ws.send(JSON.stringify({
            type: "typing",
            payload: "started",
        }));
    }

    sendStoppedTyping() {
        this.state.ws.send(JSON.stringify({
            type: "typing",
            payload: "stopped",
        }));
    }

    handleMessageInput(event) {
        event.preventDefault();

        if (event.key === "Enter") {
            // Send the message.
            this.handleSendMessage(event);

        } else {
            if (this.state.currentMessage === "") {
                this.sendStartedTyping();
            } else if (event.target.value === "") {
                this.sendStoppedTyping();
            }

            this.setState({ currentMessage: event.target.value });
        }
    }

    handleWsMessage(event) {
        try {
            let data = JSON.parse(event.data);

            switch (data.type) {
                case "rooms":
                    this.setState({ rooms: data.payload.rooms });
                    break;

                // The client has successfully joined a chatroom.
                case "joined":
                    // Update the state with the peer clients and reset any received messages.
                    this.setState({
                        peerClients: data.payload.others.filter(user => user.name !== this.state.username),
                        typingClients: data.payload.typing,
                        messages: [],
                    });

                    data.payload.messages.forEach(message => {
                        let { createdAt, user, body } = message;

                        this.addToMessages({
                            type: "message",
                            full_timestamp: UtcToTimestamp(createdAt, "full"),
                            timestamp: UtcToTimestamp(createdAt, "normal"),
                            sender: user.name,
                            message: body,
                        });

                    });

                    let userCount = data.payload.others.length;

                    this.addToMessages({
                        type: "info",
                        info: `You have entered the room. There ${userCount > 1 ? "are" : "is"} ${data.payload.others.length} ${userCount > 1 ? "users" : "user"} in the chatroom: ${data.payload.others.map(e => e.name).join(", ")}`,
                    });
                    break;

                // Another client has joined the chatroom.
                case "user-joined":
                    this.addToMessages({
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
                    this.addToMessages({
                        type: "message",
                        full_timestamp: UtcToTimestamp(createdAt, "full"),
                        timestamp: UtcToTimestamp(createdAt, "normal"),
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

                    this.addToMessages({
                        type: "info",
                        info: `${name} left the chatroom.`,
                    })
                    break;

                case "user-typing":
                    let { status } = data.payload;

                    if (status === "started") {
                        this.setState({ typingClients: [...this.state.typingClients, data.payload.user] });
                    } else if (status === "stopped") {
                        this.setState({ typingClients: this.state.typingClients.filter(user => user.id !== data.payload.user.id) });
                    }
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

        // Send to the server that we've stopped typing.
        this.sendStoppedTyping();

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

        return this.state.messages.map(message => {
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

    renderTypingClients() {
        let usernames = this.state.typingClients.map(client => client.name);

        if (usernames.length > 5) {
            return "Multiple people are typing...";
        } else if (usernames.length > 1) {
            return usernames.join(" and ") + " are typing...";
        } else if (usernames.length === 1) {
            return usernames.join("") + " is typing...";
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

                    <div id="message-box" ref={this.messageBox}>
                        {this.renderMessages()}
                    </div>

                    {this.state.typingClients.length > 0 && <div className="typing-clients">{this.renderTypingClients()}</div>}

                    <form id="message-form" className="d-flex bg-light">
                        <input id="message-input" className="form-control me-2" type="text" placeholder="Enter message.." aria-label="message" value={this.state.currentMessage} onChange={this.handleMessageInput} />
                        <button id="message-send-btn" className="btn btn-primary" type="submit" onClick={this.handleSendMessage}>Send</button>
                    </form>
                </div >
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