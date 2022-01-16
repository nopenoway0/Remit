import React from 'react'
import { Button, Grid, TextField, Typography } from '@mui/material';
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

    handleClick() {
        if (!this.props.editing) {
            this.props.onClick(this.props.name);
        }
    }

    handleEnter(e) {
        if (e.key === "Enter") {
            this.props.onEnter(this.props.name, e);
        }
    }

    render() {
        let icon;
        let size = "";
        switch(this.props.type) {
            case "TypeDirectory":  icon = <FolderSharpIcon />; break;
            case "TypeFile": icon = <InsertDriveFileSharp />; size = this.props.size; break;
            default: icon = <QuestionMarkSharpIcon/>
        }

        return (<Button fullWidth={true} variant="outlined" onClick={this.handleClick.bind(this)} elevation={0} onContextMenu={(e)=>this.props.onContextMenu({obj: this, event: e})}>
                    <Grid container>
                        <Grid item xs={2}>
                            {icon}
                        </Grid>
                        <Grid item xs={8}>
                            {this.props.editing && 
                                <TextField defaultValue={this.props.name} onKeyPress={(e)=>this.handleEnter(e)} size="small"/>
                            }
                            {!this.props.editing &&
                                <Typography style={{textTransform:'none'}}>
                                    {this.props.name}
                                </Typography>
                            }
                        </Grid>
                        <Grid item xs={2}>
                            {size}
                        </Grid>
                    </Grid>
                </Button>)
    }
}

export default RemitFile;