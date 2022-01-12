# Remit

Remit is an automated file syncer. You can connect to a remote server, navigate through a gui and select a file to open and edit. The file will be downloaded and then any edits you make will be synced to the server. Likewise, anytime you open the file it will be sync to your local machine. The initial open will download the whole file, but subsequent opens will only download/upload diffs thanks to rclone's sync.

# Usage


# Capabilites
* Connect to a server through ssh using username and password credentials
* Save configurations for different servers
* Download and open files on your local machine
* Automatically sync changes made on your local filesystem to the server
* Navigate the remote file system ( no sudo support yet)

# Roadmap
Look at milestones to see the intended features

# Dev
### `npm run tauri dev`
Opens the application in dev mode. Any changes to the frontend ( React ) or the backend ( Rust ) will result in an automatic rebuild and launch

### `npm run tauri build`
Builds a production version of the exe