import * as React from 'react'
import { Component } from 'react/cjs/react.production.min'
import {Button, Stack, TextField, Box, Backdrop, CircularProgress} from '@mui/material'
import AssignmentIcon from '@mui/icons-material/Assignment';
import OkDialog from './OkDialog'
import ButtonLoader from './ButtonLoader'
import DynamicDrawer from './DynamicDrawer'
import logo from './logo.svg'
import tauriCircles from './tauri.svg'
import tauriWord from './wordmark.svg'
import { emit, listen } from '@tauri-apps/api/event'
import { getCurrent, WebviewWindow } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api/tauri';
import './App.css'

class Login extends Component{

  componentDidMount() {
    /*document.addEventListener('contextmenu', (e)=> {
      e.preventDefault();
    });*/
    this.startup();
  }

  constructor(props) {
    super(props);
    this.state = ({displayDialog: false,
                    dialogText: "Error",
                    inputs:true, 
                    openConfigList: false,
                    config:{name:"", pass:"", host:"", port:"", user:""}});
  }

  showDialog(text) {
    this.setState({displayDialog: true, dialogText: text});
  }

  disableInputs() {
    this.setState({inputs:false});
  }

  enabledInputs() {
    this.setState({inputs: true});
  }


  startup() {
    this.getConfigs()
      .then((c)=> {
        var configs = {};
        c.forEach((c) => {
          configs[c.name] = c;
        })
        this.setState({configs: configs});
      })
      .catch((e) => {
        console.log(e);
      })
  }

  getConfigs() {
    return invoke("plugin:Remit|get_config_names")
  }

  disableScreen() {
    this.setState({disableScreen: true});
  }

  useConfig(name) {
    this.setState({config: this.state.configs[name], openConfigList: false});
  }

  connect(){
    this.disableInputs();
    let host = document.getElementById("host").value;
    let username = document.getElementById("username").value;
    let port = document.getElementById("port").value;
    let password = document.getElementById("password").value;
    return invoke("plugin:Remit|connect", {username: username, host: host, port: port, password: password});
  }

  render() {
      const handle_success = (r) => {
        console.log(r); 
        this.enabledInputs();
        this.props.loggedInCallback();
      };

      const handle_error= (e) => {
        this.showDialog(e); 
        this.enabledInputs()
      };

      const hide_dialog = ()=> {
        this.setState({displayDialog: false});
      };

      return (
        <div className="App">
          <Box sx={{position:"fixed", bgcolor:"background.paper", borderRadius:"8px"}}>
            <AssignmentIcon sx={{position: "relative"}} onClick={()=>{this.setState({openConfigList: true})}}/>
          </Box>
          <DynamicDrawer onClose={()=>this.setState({openConfigList: false})} key="config_list" onClick={this.useConfig.bind(this)} contents={this.state.configs} open={this.state.openConfigList} type="map" />
          <OkDialog key="logindialog" show={this.state.displayDialog} onClick={hide_dialog.bind(this)} title="Error Connecting" text={this.state.dialogText}></OkDialog>
          <body className="App-header">
            <Box sx={{bgcolor: 'background.paper', overflow:'hidden', borderRadius:'12px', boxShadow: 1, display: 'flex',
                        flexDirection:{ xs: 'column', md: 'row'}}}>
            <Stack sx={{margin:4 }}>
              <TextField key="username" disabled={!this.state.inputs} variant="standard" required label="Username" value={this.state.config.user} id="username" ></TextField>
              <TextField key="password" disabled={!this.state.inputs} variant="standard" required label="Password" type="password" id="password" value={this.state.config.pass}></TextField>
              <TextField key="host" disabled={!this.state.inputs} variant="standard" required label="Host" id="host" value={this.state.config.host}/>
              <TextField key="port" disabled={!this.state.inputs} variant="standard" required label="Port" id="port" value={this.state.config.port}/>
              <ButtonLoader onClick={this.connect.bind(this)} handleError={handle_error.bind(this)} handleSuccess={handle_success.bind(this)}/>
            </Stack>
            </Box>
          </body>
        </div>
      )
    }
};

export default Login;