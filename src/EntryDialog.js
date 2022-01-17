import * as React from 'react'
import { Component } from 'react/cjs/react.production.min'
import { Dialog, DialogTitle, Button, DialogContent, TextField, Stack } from '@mui/material';

/**
 * A simple dialog with an ok button. Once pressed the dialog will run the onClick command
 */
class EntryDialog extends Component {
    
    constructor(props) {
        super(props);
    }

    getInput() {
        return document.getElementById(this.props.key + "user-input").value;
    }

    accept() {
        this.props.onAccept(this.getInput());
    }

    decline() {
        this.props.onDecline(this.getInput());
    }

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