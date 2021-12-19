import * as React from 'react';
import { Button, CircularProgress } from '@mui/material';
import { Component } from 'react/cjs/react.production.min';
import { Box } from '@mui/system';

class ButtonLoader extends Component {

    constructor(props) {
        super(props);
        this.state = {disabled: false,
                        loading: false};
    }

    handleClick() {
        this.setState({disabled: true, loading: true});
        this.props.onClick()
            .then((r) => {
                this.setState({disabled: false, loading: false});
                this.props.handleSuccess(r);
            })
            .catch((e) => {
                this.setState({disabled: false, loading: false});
                this.props.handleError(e);
            })
    }

    render() {
        return (<Box sx={{position: "relative"}}>
                    <Button disabled={this.state.disabled} onClick={this.handleClick.bind(this)} variant="contained">Connect</Button>
                    {this.state.loading && <CircularProgress sx={{position: "absolute", left:80, top:4}} size={30}></CircularProgress>}
                </Box>)
    }
}


export default ButtonLoader;