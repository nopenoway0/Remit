import React from 'react'
import { Component } from 'react/cjs/react.production.min'
import Login from './Login.js'
import Navigator from './Navigator'
import SaveManager from './SaveManager'
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
                  configs: {},
                  openSaveManager: false,
                  saveManagerData: undefined};
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

  openSaveManager(data) {
    console.log("open save manager");
    this.setState({saveManagerData: data});
  }

  closeSaveManager() {
    this.setState({saveManagerData: undefined});
  }

  render() {

    let page = <Login openSaveManagerHandler={this.openSaveManager.bind(this)} loggedInCallback={this.login.bind(this)}/>
    if (this.state.saveManagerData) {
      page = <SaveManager pass={this.state.saveManagerData.password} user={this.state.saveManagerData.username} 
        host={this.state.saveManagerData.host} port={this.state.saveManagerData.port} onClose={this.closeSaveManager.bind(this)}/>
    } else if (this.state.logged) {
      page = <Navigator disconnectHandler={this.logout.bind(this)}/>;
    }
    return (<div>
              {page}
            </div> );
  }
}

export default App;
