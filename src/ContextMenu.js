import { ListItemText, MenuItem, MenuList, Paper } from "@mui/material";
import { Component } from "react/cjs/react.production.min";

class ContextMenu extends Component {

    constructor(props) {
        super(props);
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

    render() {
        if (!this.props.open) {
            return (<div sx={{position:"fixed"}}></div>);
        }
        return (<Paper sx={{position:"absolute", overflow:"hidden", zIndex:10, left:this.props.left, top:this.props.top}}>
                    <MenuList>
                        {this.props.menuitems}
                    </MenuList>
                </Paper>);
    }
}

export default ContextMenu;