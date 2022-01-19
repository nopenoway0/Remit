import { ListItemText, MenuItem, MenuList, Paper } from "@mui/material";
import { Component } from "react/cjs/react.production.min";
import React from 'react'

/**
 * Creates a context menu.
 */
class ContextMenu extends Component {

    /**
     * 
     * @param {Object} props
     * @param {bool} props.open Whether or not to show the context menu 
     * @param {number} props.left Absolute position from the left
     * @param {number} props.top Aboslute position from the top
     * @param {MenuItems[]} props.menuitems A list of Material UI Menu items to render in the menu
     */
    constructor(props) {
        super(props);
        this.dom_ref = React.createRef();
    }

    /**
     * Creates a list of MenuItems from the list of objects passed into the method.
     * 
     * @param {Object[]} items A a list of objects consisting of the items text, an onclick callback and a Material UI icon
     * @param {string} items[].text the text in the MenuItem
     * @param {NoArgNoReturnCallback} items[].callback a callback for when the item is clicked
     * @param {MaterialUIIcon} [items[].icon] a MaterialUIIcon to appear alongside the text in the MenuItem
     * @return {MenuItem[]} created list of MenuItems
     */
    static build_items(items) {
        let menuitems = [];
        for (const item of items) {
            const {icon, text, callback} = item;
            menuitems.push(<MenuItem onClick={callback}>
                                {icon}
                                <ListItemText>
                                    {text}
                                </ListItemText>
                            </MenuItem>)
        }
        return menuitems;
    }

    /**
     * Checks a click event to see if it was in the ContextMenu
     * @param {event} e a click event 
     * @returns true if the click occured in the ContextMenu component - otherwise, false
     */
    contains(e) {
        return this.dom_ref.contains(e.target);
    }

    render() {
        if (!this.props.open) {
            return (<div ref={ref=>this.dom_ref=ref} sx={{position:"fixed"}}></div>);
        }
        return (
                <div ref={ref=>this.dom_ref=ref}>
                    <Paper   sx={{position:"absolute", overflow:"hidden", zIndex:10, left:this.props.left, top:this.props.top}}>
                        <MenuList>
                            {this.props.menuitems}
                        </MenuList>
                    </Paper>                  
                </div>
                );
    }
}

export default ContextMenu;