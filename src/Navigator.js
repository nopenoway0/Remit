import React from 'react'
import { Component } from 'react/cjs/react.production.min'
import { invoke } from '@tauri-apps/api/tauri';
import {Button, Grid, Paper, Backdrop} from '@mui/material'
import RemitFile from "./RemitFile"
import ContextMenu from "./ContextMenu"
import EntryDialog from "./EntryDialog"
import './App.css'
import { DeleteForeverSharp, DriveFileRenameOutlineSharp, FolderSharp, FileUploadSharp, Email } from '@mui/icons-material';
/**
 * The navigator component is the main portion of the application. This shows the files in the current directory
 * and processes clicks on these components. It gets its data from the Rust which uses a combination of ssh2 and rclone
 * to get information on the remote server
 */
class Navigator extends Component {

    constructor(props) {
        super(props);
        this.state = {files:[],
                        lockScreen:false,
                        contextmenu:<div></div>,
                        contextMenuPos:{x:0, y:0},
                        editingIndeces:[],
                        showNameDialog: false,
                        createDialogCallbacks:{}};
        this.menu = React.createRef();
        this.menuHandler = this.outOfMenuClickHandler.bind(this);
        this.disableDefaultContextMenu = (e)=>{e.preventDefault()};
    }

    componentDidMount() {
        document.addEventListener('click', this.menuHandler);
        document.addEventListener('contextmenu', this.disableDefaultContextMenu);
        this.listFiles((files)=>this.setState({files:files}), (e)=>{console.log(e)});
    }

    componentWillUnmount() {
        document.removeEventListener('click', this.menuHandler);
        document.removeEventListener('contextmenu', this.disableDefaultContextMenu)
    }

    outOfMenuClickHandler(e) {
        if(!this.menu.current.contains(e)) {
            this.setState({contextMenuOpen: false});
        }
    }

    clearEditingMode() {
        let files = [...this.state.files];
        for (const index of this.state.editingIndeces) {
            files[index].editing = false;
        }
        this.setState({files: files});
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
        data.event.preventDefault();
        data.event.stopPropagation();
    }


    createDir(dirname) {
        this.setState({lockScreen: true, contextMenuOpen: false});
        invoke("plugin:Remit|create_dir", {dirname:dirname})
            .then((e) => {
                this.listFiles((files)=>this.setState({files:files, lockScreen: false}), (e)=>{console.log(e)});
            })
            .catch((e) => {
                console.log(e);
                this.setState({lockScreen: false})
            })
    }
    
    createFile(filename) {
        this.setState({lockScreen: true, contextMenuOpen: false});
        invoke("plugin:Remit|create_file", {filename:filename})
            .then((e) => {
                this.listFiles((files)=>this.setState({files:files, lockScreen: false}), (e)=>{console.log(e)});
            })
            .catch((e) => {
                console.log(e);
                this.setState({lockScreen: false})
            })
    }

    openCreateDialog(type) {
        const cancel_callback = text => this.setState({showNameDialog: false});
        var accept_callback = null;
        if (type == "TypeFile") {
            accept_callback = (text) => {
                this.setState({showNameDialog: false, lockScreen: true})
                if (text) {
                    this.createFile(text);
                } else {
                    this.setState({lockScreen: true});
                }
            }
        } else if (type == "TypeDirectory") {
            accept_callback = (text) => {
                this.setState({showNameDialog: false, lockScreen: true})
                if (text) {
                    this.createDir(text);
                } else {
                    this.setState({lockScreen: true});
                }
            }
        }
        this.setState({showNameDialog: true, createDialogCallbacks:{decline: cancel_callback.bind(this),
            accept:accept_callback.bind(this)}});
    }

    handleBackgroundRightClick(event) {
        const {clientX, clientY} = event;
        const pos = {x: clientX, y: clientY};
        const menu_items = [{text:"New Directory", callback:this.openCreateDialog.bind(this, "TypeDirectory"), icon:<FolderSharp/>},
                            {text:"New File", callback:this.openCreateDialog.bind(this, "TypeFile"), icon:<FileUploadSharp/>}];
        this.setState({contextMenuOpen: true, contextMenuItems:ContextMenu.build_items(menu_items),
                        contextMenuPos:pos});
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
        return (<div className="App" onContextMenu={(e)=>this.handleBackgroundRightClick(e)}>
                    <EntryDialog onAccept={this.state.createDialogCallbacks.accept} onDecline={this.state.createDialogCallbacks.decline} decline_button_text="Cancel" accept_button_text="Ok" title="Create New" prompt="Enter Name" show={this.state.showNameDialog}/>
                    <ContextMenu key="menu" ref={this.menu} open={this.state.contextMenuOpen} left={this.state.contextMenuPos.x} top={this.state.contextMenuPos.y} menuitems={this.state.contextMenuItems}/>
                    <Backdrop sx={{zIndex:99}} open={this.state.lockScreen}/>
                    <Button onClick={this.disconnect.bind(this)}>Disconnect</Button>
                    <Grid container spacing={2}>
                            {files}
                    </Grid>

        </div>);
    }

}

export default Navigator;