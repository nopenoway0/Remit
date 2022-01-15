pub mod rustssh {
    use windows::Win32::Storage::FileSystem::*;
    use std::sync::{Arc, Mutex};
    use std::thread::*;
    use std::fs::metadata;
    use crate::*;
    /// stores a file action event from the win32 ReadRDirectoryW api
    pub struct FileEvent {
        /// type of win32 file change event
        event_type: FILE_ACTION,

        /// file path as it appears on the remote source
        remote_file_path: String,

        /// filename - is a relative path to the given file/directory
        local_file_path: String
    }

    impl FileEvent {
        /// create a new FileEvent given the FILE_ACTION and filename - should be a relative path to the file
        pub fn new(event_type: FILE_ACTION, local_path: String, remote_path: String) -> FileEvent {
            return FileEvent{event_type: event_type, local_file_path: local_path, remote_file_path: remote_path};
        }
    }

    /// consumer for FileEvents. On creation will start a new thread which will consume
    /// the FileEvents present in the consumable vector. Use this struct to manage the thread. 
    /// Although the vectors are instanced, there should only be one of these running per program
    pub struct FileEventConsumer {
        consumable: Arc::<Mutex::<Vec::<FileEvent>>>,
        thread_status: Arc::<Mutex::<ThreadStatus>>,
        batch_size: Arc::<u32>,
        rclone_instance: Arc::<Mutex::<Remit::RCloneManager>>
    }

    /// helper method for processing the event vector. This will take in the fileevent
    /// and a shared Remit::RCloneManager to perform the necessary syncing operations with the
    /// corresponding file event
    fn process_event(instance: &Arc::<Mutex::<Remit::RCloneManager>>, event: &FileEvent) {
        // TODO error handling for getting current directory path
        match metadata(event.local_file_path.clone()){
            Ok(v) => {
                if v.is_dir() {
                    println!("Event concerning directory - skipping");
                    return;
                }
            }, 
            Err(e) => {
                println!("{}: {}", e, event.local_file_path.clone());
                return;
            }
        }
        println!("{}", event.remote_file_path);
        match event.event_type {
            FILE_ACTION_ADDED | FILE_ACTION_MODIFIED=> {
                let mut local_dir = Remit::Directory::new(None);
                let mut remote_dir = Remit::Directory::new(None);
                local_dir.path.set_win_path(format!("\\{}", event.local_file_path.clone()));
                remote_dir.path.set_win_path(format!("\\{}", event.remote_file_path.clone()));
                // remove filename so not interpreted as directory
                remote_dir.path.popd();
                let filename = local_dir.path.popd();
                let r = instance.lock().unwrap().upload_local_file(local_dir.path.clone(), remote_dir.path.clone(), filename);
                if r.is_err() {
                    println!("Error uploading file");
                }  
            },
            FILE_ACTION_REMOVED => {
                println!("ignore file removed. app will perform deletion");
            }, 
            FILE_ACTION_RENAMED_NEW_NAME => {
                println!("ignore rename1. app will perform renaming");
            },
            FILE_ACTION_RENAMED_OLD_NAME => {
                println!("ignore rename2. app will perform renaming");
            },
            e => {
                println!("Other event occured {}", e.0);
            }
        }
    }

    impl FileEventConsumer {

        /// get a reference to the FileEvent vector
        /// 
        /// Get use this to manually add FileEvents to the queue
        pub fn get_data_reference(&mut self) -> Arc::<Mutex::<Vec::<FileEvent>>> {
            return self.consumable.clone();
        }

        /// Create a new file consumer by passing in a shared instance of Remit::RCloneManager
        /// 
        /// upon creation, a new thread will be created to consume the objects present in the consumable field.ApiRef
        /// use the methods in this struct to control this thread
        pub fn new(rclone_manager: Arc::<Mutex::<Remit::RCloneManager>>) -> FileEventConsumer{
            let consumer = FileEventConsumer{consumable: Arc::new(Mutex::new(Vec::new())),
                                            thread_status: Arc::new(Mutex::new(ThreadStatus::Pause)),
                                            batch_size: Arc::new(5u32),
                                            rclone_instance: rclone_manager.clone()};

            // set up shared environment: queue, thread status, rclone manager and designated batch size
            let consumable_shared = consumer.consumable.clone();
            let shared_status = consumer.thread_status.clone();
            let shared_rclone_m = consumer.rclone_instance.clone();
            let batch_size = consumer.batch_size.clone();
            // starts separate consumer thread
            spawn(move || {
                let sleep_time = std::time::Duration::from_millis(1000);
                while *shared_status.lock().unwrap() != ThreadStatus::Kill  {
                    std::thread::sleep(sleep_time);
                    if *shared_status.lock().unwrap() == ThreadStatus::Pause {
                        continue;
                    }
                    let mut local_data = Vec::<FileEvent>::new();
                    // acquire lock and transfer data into local_data vector. Release lock once transfer is done
                    {
                        let mut shared_data = consumable_shared.lock().unwrap();
                        let mut loop_size = *batch_size as usize;
                        if loop_size > shared_data.len() {
                            loop_size = shared_data.len();
                        }
                        for _ in 0..loop_size {
                            local_data.push(shared_data.pop().unwrap());
                        }
                    }
                    for x in &local_data {
                        process_event(&shared_rclone_m, x);
                    }
                }
                println!("ending file event conumser");
            });
            return consumer;
        }

        /// set the consumer thread status to ThreadStatus::resume
        pub fn start(&mut self) {
            *self.thread_status.lock().unwrap() = ThreadStatus::Resume;
        }

        /// add the given event to the consumer's queue
        pub fn add_event(&mut self, f: FileEvent) {
            self.consumable.lock().unwrap().push(f);
        }

        /// clear all waiting in events in the consumer's queue
        pub fn clear(&mut self) {
            self.consumable.lock().unwrap().clear();
        }

        /// pause the consumer thread from running
        pub fn pause(&mut self) {
            *self.thread_status.lock().unwrap() = ThreadStatus::Pause;
        }

        /// end the consumer thread
        pub fn end(&mut self) {
            *self.thread_status.lock().unwrap() = ThreadStatus::Kill;
        }
    }

}