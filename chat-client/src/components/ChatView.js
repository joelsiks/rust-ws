
import React, { Component } from "react";
import { Redirect } from "react-router-dom";

import Navbar from './Navbar';

import '../css/Chat.css';

class ChatView extends Component {

    constructor(props) {
        super(props);

        let username = "";

        // Check if this.props.loation is defined, and if it is set the username
        // to the props username.
        if (this.props.location.state) {
            username = this.props.location.state.username;
        }

        this.state = {
            username,
            ws: null,
            redirectToLogin: username === "", // If the username is blank that means the user must login.
            currentMessage: "",
            receivedMessages: [],
        };

        this.checkWsConnection = this.checkWsConnection.bind(this);

        this.handleLogout = this.handleLogout.bind(this);
        this.handleMessageInput = this.handleMessageInput.bind(this);
        this.handleWsMessage = this.handleWsMessage.bind(this);
        this.handleSendMessage = this.handleSendMessage.bind(this);
        this.renderMessages = this.renderMessages.bind(this);
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
            console.log("Connected websocket main component.");

            this.setState({ ws: ws });

            that.timeout = 250;
            clearTimeout(connectInterval);
        }

        ws.onclose = e => {
            console.log(
                `Socket is closed. Reconnect will be attempted in ${Math.min(
                    10000 / 1000,
                    (that.timeout + that.timeout) / 1000
                )} second.`,
                e.reason
            );

            // Increment retry.
            that.timeout = that.timeout + that.timeout;

            connectInterval = setTimeout(this.checkWsConnection, Math.min(10000, that.timeout));
        }

        ws.onerror = err => {
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

        this.setState({ redirectToLogin: true, username: "" });
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
        let { sender, message } = JSON.parse(event.data);

        let cd = new Date();
        this.setState({
            receivedMessages: [...this.state.receivedMessages, {
                timestamp: `${cd.getHours()}:${cd.getMinutes()}`,
                sender: sender,
                message: message
            }]
        });
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
            sender: this.state.username,
            message: this.state.currentMessage
        }));

        // When we're done, clear the message from the input.
        this.setState({ currentMessage: "" });
    }

    renderMessages() {
        let key = 0;

        return this.state.receivedMessages.map(message => {
            key++;
            return (
                <p className="message" key={key}>[{message.timestamp}] {message.sender}: {message.message}</p>
            );
        });
    }

    render() {
        // Redirect back to the login view if the username isn't defined.
        if (this.state.redirectToLogin) {
            return <Redirect to="/" />
        }

        return (
            <div className="chat-box">
                <Navbar username={this.state.username} handleLogout={this.handleLogout} />

                <div id="message-box">
                    {this.renderMessages()}
                </div>

                <form id="message-form" className="d-flex">
                    <input id="message-input" className="form-control me-2" type="text" placeholder="Enter message.." aria-label="message" value={this.state.currentMessage} onChange={this.handleMessageInput} />
                    <button id="message-send-btn" className="btn btn-primary" type="submit" onClick={this.handleSendMessage}>Send message</button>
                </form>
            </div>
        );
    }
}

export default ChatView;