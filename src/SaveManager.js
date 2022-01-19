import * as React from 'react'
import { Component } from 'react/cjs/react.production.min'
import {Stack, TextField, Box, Button, Typography} from '@mui/material'
import ButtonLoader from './ButtonLoader';
import OkDialog from './OkDialog'
import { invoke } from '@tauri-apps/api/tauri';
import "./App.css"
import RemitUtilities from './utils';
var aesjs = require('aes-js')
/**
 * A window that allows users to save configurations allowing easier login
 */
class SaveMananger extends Component {

    /**
     * 
     * @param {Object} props
     * @param {NoArgNoReturnCallback} props.onClose Handle onClose event 
     * @param {string} [props.name]
     * @param {string} [props.host]
     * @param {string} [props.username]
     * @param {string} [props.password]
     * @param {string} [props.encryption_key]
     * @param {number} [props.port]
     */
    constructor(props) {
        super(props);
        let textfields = [{label:"Username", type:"standard", error:false, error_text:""},
                            {label:"Password", type:"password", error:false, error_text:""},
                            {label:"Host", type:"standard", error:false, error_text:""},
                            {label:"Port", type:"number", error:false, error_text:""},
                            {label:"Name", type:"standard", error:false, error_text:""},
                            {label:"Encryption Key", type:"standard", error:false, error_text:""}];
        this.state = {showDialog: false,
                        dialogText: "",
                        showPassword: false,
                        dialogClickHandler: ()=>{},
                        textfields: textfields};
    }

    /**
     * Retrieves designated elements and their values and puts them in a dict id->value
     * @returns {Object<string, string>}
     * @private
     */
    getFormData() {
        let fields = this.state.textfields.map(f=>RemitUtilities.string_to_key(f.label));
        return RemitUtilities.extract_elements(fields);
    }

    /**
     * Constructs Material UI TextField list to be inserted on rendering
     * @param {TextFieldRecipe[]} textfields 
     * @returns {TextField[]}
     * @private
     */
    processTextFields(textfields) {
        let result = [];
        for (const field of textfields) {
            let {label} = field;
            let id = RemitUtilities.string_to_key(label);
            let textfield = <TextField error={field.error} autoComplete="false" key={id} variant="standard" type={field.type} required label={label}
                                defaultValue={this.props[id]} helperText={field.error_text} id={id}/>;
            result.push(textfield);
        }
        return result;
    }

    /**
     * Scan the form for empty text fields
     * @returns {Object<string, bool>} A map consisting of the error field id and true. If the map contains the id, then it is incorrect
     * @private
     */
    getIncorrectInputs() {
        let fields = this.getFormData();
        let error_fields = {errors:0};
        for (const key in fields) {
            if (!RemitUtilities.filled_string(fields[key])) {
                error_fields.errors += 1;
                error_fields[key] = true;
            }
        }
        return error_fields;
    }

    /**
     * Pad a string to the desired length. Bad for security
     * @param {string} str 
     * @param {number} desired_length 
     * @returns {string} padded string
     * @private
     */
    addPadding(str, desired_length) {
        while (str.length < desired_length) {
            str += "f";
        }
        return str;
    }

    /**
     * Attempt to save the configuration
     * @returns {Promise<string, string>} Contains a message on either success or failure
     * @private
     */
    save() { 
        return new Promise((res, rej) => {
            let error_map = this.getIncorrectInputs();
            let {errors} = error_map;
            // if we have errors do markup and reject with dialog error message
            if (errors > 0) {
                let textfields = [...this.state.textfields];
                for (var field  of textfields) {
                    let key = RemitUtilities.string_to_key(field.label);
                    field.error = (key in error_map);
                    field.error_text = (field.error) ? "Please fill in" : "";
                }
                this.setState({textfields:textfields});
                rej("Please fill out all required fields");
            } else {
                // no errors means let's encrypt using the key and attempt to create the config file
                let form_data = this.getFormData();
                form_data.encryptedpassword = form_data.password;
                if (RemitUtilities.filled_string(form_data.encryption_key)) {
                    let padded_key = aesjs.utils.utf8.toBytes(this.addPadding(form_data.encryption_key, 32));
                    let aesCtr = new aesjs.ModeOfOperation.ctr(padded_key);
                    form_data.encryptedpassword = aesjs.utils.hex.fromBytes(aesCtr.encrypt(aesjs.utils.utf8.toBytes(form_data.password)));
                }
                invoke("plugin:Remit|save_config", form_data)
                    .then((e)=>res(e))
                    .catch((e)=>rej(e));
            }
        });
    }

    /**
     * enable dialog and set dialog click handler
     * @param {string} s dialog text 
     * @private
     */
    success(s) {
        this.setState({showDialog: true, dialogText: s, dialogClickHandler:this.closeHandler.bind(this)});
    }

    /**
     * enable dialog and set click handler
     * @param {string} f dialog text 
     * @private
     */
    fail(f) {
        this.setState({showDialog: true, dialogText: f, dialogClickHandler:()=>this.setState({showDialog: false})});
    }

    /**
     * Run the close handler
     * @private
     */
    closeHandler() {
        this.props.onClose();
    }

    render() {
        let name = (this.props.name != undefined) ? this.props.name : "";
        let textfields = this.processTextFields(this.state.textfields);
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
                            {textfields}
                            <Stack direction="row" justifyContent="center" alignItems="center" spacing={2}>
                                <Button variant="outlined" onClick={this.closeHandler.bind(this)}>Cancel</Button>
                                <ButtonLoader text={"Save"} onClick={this.save.bind(this)} handleSuccess={this.success.bind(this)}  handleError={this.fail.bind(this)} /> 
                            </Stack>
                        </Stack>
                    </Box>
                </body>
            </div>
        );
    }
}

export default SaveMananger;