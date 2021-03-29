
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
        if(this.props.location.state) {
            username = this.props.location.state.username;
        }

        this.state = {
            username,
            redirectToLogin: username === "", // If the username is blank that
                                              // means the user must login.
        };

        this.handleLogout = this.handleLogout.bind(this);
    }

    handleLogout(event) {
        event.preventDefault();

        this.setState({redirectToLogin: true, username: ""});
    }

    render() {
        // Redirect back to the login view if the username isn't defined.
        if(this.state.redirectToLogin) {
            return <Redirect to="/" />
        }

        return (
            <div className="chat-box">
                <Navbar username={this.state.username} handleLogout={this.handleLogout} />

                <div id="message-box">
                    <p className="message">[13:23] Joel: 123</p>
                </div>

                <form id="message-form" className="d-flex">
                    <input id="message-input" className="form-control me-2" type="search" placeholder="Search" aria-label="Search" />
                    <button id="message-send-btn" className="btn btn-primary" type="submit">Send message</button>
                </form>
            </div>
        );
    }
}

export default ChatView;