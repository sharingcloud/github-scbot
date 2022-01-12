import * as React from 'react';
import Nav from 'react-bootstrap/Nav'
import NavDropdown from 'react-bootstrap/NavDropdown'
import Navbar from 'react-bootstrap/Navbar'
import Container from 'react-bootstrap/Container'
import { useState } from 'react';

export const App = () => {
    const [value, setValue] = useState<string>("");

    const sendToken = async () => {
        console.log("Will send", value);
        const body = JSON.stringify({key: value});
        fetch("http://localhost:8008/admin/login", {
            method: 'POST',
            mode: 'no-cors',
            body,
            headers: {
                'Content-Type': 'application/json'
            }
        })
    };

    return (
        <>
            <Navbar bg="dark" variant="dark">
                <Container>
                <Navbar.Brand href="#home">React-Bootstrap</Navbar.Brand>
                <Navbar.Toggle aria-controls="responsive-navbar-nav" />
                <Navbar.Collapse role="menu" id="responsive-navbar-nav">
                    <Nav className="me-auto">
                    <Nav.Link href="#features">Features</Nav.Link>
                    <Nav.Link href="#pricing">Pricing</Nav.Link>
                    <NavDropdown title="Dropdown" id="collasible-nav-dropdown">
                        <NavDropdown.Item href="#action/3.1">Action</NavDropdown.Item>
                        <NavDropdown.Item href="#action/3.2">Another action</NavDropdown.Item>
                        <NavDropdown.Item href="#action/3.3">Something</NavDropdown.Item>
                        <NavDropdown.Divider />
                        <NavDropdown.Item href="#action/3.4">Separated link</NavDropdown.Item>
                    </NavDropdown>
                    </Nav>
                    <Nav>
                    <Nav.Link href="#deets">More deets</Nav.Link>
                    <Nav.Link eventKey={2} href="#memes">
                        Dank memes
                    </Nav.Link>
                    </Nav>
                </Navbar.Collapse>
                </Container>
            </Navbar>
            <Container>
                Token: <input type="text" value={value} onChange={(e) => setValue(e.target.value)} name="token" />
                <button onClick={sendToken}>Authenticate</button>
            </Container>
        </>
    )
}
