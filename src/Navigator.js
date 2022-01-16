import React from 'react'
import { Component } from 'react/cjs/react.production.min'
import { invoke } from '@tauri-apps/api/tauri';
import {Button, Grid, Paper, Box, List, Backdrop} from '@mui/material'
import RemitFile from "./RemitFile"
import ContextMenu from "./ContextMenu"
import './App.css'
import { ConnectingAirportsOutlined, ContentCutTwoTone, DeleteForeverSharp, DriveFileRenameOutlineSharp, ThirtyFpsSelect } from '@mui/icons-material';

/**
 * The navigator component is the main portion of the application. This shows the files in the current directory
 * and processes clicks on these components. It gets its data from the Rust which uses a combination of ssh2 and rclone
 * to get information on the remote server
 */
class Navigator extends Component {

    componentDidMount() {
        /*document.addEventListener('contextmenu', (e)=> {
            e.preventDefault();
            //this.setState({contextMenuOpen:true, contextMenuItems: ContextMenu.build_items([{text:"New File", callback:null}])});
        });*/
        this.listFiles((files)=>this.setState({files:files}), (e)=>{console.log(e)});
    }

    clearEditingMode() {
        let files = [...this.state.files];
        for (const index of this.state.editingIndeces) {
            files[index].editing = false;
        }
        this.setState({files: files});
    }

    constructor(props) {
        super(props);
        this.state = {files:[],
                        lockScreen:false,
                        contextmenu:<div></div>,
                        contextMenuPos:{x:0, y:0},
                        editingIndeces:[]};
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

    delete(file, type) {
        this.setState({lockScreen: true, contextMenuOpen: false})
        invoke("plugin:Remit|delete_file", {file: file})
            .then((f) => {
                console.log(f);
                this.listFiles((files)=>this.setState({files:files, lockScreen: false}), (e)=>{});                
            })
            .catch((e)=>{
                this.setState({lockScreen: false});
                console.log(e)
            });
    }

    rename(file, new_name) {
        console.log("rename " + file);
        invoke("plugin:Remit|rename_file", {file:file, newname: new_name})
            .then((f) => {
                console.log(f);
                this.listFiles((files)=>this.setState({files:files, lockScreen: false}), (e)=>{});                
            })
            .catch((e) => console.log(e));
    }

    renameHandler(file, obj) {
        this.clearEditingMode();
        this.enableEditMode(obj.props.index);
        this.setState({contextMenuOpen: false});
    }

    enableEditMode(index) {
        let files = [...this.state.files];
        files[index].editing = true;
        let editingIndeces = [...this.state.editingIndeces];
        editingIndeces.push(index);
        this.setState({files:files, editingIndeces:editingIndeces});
    }

    handleEnter(oldname, e) {
        this.rename(oldname, e.target.value);
        this.clearEditingMode();
    }

    handleNavigatorRightClick(type, file, data) {
        if (data.obj.props.index == 1)
            return;
        const {clientX, clientY} = data.event;
        const pos = {x:clientX, y:clientY};
        const delete_icon = <DeleteForeverSharp></DeleteForeverSharp>;
        const rename_icon = <DriveFileRenameOutlineSharp></DriveFileRenameOutlineSharp>;
        if (type == "TypeDirectory" || type == "TypeFile") {
            const directory_menu_items = [{text:"Delete", callback:()=>this.delete(file, type, "renamed file"), icon:delete_icon},
                                            {text:"Rename", callback:()=>this.renameHandler(file, data.obj), icon:rename_icon}];
            this.setState({contextMenuOpen:true, contextMenuItems:ContextMenu.build_items(directory_menu_items),
                            contextMenuPos:pos});
        }
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
                });
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

    disconnect() {
        return invoke("plugin:Remit|disconnect")
        .catch((e)=> {
            console.log(e);
        })
        .then((r) => {
            this.props.disconnectHandler();
        })
    }

    render() {
        let files = [];
        let index = 0;
        for (const file of this.state.files) {
            if (file.name != ".") {
                files.push(<Grid item xs={4}>
                    <Paper>
                        <RemitFile key={file.name} index={index} editing={file.editing} onContextMenu={(e)=>{this.handleNavigatorRightClick(file.type, file.name, e)}} 
                            onClick={()=>{this.handleNavigatorClick(file.type, file.name)}} type={file.type} name={file.name} size={file.size}
                            onEnter={this.handleEnter.bind(this)} onEsc={()=>{this.clearEditingMode.bind(this); this.setState({contextMenuOpen: false})}}></RemitFile>
                    </Paper>
                </Grid>);    
            }
            index += 1;
        }
        return (<div className="App">
            <ContextMenu open={this.state.contextMenuOpen} left={this.state.contextMenuPos.x} top={this.state.contextMenuPos.y} menuitems={this.state.contextMenuItems}/>
            <Backdrop sx={{zIndex:99}} open={this.state.lockScreen}/>
            <Button onClick={this.disconnect.bind(this)}>Disconnect</Button>
            <Grid container spacing={2}>
                    {files}
            </Grid>
        </div>);
    }

}

export default Navigator;