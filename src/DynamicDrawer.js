import React from 'react'
import { Component } from 'react/cjs/react.production.min'
import { Drawer, List, ListItem, ListItemIcon, ListItemText } from '@mui/material';
class DynamicDrawer extends Component {

    constructor(props) {
        super(props);
    }

    processList(input) {
        return [];
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
            contents = (this.props.type == "map") ? this.processMap(this.props.contents) : this.processList(this.props.contents);
        }
        return (
            <Drawer onClose={this.props.onClose} open={this.props.open}>
                <List>
                    {contents}
                </List>
            </Drawer>
        );
    }


}

export default DynamicDrawer;