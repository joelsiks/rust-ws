
import React, { Component } from "react";
import { Redirect } from "react-router-dom";

import '../css/Login.css';

const USERNAME_MAX_LENGTH = 32;
const CHAT_REDIRECT_ROUTE = "/chat";

class LoginView extends Component {

    constructor(props) {
        super(props);

        this.state = {
            username: "",
            redirectToChatView: false,
        };

        this.handleUsernameKeyDown = this.handleUsernameKeyDown.bind(this);
        this.handleUsernameChange = this.handleUsernameChange.bind(this);
        this.handleLogin = this.handleLogin.bind(this);
    }

    handleUsernameKeyDown(event) {
        if (event.key === "Enter") {
            this.handleLogin(event);
        }
    }

    handleUsernameChange(event) {
        this.setState({ username: event.target.value });
    }

    handleLogin(event) {
        event.preventDefault();

        // TODO: Add more invalid names?
        if (this.state.username === "" || this.state.username.length > USERNAME_MAX_LENGTH) {
            // TODO: Handle invalid username.
            console.log("Invalid username: " + this.state.username);
        } else {
            // Set redirect status to true to redirect to the Chat View.
            this.setState({ redirectToChatView: true });
        }
    }

    render() {
        if (this.state.redirectToChatView) {
            return <Redirect to={{ pathname: CHAT_REDIRECT_ROUTE, state: { username: this.state.username } }} />
        }

        return (
            <div className="Login">
                <div className="container-fluid" id="login-container">

                        <div className="card align-middle">
                            <div className="card-body">
                                <h2 className="card-title">Welcome to rust-ws 👋🎉</h2>
                                <hr />
                                <h6 className="card-subtitle mb-2 text-muted">Choose a username to enter the chatroom.</h6>

                                <input autoFocus id="username-input" className="form-control" type="text" placeholder="Username" aria-label="Visible to other users in the chatroom." value={this.state.username} onChange={this.handleUsernameChange} onKeyDown={this.handleUsernameKeyDown} />
                                <button type="button" className="btn btn-primary" onClick={this.handleLogin}>Enter the chatroom</button>
                            </div>
                        </div>

                </div>
            </div>
        );
    }
}

export default LoginView;