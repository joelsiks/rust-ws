
import React, { Component } from "react";

import DropdownButton from 'react-bootstrap/DropdownButton';
import Dropdown from 'react-bootstrap/Dropdown';

import '../css/Chat.css';
import '../css/Navbar.css';

class ChatView extends Component {

    renderDropdownPeers() {
        if (this.props.connected) {
            return (
                <DropdownButton id="users-navbar-btn" title="Users">
                    {this.props.renderPeers()}
                </DropdownButton>
            );
        }
    }

    render() {
        return (
            <div className="Navbar" style={{ padding: "0px" }}>
                <nav className="navbar navbar-light bg-light">
                    <div className="container-fluid">
                        <a className="navbar-brand">rust-ws</a>
                        <form className="d-flex">
                            <p id="navbar-username">{this.props.username}</p>
                            <button id="rooms-btn" className="btn btn-primary" onClick={this.props.exitRoom}>Rooms</button>

                            {this.renderDropdownPeers()}

                            <button id="logout-btn" className="btn btn-outline-danger" onClick={this.props.handleLogout}>Logout</button>
                        </form>
                    </div>
                </nav>
            </div>
        );
    }
}

export default ChatView;