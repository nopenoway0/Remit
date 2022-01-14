# Frontend Code
This frontend is built with React objects that interface with the rust backend using bindings offered by Tauri. This is done by importing invoke via:
`import { invoke } from '@tauri-apps/api/tauri';` then passing the function and required plugin to call the function. More information on calling the rust backend through tauri offerings available [here.](https://tauri.studio/en/docs/usage/intro/)

# Documentation
Worked on so that the application will compile with JSDocs