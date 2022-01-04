import React from 'react'
import { Button, Grid } from '@mui/material';
import { Component } from 'react/cjs/react.production.min'
import { InsertDriveFileSharp } from '@mui/icons-material';
import FolderSharpIcon from '@mui/icons-material/FolderSharp';
import QuestionMarkSharpIcon from '@mui/icons-material/QuestionMarkSharp';

/**
 * A graphical representation of a RemitFile. A simple graphical file tile
 */
class RemitFile extends Component {
    constructor(props) {
        super(props);
    }

    render() {
        let icon;
        let size = "";
        switch(this.props.type) {
            case "TypeDirectory":  icon = <FolderSharpIcon />; break;
            case "TypeFile": icon = <InsertDriveFileSharp />; size = this.props.size; break;
            default: icon = <QuestionMarkSharpIcon/>
        }

        return (<Button fullWidth={true} variant="outlined" onClick={()=>{this.props.onClick(this.props.name)}}elevation={0} onClick={this.props.onClick}>
                    <Grid container>
                        <Grid item xs={2}>
                            {icon}
                        </Grid>
                        <Grid item xs={8}>
                            {this.props.name}
                        </Grid>
                        <Grid item xs={2}>
                            {size}
                        </Grid>
                    </Grid>
                </Button>)
    }
}

export default RemitFile;