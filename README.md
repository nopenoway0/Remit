# Remit
[![file sync example](https://user-images.githubusercontent.com/13967957/149494591-1ecaf700-8fda-47fe-969b-75beb39c6046.mp4)]
Remit is an automated file syncer. You can connect to a remote server, navigate through a gui and select a file to open and edit. The file will be downloaded and then any edits you make will be synced to the server. Likewise, anytime you open the file it will be sync to your local machine. The initial open will download the whole file, but subsequent opens will only download/upload diffs thanks to rclone's sync.

# Usage

## Create a Configuration File
Use the configuration tab to enter your credentials. 
[![Create config example](https://user-images.githubusercontent.com/13967957/149495836-439dddef-a012-4a44-868b-6ed2e4365331.mp4)]
Note, Remit currently only supports username/password authentication, but I intend to add support for additional login options in the future.

## With Working Configuration File
Simply select your config from the side bar, enter the word you used to encrypt it, and click connect:
[![connect and navigate example](https://user-images.githubusercontent.com/13967957/149493516-9c0c77af-aa2d-4b22-be05-e898840ce67c.mp4)]

# Capabilites
* Connect to a server through ssh using username and password credentials
* Save configurations for different servers
* Download and open files on your local machine
* Automatically sync changes made on your local filesystem to the server
* Navigate the remote file system ( no sudo support yet)

# Roadmap
Look at milestones to see the intended features


# Dev

## External Dependencies
A compiled binary of rclone is needed to use this application. Place the rclone binary into src-tauri/bin/ and rename it rclone-x86_64-pc-windows-msvc.exe. This will allow the production build to include it in the installer. In order to run the dev copy place the renamed exe in src-tauri.

## Running and Building

### `npm run tauri dev`
Opens the application in dev mode. Any changes to the frontend ( React ) or the backend ( Rust ) will result in an automatic rebuild and launch

### `npm run tauri build`
Builds a production version of the exe