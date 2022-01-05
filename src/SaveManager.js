import * as React from 'react'
import { Component } from 'react/cjs/react.production.min'
import {Stack, TextField, Box, Backdrop, CircularProgress, Button} from '@mui/material'
import "./App.css"
import ButtonLoader from './ButtonLoader';
import OkDialog from './OkDialog'
class SaveMananger extends Component {

    constructor(props) {
        super(props);
        this.state = {showDialog: false,
                        dialogText: ""};
    }

    //TODO rewrite a better for loop
    getIncorrectInputs() {
        let fields = {};
        fields.host = document.getElementById("host").value;
        fields.username = document.getElementById("username").value;
        fields.port = document.getElementById("port").value;
        fields.password = document.getElementById("password").value; 
        fields.name = document.getElementById("name").value;

        let error_fields = {errors:0};
        Object.entries(fields).forEach((key) => {
            if (fields[key] == undefined || fields[key].len == 0) {
                error_fields.errors += 1;
            }
        });
        return error_fields;
    }

    save() { 
        return new Promise((res, rej) => {
            let errors = this.getIncorrectInputs();
            // do backend save here
            if (errors.errors > 0) {
                rej(JSON.stringify(errors));
            } else {
                res("Saved");
            }
        });
    }

    success(s) {
        this.setState({showDialog: true, dialogText: s});
    }

    fail(f) {
        this.setState({showDialog: true, dialogText: f});
    }

    render() {
        return (
            <div className="App">
                <body className="App-header">
                    <OkDialog show={this.state.showDialog} text={this.state.dialogText} title={"Save Status"} onClick={()=>this.setState({showDialog: false})} />
                    <Box sx={{bgcolor: 'background.paper', overflow:'hidden', borderRadius:'12px', boxShadow: 1, display: 'flex',
                            flexDirection:{ xs: 'column', md: 'row'}}}>
                        <Stack sx={{margin:4 }}>
                            <TextField key="username" variant="standard" required label="Username" value={this.props.user} id="username" ></TextField>
                            <TextField key="password" variant="standard" required label="Password" type="password" id="password" value={this.props.pass}></TextField>
                            <TextField key="host" variant="standard" required label="Host" id="host" value={this.props.host}/>
                            <TextField key="port" variant="standard" required label="Port" id="port" value={this.props.port}/>
                            <TextField key="name" variant="standard" required label="Name" id="name"/>
                        </Stack>
                    </Box>
                    <Button>Cancel</Button>
                    <ButtonLoader text={"Save"} onClick={this.save.bind(this)} handleSuccess={this.success.bind(this)}  handleError={this.fail.bind(this)} />
                </body>
            </div>
        );
    }
}

export default SaveMananger;