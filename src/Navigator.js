import React from 'react'
import { Component } from 'react/cjs/react.production.min'
import { invoke } from '@tauri-apps/api/tauri';
import {Button, Grid, Paper, Box, List, Backdrop} from '@mui/material'
import RemitFile from "./RemitFile"
import './App.css'
import { ConnectingAirportsOutlined } from '@mui/icons-material';


class Navigator extends Component {

    componentDidMount() {
        this.listFiles((files)=>this.setState({files:files}), (e)=>{console.log(e)});
    }

    constructor(props) {
        super(props);
        this.state = {files:[],
                        lockScreen:false};
    }

    listFiles(successHandler, errorHandler) {
        invoke("plugin:Remit|list_current_directory", {path: "."})
            .then((files)=>{
                //this.setState({files: files});
                successHandler(files);
            })
            .catch((e)=>{
                //console.log(e);
                errorHandler(e);
        });
    }

    handleNavigatorClick(type, file) {
        this.setState({lockScreen:true})
        // do navigation
        if (type == "TypeDirectory") {
            invoke("plugin:Remit|pushd", {d: file})
                .then(()=>{
                    this.listFiles((files)=>this.setState({files:files, lockScreen: false}), (e)=>{});
                })
                .catch((e)=>{
                    this.setState({lockScreen: false});
                })
        } else if(type == "TypeFile") {
            invoke("plugin:Remit|download", {filename: file, open:true})
                .then(() => {
                    this.setState({lockScreen: false});
                })
                .catch((e)=>{
                    console.log(e);
                    this.setState({lockScreen: false});
                });
        }
    }

    render() {
        const disconnect = () => {
            return invoke("plugin:Remit|disconnect")
                .catch((e)=> {
                    console.log(e);
                })
                .then((r) => {
                    this.props.disconnectHandler();
                })
        };

        let files = [];
        this.state.files.forEach((file) => {
            if (file.name != ".") {
                files.push(<Grid item xs={4}>
                                <Paper>
                                    <RemitFile key={file.name} onClick={()=>{this.handleNavigatorClick(file.type, file.name)}} type={file.type} name={file.name} size={file.size}></RemitFile>
                                </Paper>
                            </Grid>);
            }
        })
        return (<div className="App">
            <Backdrop sx={{zIndex:99}} open={this.state.lockScreen}/>
            <Button onClick={disconnect.bind(this)}>Disconnect</Button>
            <Grid container spacing={2}>
                    {files}
            </Grid>
        </div>);
    }

}

export default Navigator;