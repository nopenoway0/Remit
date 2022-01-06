import * as React from 'react'
import { Component } from 'react/cjs/react.production.min'
import {Stack, TextField, Box, Button, Typography} from '@mui/material'
import ButtonLoader from './ButtonLoader';
import OkDialog from './OkDialog'
import { invoke } from '@tauri-apps/api/tauri';
import "./App.css"

class SaveMananger extends Component {

    constructor(props) {
        super(props);
        this.state = {showDialog: false,
                        dialogText: "",
                        showPassword: false,
                        dialogClickHandler: ()=>{}};
    }

    getFormData() {
        let fields = {};
        fields.host = document.getElementById("host").value;
        fields.user = document.getElementById("username").value;
        fields.port = document.getElementById("port").value;
        fields.password = document.getElementById("password").value; 
        fields.name = document.getElementById("name").value;
        return fields;
    }

    //TODO rewrite a better for loop
    getIncorrectInputs() {
        let fields = this.getFormData();
        let error_fields = {errors:0};
        Object.entries(fields).forEach((entry) => {
            if (entry[1] == undefined || entry[1].length == 0) {
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
                invoke("plugin:Remit|save_config", this.getFormData())
                    .then((e)=>res(e))
                    .catch((e)=>rej(e));
            }
        });
    }

    success(s) {
        this.setState({showDialog: true, dialogText: s, dialogClickHandler:this.closeHandler.bind(this)});
    }

    fail(f) {
        this.setState({showDialog: true, dialogText: f, dialogClickHandler:()=>this.setState({showDialog: false})});
    }

    closeHandler() {
        this.props.onClose();
    }

    render() {
        let name = (this.props.name != undefined) ? this.props.name : "";
        return (
            <div className="App">
                <body className="App-header">
                    <OkDialog show={this.state.showDialog} text={this.state.dialogText} title={"Save Status"} onClick={this.state.dialogClickHandler.bind(this)} />
                    <Box sx={{bgcolor: 'background.paper', overflow:'hidden', borderRadius:'12px', boxShadow: 1, display: 'flex',
                            flexDirection:{ xs: 'column', md: 'row'}}}>
                        <Stack sx={{margin:4 }}>
                            <Typography sx={{color:'black'}} variant="h6" gutterBottom>
                                Enter Configuration Information
                            </Typography>
                            <TextField key="username" variant="standard" required label="Username" defaultValue={this.props.user} id="username"></TextField>
                            <TextField key="password" variant="standard" required label="Password" type="password" id="password" defaultValue={this.props.pass}></TextField>
                            <TextField key="host" variant="standard" required label="Host" id="host" defaultValue={this.props.host}/>
                            <TextField key="port" variant="standard" required label="Port" id="port" defaultValue={this.props.port}/>
                            <TextField key="name" variant="standard" required label="Name" id="name" defaultValue={name}/>
                        </Stack>
                        <Box>
                            <Stack direction="row" justifyContent="center" alignItems="center" spacing={2}>
                                <Button variant="outlined" onClick={this.closeHandler.bind(this)}>Cancel</Button>
                                <ButtonLoader text={"Save"} onClick={this.save.bind(this)} handleSuccess={this.success.bind(this)}  handleError={this.fail.bind(this)} /> 
                            </Stack>
                        </Box>
                    </Box>
                </body>
            </div>
        );
    }
}

export default SaveMananger;