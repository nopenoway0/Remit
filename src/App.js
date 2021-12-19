import React from 'react'
import { Component } from 'react/cjs/react.production.min'
import { Backdrop, CircularProgress } from '@mui/material'
import OkDialog from './OkDialog.js'
import Login from './Login.js'
import Navigator from './Navigator'
import { invoke } from '@tauri-apps/api/tauri';
import './App.css'

class App extends Component{

  constructor(props) {
    super(props);
    this.state = {logged: false,
                  disableScreen: true,
                  missingDeps: false,
                  startup: true,
                  configs: {}};
  }

  render() {

    const logged_in = () => {
      this.setState({logged: true});
    };

    const logged_out = () => {
      this.setState({logged: false});
    }

    let page = <Login loggedInCallback={logged_in.bind(this)}/>
    if (this.state.logged)
      page = <Navigator disconnectHandler={logged_out.bind(this)}/>;
    return (<div>
              {page}
            </div> );
  }
}

export default App;
