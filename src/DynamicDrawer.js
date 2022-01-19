import React from 'react'
import { Component } from 'react/cjs/react.production.min'
import { Drawer, List, ListItem, ListItemIcon, ListItemText, Typography } from '@mui/material';

/**
 * Dynamically created drawer that consists of a list of contents. Contents passed in (form of a map) are processed by key only
 * The key is stored in a list item. When a list item is clicked, the key of that item is passed via the onClick method
 */
class DynamicDrawer extends Component {

    /**
     * Create a DynamicDrawer Component
     * @param {Object} props 
     * @param {GenericCallback} props.onClick Call this function when any element in the DynamicDrawer is clicked. 
     *                                          It will pass the key {string} into this function. No return value is used
     * @param {string} props.title The title of the element
     * @param {string} props.type Set to map otherwise the element won't render
     * @param {NoArgNoReturnCallback} props.onClose Use this function when the Drawer is closed
     * @param {bool} props.open Choose whether to show the element
     */
    constructor(props) {
        super(props);
    }

    /**
     * Constructs a list of ListItems from input. The input is a series of keys. The created components
     * will pass the key into the props.onClick handler to be processed
     * @param {Object[]} input a list of objects the contain a key
     * @param {string} input[].key key for object. This will also be displayed as text
     * @returns {ListItem[]} A list of ListItems with the keys set
     */
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