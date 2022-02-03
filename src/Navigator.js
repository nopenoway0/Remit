import React from 'react'
import { Component } from 'react/cjs/react.production.min'
import { invoke } from '@tauri-apps/api/tauri';
import {Button, Grid, Paper, Backdrop, Box} from '@mui/material'
import RemitFile from "./RemitFile"
import ContextMenu from "./ContextMenu"
import EntryDialog from "./EntryDialog"
import './App.css'
import { DeleteForeverSharp, DriveFileRenameOutlineSharp, FolderSharp, FileUploadSharp, Email } from '@mui/icons-material';
import {FileType} from './constants'
/**
 * The navigator component is the main portion of the application. This shows the files in the current directory
 * and processes clicks on these components. It gets its data from the Rust which uses a combination of ssh2 and rclone
 * to get information on the remote server
 */
class Navigator extends Component {

    /**
     * Create a Navigator instance
     * @param {Object} props
     * @param {NoArgNoReturnCallback} props.disconnectHandler This is function is called when the disconnect button is clicked 
     */
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

    /**
     * Add click and context menu listeners as well as load files on mount
     * @access private
     */
    componentDidMount() {
        document.addEventListener('click', this.menuHandler);
        document.addEventListener('contextmenu', this.disableDefaultContextMenu);
        this.listFiles((files)=>this.setState({files:files}), (e)=>{console.log(e)});
    }

    /**
     * Remove added listeners on unmount
     * @access private
     */
    componentWillUnmount() {
        document.removeEventListener('click', this.menuHandler);
        document.removeEventListener('contextmenu', this.disableDefaultContextMenu)
    }

    /**
     * Checks if a click happened inside of a context menu. If it did not, close the context menu
     * @param {event} e Click event
     * @access private
     */
    outOfMenuClickHandler(e) {
        if(!this.menu.current.contains(e)) {
            this.setState({contextMenuOpen: false});
        }
    }

    /**
     * Clear file/directory editing mode. Any files that are being renamed will stop
     * @access private
     */
    clearEditingMode() {
        let files = [...this.state.files];
        for (const index of this.state.editingIndeces) {
            files[index].editing = false;
        }
        this.setState({files: files});
    }

    /**
     * Load files in the current remote directory
     * @param {Navigator~fileListHandler} successHandler On success the files will be passed to this function
     * @param {Navigator~fileListHandler} errorHandler On failure, the failure will be passed to this function
     * @access private
     */
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

    /**
     * Delete the passed in file both locally and remotely
     * @param {string} file The file name 
     * @param {FileType} [type] Not currently used
     * @access private
     */
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

    /**
     * Renames a file both locally and remotely
     * @param {string} file The file name
     * @param {string} new_name Name to change to
     * @access private
     */
    rename(file, new_name) {
        console.log("rename " + file);
        invoke("plugin:Remit|rename_file", {file:file, newname: new_name})
            .then((f) => {
                console.log(f);
                this.listFiles((files)=>this.setState({files:files, lockScreen: false}), (e)=>{});                
            })
            .catch((e) => console.log(e));
    }

    /**
     * Sets a directory or file into renaming mode. This enables the text field so that the user can change the name
     * @param {string} [file] The file name. Not currently used 
     * @param {RemitFile} obj The RemitFile element that is to be renamed
     * @access private
     */
    renameHandler(file, obj) {
        this.clearEditingMode();
        this.enableEditMode(obj.props.index);
        this.setState({contextMenuOpen: false});
    }

    /**
     * Enables editing mode on a RemitFile component
     * @param {number} index The index of the RemitFile in this.state.files
     * @access private 
     */
    enableEditMode(index) {
        let files = [...this.state.files];
        files[index].editing = true;
        let editingIndeces = [...this.state.editingIndeces];
        editingIndeces.push(index);
        this.setState({files:files, editingIndeces:editingIndeces});
    }

    /**
     * On enter click off the necessary renaming functions and disable editing mode on the component
     * @param {string} oldname old file name
     * @param {event} e Enter keydown even
     * @access private 
     */
    handleEnter(oldname, e) {
        this.rename(oldname, e.target.value);
        this.clearEditingMode();
    }

