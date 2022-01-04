import React from 'react'
import { Component } from 'react/cjs/react.production.min'
import Login from './Login.js'
import Navigator from './Navigator'
import './App.css'
/**
 * Application class
 */
class App extends Component{

  constructor(props) {
    super(props);
    this.state = {logged: false,
                  disableScreen: true,
                  missingDeps: false,
                  startup: true,
                  configs: {}};
  }
  /**
   * set logged in state to true
   */
  login() {
    this.setState({logged: true});
  }

  /**
   * set logged in state to false
   */
  logout() {
    this.setState({logged: false});
  }

  render() {

    let page = <Login loggedInCallback={this.login.bind(this)}/>
    if (this.state.logged)
      page = <Navigator disconnectHandler={this.logout.bind(this)}/>;
    return (<div>
              {page}
            </div> );
  }
}

export default App;
