import React from 'react'
import { Component } from 'react/cjs/react.production.min'
import { Drawer, List, ListItem, ListItemIcon, ListItemText, Typography } from '@mui/material';

/**
 * Dynamically created drawer that consists of a list of contents. Contents passed in (form of a map) are processed by key only
 * The key is stored in a list item. When a list item is clicked, the key of that item is passed via the onClick method
 */
class DynamicDrawer extends Component {

    constructor(props) {
        super(props);
    }

    processMap(input) {
        let keys = [];
        Object.keys(input).forEach((key) => {
            keys.push(<ListItem button onClick={()=>this.props.onClick(key)} key={key}><ListItemText primary={key}></ListItemText></ListItem>)
        });
        return keys
    }

    render() {
        let contents = [];
        if (this.props.contents) {
            contents = (this.props.type == "map") ? this.processMap(this.props.contents) : [];
        }
        return (
            <Drawer onClose={this.props.onClose} open={this.props.open}>
                <Typography sx={{color:'black'}} variant="h6" gutterBottom>
                    {this.props.title}
                </Typography>
                <List>
                    {contents}
                </List>
            </Drawer>
        );
    }


}

export default DynamicDrawer;