    /**
     * Handle a right click on a RemitFile component
     * @param {FileType} type
     * @param {string} file File name 
     * @param {object} data
     * @param {RemitFile} data.obj The right clicked RemitFile
     * @param {event} data.event The right click event 
     * @access private
     */
    handleNavigatorRightClick(type, file, data) {
        if (data.obj.props.index == 1)
            return;
        const {clientX, clientY} = data.event;
        const pos = {x:clientX, y:clientY};
        const delete_icon = <DeleteForeverSharp></DeleteForeverSharp>;
        const rename_icon = <DriveFileRenameOutlineSharp></DriveFileRenameOutlineSharp>;
        if (type == FileType.TypeDirectory || type == FileType.TypeFile) {
            const directory_menu_items = [{text:"Delete", callback:()=>this.delete(file, type, "renamed file"), icon:delete_icon},
                                            {text:"Rename", callback:()=>this.renameHandler(file, data.obj), icon:rename_icon}];
            this.setState({contextMenuOpen:true, contextMenuItems:ContextMenu.build_items(directory_menu_items),
                            contextMenuPos:pos});
        }
        data.event.preventDefault();
        data.event.stopPropagation();
    }

    /**
     * Create a folder in the current directory
     * @param {string} dirname 
     * @private
     */
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
    
    /**
     * Create a file in the current directory
     * @param {string} filename 
     * @access private
     */
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

    /**
     * 
     * @param {FileType} type Type of creation.
     * @access private
     */
    openCreateDialog(type) {
        const cancel_callback = text => this.setState({showNameDialog: false});
        var accept_callback = null;
        if (type == FileType.TypeFile) {
            accept_callback = (text) => {
                this.setState({showNameDialog: false, lockScreen: true})
                if (text) {
                    this.createFile(text);
                } else {
                    this.setState({lockScreen: true});
                }
            }
        } else if (type == FileType.TypeDirectory) {
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

    /**
     * Handle a right click event that didn't occur on a RemitFile
     * @param {event} event Right click event 
     * @access private
     */
    handleBackgroundRightClick(event) {
        const {clientX, clientY} = event;
        const pos = {x: clientX, y: clientY};
        const menu_items = [{text:"New Directory", callback:this.openCreateDialog.bind(this, FileType.TypeDirectory), icon:<FolderSharp/>},
                            {text:"New File", callback:this.openCreateDialog.bind(this, FileType.TypeFile), icon:<FileUploadSharp/>}];
        this.setState({contextMenuOpen: true, contextMenuItems:ContextMenu.build_items(menu_items),
                        contextMenuPos:pos});
    }

    /**
     * Process left click on file or directory
     * @param {FileType} type 
     * @param {string} file File or Directory name
     * @access private
     */
    handleNavigatorClick(type, file) {
        this.setState({lockScreen:true})
        // do navigation
        if (type == FileType.TypeDirectory) {
            invoke("plugin:Remit|pushd", {d: file})
                .then(()=>{
                    this.listFiles((files)=>this.setState({files:files, lockScreen: false}), (e)=>{});
                })
                .catch((e)=>{
                    this.setState({lockScreen: false});
                });
        } else if(type == FileType.TypeFile) {
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

    /**
     * Disconnect current ssh session
     * @returns {Promise<string, string>} returns a promise to string explaining either the error or success
     * @access private
     */
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
        return (<Box height="100vh" className="App" onContextMenu={(e)=>this.handleBackgroundRightClick(e)}>
                    <EntryDialog onAccept={this.state.createDialogCallbacks.accept} onDecline={this.state.createDialogCallbacks.decline} decline_button_text="Cancel" accept_button_text="Ok" title="Create New" prompt="Enter Name" show={this.state.showNameDialog}/>
                    <ContextMenu key="menu" ref={this.menu} open={this.state.contextMenuOpen} left={this.state.contextMenuPos.x} top={this.state.contextMenuPos.y} menuitems={this.state.contextMenuItems}/>
                    <Backdrop sx={{zIndex:99}} open={this.state.lockScreen}/>
                    <Button onClick={this.disconnect.bind(this)}>Disconnect</Button>
                    <Grid container spacing={2}>
                            {files}
                    </Grid>

                </Box>);
    }

}

export default Navigator;

/**
 * Called when file information is loaded
 * @callback Navigator~fileListHandler
 * @param {Navigator~ListFile[]} files List of files to be handled
 */