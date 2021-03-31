
import React, { Component } from "react";

import DropdownButton from 'react-bootstrap/DropdownButton';
import Dropdown from 'react-bootstrap/Dropdown';

import '../css/Chat.css';
import '../css/Navbar.css';

class ChatView extends Component {
    render() {
        return (
            <div className="Navbar" style={{ padding: "0px" }}>
                <nav className="navbar navbar-light bg-light">
                    <div className="container-fluid">
                        <a className="navbar-brand">rust-ws</a>
                        <form className="d-flex">
                            <p id="navbar-username">{this.props.username}</p>

                            <DropdownButton id="users-navbar-btn" title="Connected users">
                                {this.props.renderPeers()}
                            </DropdownButton>

                            <button className="btn btn-outline-danger" onClick={this.props.handleLogout}>Logout</button>
                        </form>
                    </div>
                </nav>
            </div>
        );
    }
}

export default ChatView;