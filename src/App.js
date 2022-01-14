import React from 'react'
import { Component } from 'react/cjs/react.production.min'
import Login from './Login.js'
import Navigator from './Navigator'
import SaveManager from './SaveManager'
import './App.css'
import { invoke } from '@tauri-apps/api/tauri'
import OkDialog from './OkDialog.js'
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
                  saveManagerData: undefined,
                  rclone_exists: true};
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
    this.setState({saveManagerData: data});
  }

  componentDidMount() {
    this.checkForRclone()
      .then((r) => {
        console.log(r);
        this.setState({rclone_exists: r});
      });
  }

  checkForRclone() {
    return invoke("plugin:Remit|rclone_exe_exists");
  }

  closeSaveManager() {
    this.setState({saveManagerData: undefined});
  }

  render() {

    let page = <Login openSaveManagerHandler={this.openSaveManager.bind(this)} loggedInCallback={this.login.bind(this)}/>
    if (!this.state.rclone_exists) {
      page = <OkDialog show={true} title="rclone not found!" text="rclone-x86_64-pc-windows-msvc.exe not found. Please download rclone, rename it accordingly and place it in the directory"
                onClick={()=>this.setState({logged:false})}/>;
    } else if (this.state.saveManagerData) {
        page = <SaveManager password={this.state.saveManagerData.password} username={this.state.saveManagerData.username} 
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
