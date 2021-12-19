import React from 'react'
import { Component } from 'react/cjs/react.production.min'
import logo from './logo.svg'
import tauriCircles from './tauri.svg'
import tauriWord from './wordmark.svg'
import { emit, listen } from '@tauri-apps/api/event'
import { getCurrent, WebviewWindow } from '@tauri-apps/api/window'
import Login from './Login.js'
import Navigator from './Navigator'
import './App.css'

class DependencyLoader extends Component {

    render() {
        return (<div></div>);
    }
}

export default DependencyLoader;