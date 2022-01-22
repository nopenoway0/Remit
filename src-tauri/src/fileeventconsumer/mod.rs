pub mod rustssh {
    //! This module is responsible for handling [`FileEvent`]s as they occur in the local directory. These events are created in the 
    //! [`crate::filetracker::rustssh::DirectoryTracker`] struct. 
    //! 
    //! Upon creating a new [`FileEventConsumer`] using the [`FileEventConsumer::new`] method, a separate thread, controlled by the consumer will be created.
    //! This thread can be controlled through the [`FileEventConsumer::start`], [`FileEventConsumer::pause`] and [`FileEventConsumer::end`] methods. 
    //! 
    //! To kill the thread call the [`FileEventConsumer::end`] method.
    //! 
    //! **[`FileEventConsumer`]s should not be created without using the new method. The consuming thread will not be created if made this way.**
    //! 
    //! Currently, [`FileEventConsumer`] only processes file modification events. It ignores deletions and creations. These are expected
    //! to be handled directly through the gui application and so the events reported by Windows can be ignored



    use windows::Win32::Storage::FileSystem::*;
    use std::sync::{Arc, Mutex};
    use std::thread::*;
    use std::fs::metadata;
    use crate::*;

    /// Contains information from a Win32 [ReadDirectoryW](https://docs.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-readdirectorychangesw)
    /// api alert
    pub struct FileEvent {
        /// The type of Win32 file event
        event_type: FILE_ACTION,

        /// Remote path to the affected file - translated from the local path
        remote_file_path: String,

        /// Local path to the affected file
        local_file_path: String
    }

    impl FileEvent {
        /// Create a new FileEvent with the provided parameters
        /// # Arguments
        /// * `event_type` - A Win32 FILE_ACTION
        /// * `local_path` - Path to the file on the local computer
        /// * `remote_path` - Path on the remote system
        pub fn new(event_type: FILE_ACTION, local_path: String, remote_path: String) -> FileEvent {
            return FileEvent{event_type: event_type, local_file_path: local_path, remote_file_path: remote_path};
        }
    }

    /// Consumer for FileEvents. On creation will start a new thread to process
    /// FileEvents present in the shared vector. Use this struct to manage the thread. 
    /// Although the vectors are instanced, there should only be one of these running per program
    pub struct FileEventConsumer {
        /// Shared Mutex to a Vector of shared FileEvents. Usually created by a [`crate::filetracker::rustssh::DirectoryTracker`]
        consumable: Arc::<Mutex::<Vec::<FileEvent>>>,

        /// Shared Mutex to ThreadStatus
        thread_status: Arc::<Mutex::<ThreadStatus>>,

        /// Set the batch size. This controls how many FileEvents are processed in 1 loop. This is set at instantiation and shouldn't
        /// be changed after
        batch_size: Arc::<u32>,

        /// An instance to RCloneManager in order to manage syncing
        rclone_instance: Arc::<Mutex::<Remit::RCloneManager>>
    }

    /// Helper method for processing the event vector. This will take in the fileevent
    /// and a shared Remit::RCloneManager to perform the necessary syncing operations with the
    /// corresponding file event
    /// # Arguments
    /// * `instance` - Shared RCloneManager to access rclone exe
    /// * `event` - Incoming file event to process
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

        // We're checking the type of file event. We only cared about files that are added or modified. Everything else should be done
        // through the GUI.
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
        /// Upon creation, a new thread will be created to consume the objects present in the consumable field.ApiRef
        /// use the methods in this struct to control this thread
        /// # Argument
        /// * `rclone_manager` - Shared rclone_manager instance to extract FileEvents from
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

        /// Starts the consumer thread
        pub fn start(&mut self) {
            *self.thread_status.lock().unwrap() = ThreadStatus::Resume;
        }

        /// Pushes an event onto consumable event queue
        /// # Arguments
        /// * `f` - event to push onto queue
        pub fn add_event(&mut self, f: FileEvent) {
            self.consumable.lock().unwrap().push(f);
        }

        /// Clear all events in the consumable queue without processing them
        pub fn clear(&mut self) {
            self.consumable.lock().unwrap().clear();
        }

        /// Pauses the consumer thread
        pub fn pause(&mut self) {
            *self.thread_status.lock().unwrap() = ThreadStatus::Pause;
        }

        /// Ends the consumer thread
        pub fn end(&mut self) {
            *self.thread_status.lock().unwrap() = ThreadStatus::Kill;
        }
    }

}