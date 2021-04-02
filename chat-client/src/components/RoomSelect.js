
import React, { Component } from "react";
import { Card, Table } from 'react-bootstrap';

import '../css/RoomSelect.css';

class RoomSelect extends Component {

    constructor(props) {
        super(props);

        this.renderRooms = this.renderRooms.bind(this);
    }

    renderRooms() {
        return this.props.rooms.map(room => {
            return (
                <tr className="room-row" key={room.id} onClick={() => this.props.handleJoinRoom(room.id)}>
                    <td>
                        {room.name}
                    </td>
                    <td>
                        {room.connectedClients} / {room.maxClients}
                    </td>
                </tr>
            );
        });
    }

    render() {
        return (
            <div className="RoomSelect">
                <Card>
                    <Card.Body>Available rooms:</Card.Body>

                    <Table>
                        <thead>
                            <tr>
                                <th>Room name</th>
                                <th>Users</th>
                            </tr>
                        </thead>
                        <tbody>
                            {this.renderRooms()}
                        </tbody>
                    </Table>
                </Card>
            </div>
        );
    }
}

export default RoomSelect;