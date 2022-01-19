import { ListItemText, MenuItem, MenuList, Paper } from "@mui/material";
import { Component } from "react/cjs/react.production.min";
import React from 'react'

class ContextMenu extends Component {

    constructor(props) {
        super(props);
        this.dom_ref = React.createRef();
    }

    /**
     * 
     * @param {*} items an object of text, material ui icon and callback 
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