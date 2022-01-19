import React from 'react'
import { Button, Grid, TextField, Typography } from '@mui/material';
import { Component } from 'react/cjs/react.production.min'
import { InsertDriveFileSharp } from '@mui/icons-material';
import FolderSharpIcon from '@mui/icons-material/FolderSharp';
import QuestionMarkSharpIcon from '@mui/icons-material/QuestionMarkSharp';
import {FileType} from './constants'

/**
 * A graphical representation of a RemitFile. A simple graphical file tile
 */
class RemitFile extends Component {

    /**
     * 
     * @param {Object} props 
     * @param {string} props.name Name of the file
     * @param {bool} props.editing Whether or not the file name is currently being edited
     * @param {RemitFile~onEnter} props.onEnter Handle when the enter key is pressed on an edited file
     * @param {RemitFile~onClick} props.onClick Handle when a RemitFile is left clicked
     * @param {RemitFile~onContextMenu} props.onContextMenu Handle a right click
     * @param {FileType} props.type Type of file
     * @param {number} props.size Size of the file
     */
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
            case FileType.TypeDirectory:  icon = <FolderSharpIcon />; break;
            case FileType.TypeFile: icon = <InsertDriveFileSharp />; size = this.props.size; break;
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

/**
 * Callback to process when the enter key is press on an edited file
 * @callback RemitFile~onEnter
 * @param {string} name Current file name
 * @param {event} e Enter event. Can extract new file name from this
 */

/**
 * A funciton to process when a RemitFile component is clicked
 * @callback RemitFile~onClick 
 * @param {string} name Receives the file name
 */

/**
 * A function to handle when the component is right clicked
 * @callback RemitFile~onContextMenu
 * @param {Object} data
 * @param {RemitFile} data.obj RemitFile that was right clicked
 * @param {event} data.e The right click event 
 */