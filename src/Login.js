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
    let textfields = [{label:"Username", required:true, type:"standard", error:false, error_text:""},
                      {label:"Password", required:true, type:"password", error:false, error_text:""},
                      {label:"Host", required:true, type:"standard", error:false, error_text:""},
                      {label:"Port", required:true, type:"number", error:false, error_text:""},
                      {label:"Name", required:true, type:"standard", error:false, error_text:""},
                      {label:"Encryption Key", required: false, type:"standard", error:false, error_text:""}];

    this.state = ({displayDialog: false,
                    dialogText: "Error",
                    inputs:true, 
                    openConfigList: false,
                    config:{name:"", pass:"", host:"", port:"", user:""},
                    openSaveManager: false,
                    callbacks: {saveManager: this.openSaveManager.bind(this), configTab: this.openConfigList.bind(this)},
                    textfields:textfields});
  }

  emptyFunction() {

  }

  showDialog(text) {
    this.setState({displayDialog: true, dialogText: text});
  }

  disableInputs() {
    this.setState({inputs:false, callbacks:{saveManager: this.emptyFunction, configTab: this.emptyFunction}});
  }

  enabledInputs() {
    this.setState({inputs: true, callbacks:{saveManager: this.openSaveManager.bind(this), configTab: this.openConfigList.bind(this)}});
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

  useConfig(name) {
    this.setState({config: this.state.configs[name], openConfigList: false});
  }

  addPadding(str, desired_length) {
    while (str.length < desired_length) {
        str += "f";
    }
    return str;
}

  getFormData() {
    let fields = this.state.textfields.map(f=>RemitUtilities.string_to_key(f.label));
    return RemitUtilities.extract_elements(fields);
  }

  validateFormData(form_data) {
    let textfields = [...this.state.textfields];
    let error = false;
    for (const field of textfields) {
      let id = RemitUtilities.string_to_key(field.label);
      field.error = !RemitUtilities.filled_string(form_data[id]) && field.required;
      field.error_text = (field.error) ? "Please fill in" : "";
      error |= field.error;
    }
    this.setState({textfields:textfields});
    return !error;
  }

  connect(){
    return new Promise((res, rej) => {
      this.disableInputs();
      let form_data = this.getFormData();
      if (!this.validateFormData(form_data)) {
        rej("Please fill out required fields");
      }
      if (RemitUtilities.filled_string(form_data.encryption_key)) {
        let padded_key = aesjs.utils.utf8.toBytes(this.addPadding(form_data.encryption_key, 32));
        let aesCtr = new aesjs.ModeOfOperation.ctr(padded_key);
        form_data.password = String.fromCharCode.apply(String, aesCtr.decrypt(aesjs.utils.hex.toBytes(form_data.password)));
      }
      form_data.config = (this.state.config.name != undefined && this.state.config.name.length > 0) ? this.state.config.name : "default_remitconfig";
      invoke("plugin:Remit|connect", form_data)
        .then((s)=>res(s))
        .catch((e)=>rej(e));
    });
  }

  handleSuccess(r) {
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

  passthrough(key, e) {
    let c = {...this.state.config};
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

  createFormFields() {
    let fields = [];
    for (const field of this.state.textfields) {
      let {label, type, required, error, error_text} = field;
      let key = RemitUtilities.string_to_key(label);
      let value = (this.state.config && this.state.config[key]) ? this.state.config[key] : "";
      fields.push(<TextField required={required} value={value} error={error} helperText={error_text} onChange={this.passthrough.bind(this, key)}
                    autoComplete="false" key={key} type={type} disabled={!this.state.inputs} variant="standard" label={label} id={key}/>);
    }
    return fields;
  }

  render() {
      const icon_color = (this.state.inputs) ? "" : "disabled";
      return (
        <div className="App">
          <Box sx={{position:"fixed", bgcolor:"background.paper", borderRadius:"2px"}}>
            <AssignmentIcon id="save-icon" color={icon_color} sx={{position: "relative"}} onClick={this.state.callbacks.configTab}/>
            <SaveIcon id="save-icon" color={icon_color} sx={{position: "relative"}} onClick={this.state.callbacks.saveManager} />
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
              {this.createFormFields()}
              <ButtonLoader text={"Connect"} onClick={this.connect.bind(this)} handleError={this.handleError.bind(this)} handleSuccess={this.handleSuccess.bind(this)}/>
            </Stack>
            </Box>
          </body>
        </div>
      )
    }
};

export default Login;