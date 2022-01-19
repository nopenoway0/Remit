import * as React from 'react';
import { Button, CircularProgress } from '@mui/material';
import { Component } from 'react/cjs/react.production.min';
import { Box } from '@mui/system';

/**
 * A button class with a built in loader. The button will automatically disable itself upon click until both the handleClick and handleSuccess/handleError
 * functions are completed
 */
class ButtonLoader extends Component {

    /**
     * Creates a ButtonLoader
     * @param {Object} props
     * @param {ButtonLoader~onClick} props.handleClick A click handler passed in via prop. Must return a promise
     * @param {string} props.text Text that will be inside the button passed in via prop
     * @param {ButtonLoader~handleOutcome} [props.handleSuccess] A handler to handle a successful run of the click handler passed in via prop
     * @param {ButtonLoader~handleOutcome} [props.handleError] A handler to handle a failed run of the click handler passed in via prop
     */
    constructor(props) {
        super(props);
        this.state = {disabled: false,
                        loading: false};
    }


    /**
     * Set the button to its loading state and run the passed in onClick function. Upon success or failure, set the button to its clickable state
     * and then run the corresponding success or fail handler
     * @access private
     */
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
            });
    }

    /**
     * 
     * @access private 
     */
    render() {
        return (<Box sx={{position: "relative"}}>
                    <Button disabled={this.state.disabled} onClick={this.handleClick.bind(this)} variant="contained">
                        {this.props.text}
                        {this.state.loading && <CircularProgress sx={{position: "absolute", left:"40%"}} size={30}></CircularProgress>}
                    </Button>
                    
                </Box>)
    }
}


export default ButtonLoader;

/**
 * Function performed when the button loader is left clicked
 * @callback ButtonLoader~onClick
 * @return {Promise<*,*>} Return a promise to be handled by the according handleSuccess or handleFailure functions
 */

/**
 * Handle successful output from the onClick function
 * @callback ButtonLoader~handleOutcome
 * @param {*} data Output from the onClick
 */