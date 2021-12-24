import * as React from 'react'
import { Component } from 'react/cjs/react.production.min'
import { Dialog, DialogTitle, Button, DialogContent } from '@mui/material';
class OkDialog extends Component {
    
    constructor(props) {
        super(props);
    }

    render() {
        return (<div>
            <Dialog open={this.props.show}>
                <DialogTitle>
                    {this.props.title}
                </DialogTitle>
                <DialogContent>
                    {this.props.text}
                </DialogContent>
                <Button variant="outlined" color="error" onClick={()=>this.props.onClick()}>Ok</Button>
            </Dialog>
        </div>);
    }
}
export default OkDialog;