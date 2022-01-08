import * as React from 'react'
import { Component } from 'react/cjs/react.production.min'
import {Stack, TextField, Box, Backdrop, CircularProgress, Typography} from '@mui/material'
import AssignmentIcon from '@mui/icons-material/Assignment';
import SaveIcon from '@mui/icons-material/Save';
import OkDialog from './OkDialog'
import ButtonLoader from './ButtonLoader'
import DynamicDrawer from './DynamicDrawer'
import { invoke } from '@tauri-apps/api/tauri';
import './App.css'
import RemitUtilities from './utils';

var aesjs = require('aes-js')

/**
 * Manages the login state of the application. Loads saved ssh configurations using
 * the rust backend method get_config_names
 */
class Login extends Component{

  componentDidMount() {
    /*document.addEventListener('contextmenu', (e)=> {
      e.preventDefault();
    });*/
  }

  constructor(props) {
    super(props);
    this.state = ({displayDialog: false,
                    dialogText: "Error",
                    inputs:true, 
                    openConfigList: false,
                    config:{name:"", pass:"", host:"", port:"", user:""},
                    openSaveManager: false});
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


  loadConfigs() {
    return new Promise((res, rej) => {
      this.getConfigs()
      .then((c)=> {
        var configs = {};
        c.forEach((c) => {
          configs[c.name] = c;
        })
        res(configs);
      })
      .catch((e) => {
        rej(e);
      })
    })
  }

  getConfigs() {
    return invoke("plugin:Remit|get_config_names")
  }

  disableScreen() {
    this.setState({disableScreen: true});
  }

  unlock(name) {
    return new Promise((res, rej) => {
      res("");
    });
  }

  useConfig(name) {
    this.unlock(name)
      .then((e) => {
        this.setState({config: this.state.configs[name], openConfigList: false});
      });
  }

  addPadding(str, desired_length) {
    while (str.length < desired_length) {
        str += "f";
    }
    return str;
}

  getFormData() {
    let fields = ['host', 'username', 'port', 'password', 'encrypt_key'];
    return RemitUtilities.extract_elements(fields);
  }

  connect(){
    this.disableInputs();
    let form_data = this.getFormData();
    if (form_data.encrypt_key != undefined && form_data.encrypt_key.length > 0) {
      let padded_key = aesjs.utils.utf8.toBytes(this.addPadding(form_data.encrypt_key, 32));
      let aesCtr = new aesjs.ModeOfOperation.ctr(padded_key);
      form_data.password = String.fromCharCode.apply(String, aesCtr.decrypt(aesjs.utils.hex.toBytes(form_data.password)));
    }
    form_data.config = (this.state.config.name != undefined && this.state.config.name.length > 0) ? this.state.config.name : "default_remitconfig";
    return invoke("plugin:Remit|connect", form_data);
  }

  handleSuccess(r) {
    //console.log(r); 
    this.enabledInputs();
    this.props.loggedInCallback();
  }

  handleError(e) {
    this.showDialog(e); 
    this.enabledInputs();
  }

  hideDialog() {
    this.setState({displayDialog: false});
  }

  updateConfig(key, e) {
    let c = this.state.config;
    c[key] = e.target.value;
    this.setState({config: c});
  }

  openSaveManager() {
    this.props.openSaveManagerHandler(this.getFormData());
  }

  openConfigList() {
    this.loadConfigs()
      .then((configs) => {
        this.setState({configs: configs, openConfigList: true});
      })
      .catch((e) => {
        console.log(e);
      })
  }

  render() {
      return (
        <div className="App">
          <Box sx={{position:"fixed", bgcolor:"background.paper", borderRadius:"2px"}}>
            <AssignmentIcon sx={{position: "relative"}} onClick={this.openConfigList.bind(this)}/>
            <SaveIcon sx={{position: "relative"}} onClick={this.openSaveManager.bind(this)} />
          </Box>
          <DynamicDrawer title="Configs" onClose={()=>this.setState({openConfigList: false})} key="config_list" onClick={this.useConfig.bind(this)} contents={this.state.configs} open={this.state.openConfigList} type="map" />
          <OkDialog key="logindialog" show={this.state.displayDialog} onClick={this.hideDialog.bind(this)} title="Error Connecting" text={this.state.dialogText}></OkDialog>
          <body className="App-header">
            <Box sx={{bgcolor: 'background.paper', overflow:'hidden', borderRadius:'12px', boxShadow: 1, display: 'flex',
                        flexDirection:{ xs: 'column', md: 'row'}}}>
            <Stack sx={{margin:4 }}>
            <Typography sx={{color:'black'}} variant="h6" gutterBottom>
                                Enter Connection Information
              </Typography>
              <TextField autoComplete="false" key="username" disabled={!this.state.inputs} variant="standard" required label="Username" onChange={this.updateConfig.bind(this, "user")} 
                value={this.state.config.user} id="username"/>
              <TextField autoComplete="false" key="password" disabled={!this.state.inputs} variant="standard" required label="Password" type="password" id="password" value={this.state.config.pass}
                onChange={this.updateConfig.bind(this, "pass")}/>
              <TextField autoComplete="false" key="host" disabled={!this.state.inputs} variant="standard" required label="Host" id="host" value={this.state.config.host}
                onChange={this.updateConfig.bind(this, "host")}/>
              <TextField autoComplete="false" key="port" disabled={!this.state.inputs} variant="standard" required label="Port" id="port" value={this.state.config.port}
                onChange={this.updateConfig.bind(this, "port")}/>
              <TextField autoComplete="false" key="encrypt-key" type="password" disabled={!this.state.inputs} variant="standard" label="Encryption Key" id="encrypt_key"/>
              <ButtonLoader text={"Connect"} onClick={this.connect.bind(this)} handleError={this.handleError.bind(this)} handleSuccess={this.handleSuccess.bind(this)}/>
            </Stack>
            </Box>
          </body>
        </div>
      )
    }
};

export default Login;