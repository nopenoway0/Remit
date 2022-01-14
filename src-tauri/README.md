# Backend
The source in this directory is used to build the rust backend. The backend is responsible for opening the ssh tunnel, creating and managing configuration files, reaching out and performing commands to rclone. These are all done through the manager class. The main.rs contains the code used to interface with the frontend. The lib.rs contains all the necessary includes for the various Remit structs. To compile documentation for all of the dependencies and classes build using:
`cargo doc --document-private-items`