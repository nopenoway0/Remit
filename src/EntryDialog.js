import * as React from 'react'
import { Component } from 'react/cjs/react.production.min'
import { Dialog, DialogTitle, Button, DialogContent, TextField, Stack } from '@mui/material';

/**
 * A simple dialog with an ok button. Once pressed the dialog will run the onClick command
 */
class EntryDialog extends Component {
    
    /**
     * 
     * @param {Object} props 
     * @param {string} props.title Text to appear on the top of the box
     * @param {string} props.prompt Text inside the box that will give the user directions on what to enter
     * @param {bool} props.show Whether or not the dialog shoud be visible
     * @param {string} props.key Key to uniquely identify the dialog
     * @param {EntryDialog~onOutcome} props.onAccept A function that will receive the input of the TextField when the user uses the accept button
     * @param {EntryDialog~onOutcome} props.onDecline A function that will receive the input of the TextField when the user uses the accept button
     * @param {string} props.decline_button_text Text to appear in the button that kicks off the onDecline method
     * @param {string} props.accept_button_text Text to appear in the button that kicks off the onAccept method
     * @param {string} [props.label] The label that appears above the TextField
     */
    constructor(props) {
        super(props);
    }

    /**
     * @access private
     * @returns {string} The input in the textfield
     */
    getInput() {
        return document.getElementById(this.props.key + "user-input").value;
    }

    /**
     * Passes input the onAccept callback
     * @access private
     */
    accept() {
        this.props.onAccept(this.getInput());
    }

    /**
     * Passes the input to the onDecline callback
     * @access private
     */
    decline() {
        this.props.onDecline(this.getInput());
    }

    /**
     * @access private
     */
    render() {
        return (<div>
            <Dialog open={this.props.show}>
                <DialogTitle>
                    {this.props.title}
                </DialogTitle>
                <DialogContent>
                    {this.props.prompt}
                </DialogContent>
                <TextField id={this.props.key + "user-input"} label={this.props.label}></TextField>
                <Stack direction="row" justifyContent="center" alignItems="center" spacing={2}>
                    <Button variant="outlined" onClick={this.decline.bind(this)}>{this.props.decline_button_text}</Button>
                    <Button variant="outlined" onClick={this.accept.bind(this)}>{this.props.accept_button_text}</Button>
                </Stack>

            </Dialog>
        </div>);
    }
}
export default EntryDialog;
/**
 * Call this function when the user either declines or accepts the dialog box
 * @callback EntryDialog~onOutcome
 * @param {string} input The user input from the textfield
 */