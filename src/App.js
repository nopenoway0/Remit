import React from 'react'
import { Component } from 'react/cjs/react.production.min'
import Login from './Login.js'
import Navigator from './Navigator'
import SaveManager from './SaveManager'
import './App.css'
import { invoke } from '@tauri-apps/api/tauri'
import OkDialog from './OkDialog.js'

class App extends Component{

  /**
   * 
   * @param {*} props 
   */
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
   * Set the logged in state to true
   * @access private
   */
  login() {
    this.setState({logged: true});
  }

  /**
   * Set the logged in state to false
   * @access private
   */
  logout() {
    this.setState({logged: false});
  }

  /**
   * Passes currently entered data to the save manager and opens the save manager window. This will copy the username, password etc.
   * so that if users begin typing in the login screen and then decide they'd like to save this configuration, most of the information will
   * be auto saved when switching to the save config screen.
   * @param {Object} data consists of the required fields in a config that are passed onto the save manager component. All data is optional
   * @param {string} [data.host] host to connect to
   * @param {string} [data.username] ssh username
   * @param {string} [data.password] ssh password
   * @param {number} [data.port] port the server's ssh process is listening on
   * @access private
   */
  openSaveManager(data) {
    this.setState({saveManagerData: data});
  }

  /**
   * On mount, check for a valid rclone executable in the running folder
   * @access private
   */
  componentDidMount() {
    this.checkForRclone()
      .then((r) => {
        console.log(r);
        this.setState({rclone_exists: r});
      });
  }

  /**
   * Make call to the backend to check if a valid rclone exists
   * @returns {bool} True if a valid rclone exe exists, otherwise false
   * @access private
   */
  checkForRclone() {
    return invoke("plugin:Remit|rclone_exe_exists");
  }

  /**
   * Clear all data from the save manager screen. When set to undefined, the Save manager screen will close
   * @access private
   */
  closeSaveManager() {
    this.setState({saveManagerData: undefined});
  }

  /**
   * @access private
   */
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
