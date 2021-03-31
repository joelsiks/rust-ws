
import React from "react";
import {
    BrowserRouter as Router,
    Switch,
    Route,
} from "react-router-dom";

import LoginView from './components/LoginView.js';
import ChatView from './components/ChatView';

import './css/App.css';

export default function App() {
    return (
        <Router>
            <Switch>
                <Route path="/chat" render={(props) => <ChatView {...props} />} />
                <Route path="/" render={(props) => <LoginView {...props} />} />
            </Switch>
        </Router>
    );
}