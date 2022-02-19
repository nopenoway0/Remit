//! This module is responsible for catching file change events. It uses the Win32 API to receive any changes to a directory. These events
//! are then converted and pushed into a vector where a consumer processes each event. The Directory struct
//! also contains a corresponding consumer.
//! 
//! Because there can be 2 threads running in this class, there are 2 methods to control these threads. The start tracking method
//! starts the consumer and creates a thread to track changers. Conversely, the stop tracking pauses the consumer thread and kills
//! the tracking thread. It's important to start tracking before calling the stop tracking as doing so can result in 2 threads running at the same time 
//! producing duplicate events.

pub mod rustssh {
    #[allow(unused_imports,dead_code)]
    use windows::Win32::Storage::FileSystem::*;
    use windows::Win32::System::Threading::CreateEventA;

    use windows::Win32::System::IO::{OVERLAPPED, GetOverlappedResult};
    use windows::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE, GetLastError, PSTR};
    use std::thread::spawn;
    use std::sync::{Arc, Mutex};
    use std::env::current_dir;
    use std::fs::create_dir_all;
    use crate::*;

    /// DirectoryTracker tracks all the file events in a given directory. Use the [`DirectoryTracker::start_tracking`] method to spawn a new thread. Control this spawned
    /// thread through the stop_tracking method. This thread creates and pushes [`Remit::FileEvent`] into a vector to be consumed. 
    pub struct DirectoryTracker {
        /// Mutex to a system path. This is directory to be tracked
        path: Arc::<Mutex::<Remit::SystemPath>>,

        /// Current thread status
        thread_control: Arc<Mutex::<ThreadStatus>>,
        
        /// The retrieved handle to the directory
        dir_handle: Arc<HANDLE>,

        /// The consumer for the file events
        consumer: Arc::<Mutex::<Remit::FileEventConsumer>>
    }

    impl DirectoryTracker {
        /// Create a new directory tracker
        /// # Arguments
        /// * `path` - Path to track
        /// * `rclone_instance` - Shared rclone manager
        pub fn new(path: Remit::SystemPath, rclone_instance: Arc::<Mutex::<Remit::RCloneManager>>) -> DirectoryTracker {
            return DirectoryTracker{path: Arc::new(Mutex::new(path)),
                                    thread_control: Arc::new(Mutex::new(ThreadStatus::Resume)),
                                    dir_handle: Arc::new(INVALID_HANDLE_VALUE),
                                    consumer: Arc::new(Mutex::new(Remit::FileEventConsumer::new(rclone_instance.clone())))};
        }

        /// This function sets the directory tracking thread status to kill. Then it pauses the consumer and clears all waiting events. 
        /// TODO: need to add a drop to kill the consumer method as well
        pub fn stop_tracking(&mut self) {
            let mut control = self.thread_control.lock().unwrap();
            *control = ThreadStatus::Kill;
            self.consumer.lock().unwrap().pause();
            self.consumer.lock().unwrap().clear();
        }

        /// Given a FILE_NOTIFY_INFORMATION object extract the uf16 information and convert it
        /// to a utf8 String
        /// # Arguments
        /// * `obj` - A Win32 FILE_NOTIFY_INFORMATION for the FileName field to be extracted
        fn filename_from_notify_obj(obj: &FILE_NOTIFY_INFORMATION) -> String{
            let mut buffer: Vec::<u16> = Vec::new();
            let buffer_length = obj.FileNameLength/2;
            buffer.resize(buffer_length as usize, 0);
            let trav_ptr = &obj.FileName as *const u16;
            unsafe {
                for x in 0..buffer_length {
                    buffer[x as usize] = *(trav_ptr.offset(x as isize));
                }
            }
            return String::from_utf16_lossy(&buffer);
        }

        /// Open a file handle to the directory in the path
        /// 
        /// File handles are opened via CreateFileA with a flags 
        /// FILE_GENERIC_READ | SYNCHRONIZE | FILE_LIST_DIRECTORY | FILE_GENERIC_WRITE , FILE_SHARE_READ | FILE_SHARE_WRITE and 
        /// OPEN_EXISTING, FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OVERLAPPED pass to it. If the handle is invalid or NULL then an error is returned
        /// Paths are only valid for windows as the formatting is ({}\\{}, current_directory, relative_windows_path)
        /// # Arguments
        /// * `path` - Path to track. Will be created if necessary
        fn set_dir_handle(&mut self, path: &Remit::SystemPath) -> Result<(), IOError> {
            // build absolute path on windows
            let track_path = format!("{}\\{}", current_dir().unwrap().to_str().unwrap(), path.get_windows_path());
            
            // check if exists - if it doesn't create it
            create_dir_all(track_path.clone())?;

            // create file handle to directory we're tracking
            unsafe {
                let mut path_str = track_path.clone();
                let pstr: PSTR = PSTR(path_str.as_mut_ptr());
                self.dir_handle = Arc::new(CreateFileA(pstr, FILE_GENERIC_READ | SYNCHRONIZE | FILE_LIST_DIRECTORY | FILE_GENERIC_WRITE , FILE_SHARE_READ | FILE_SHARE_WRITE, std::ptr::null(), 
                                            OPEN_EXISTING, FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OVERLAPPED, HANDLE(0)));  
                if *self.dir_handle.as_ref() == INVALID_HANDLE_VALUE || (*self.dir_handle.as_ref()).0 as isize == 0 as isize{
                    return Err(IOError::new(IOErrorKind::Other, format!("Error opening handle with CreateFileA {}", GetLastError())));
                } 
            }
            return Ok(());
        }

        /// Spawn a thread to track file change events at a given path. Create [`Remit::FileEvent`] from the Win32 event and push them onto a vector
        /// to be processed by the [`Remit::FileEventConsumer`]. 
        /// # Arguments
        /// * `path` - System path to track
        /// TODO add error handling
        pub fn start_tracking(&mut self, path: &mut Remit::SystemPath) -> Result<(), IOError> {
            self.set_dir_handle(path)?;

            // set variables for multithreading
            *self.thread_control.lock().unwrap() = ThreadStatus::Resume;

            (*self.path.lock().unwrap()) = path.clone();
            let thread_flag = self.thread_control.clone();
            let shared_dir_handle = self.dir_handle.clone();
            let shared_consumer = self.consumer.clone();
            let thread_path = self.path.clone();

            // start thread to monitor any changes. Changes are pushed into the consumers queue in fileeventconsumer
            spawn(move || -> Result<(), IOError>{
                // start consumer
                {
                    shared_consumer.lock().unwrap().start();
                }

                let mut buffer: Vec::<u8> = Vec::new();
                buffer.resize(2048, 0);
                let mut bytes_out:  u32 = 0u32;
                let mut operation_in_progress: bool = false;
                // TODO on finish close handles and clean up
                let mut overlap: OVERLAPPED = OVERLAPPED::default();
                let sleep_time = std::time::Duration::from_millis(1000);
                while *thread_flag.lock().unwrap() == ThreadStatus::Resume {
                    std::thread::sleep(sleep_time);
                    if !operation_in_progress {
                        // start a new readdirectory operation by zeroeing the overlap object and
                        // starting an async ReadDirectoryChangesW function
                        overlap = OVERLAPPED::default();
                        unsafe {
                            let event_name_pstr: PSTR = PSTR("remit_dir_poll".to_string().as_mut_ptr());
                            overlap.hEvent = CreateEventA(std::ptr::null(), false, true, event_name_pstr);
                            if ReadDirectoryChangesW(shared_dir_handle.as_ref(), buffer.as_mut_ptr() as *mut _ as *mut std::ffi::c_void, 
                                                            buffer.len() as u32,  true, 
                                                            FILE_NOTIFY_CHANGE_LAST_WRITE | FILE_NOTIFY_CHANGE_CREATION |
                                                            FILE_NOTIFY_CHANGE_DIR_NAME | FILE_NOTIFY_CHANGE_FILE_NAME, 
                                                            &mut bytes_out, 
                                                            &mut overlap, 
                                                            None).0 == 0 {
                                println!("Error During ReadDirectory Changes: {}", GetLastError());
                            }
                        }
                        operation_in_progress = true;
                    }    
                    // process operation in progress here
                    else {
                        unsafe {
                            if GetOverlappedResult(shared_dir_handle.as_ref(), &mut overlap,&mut bytes_out, false).0 == 0 as i32 {
                                let error = GetLastError();
                                if error != 996 {
                                    return Err(IOError::new(IOErrorKind::Other, format!("Error during result reading: {}", error)));
                                }
                            }
                        }
                        // set operation is finished
                        operation_in_progress = false;

                        // process all info
                        let mut index = 0u32;
                        #[allow(while_true)]
                        while true && bytes_out != 0 {
                            let info: *const FILE_NOTIFY_INFORMATION = &buffer[index as usize] as *const u8 as *const _ as *const FILE_NOTIFY_INFORMATION; 
                            unsafe {
                                // only push valid actions not 0
                                if (*info).Action != 0 {
                                    let filename = DirectoryTracker::filename_from_notify_obj(&*info);
                                    shared_consumer.lock().unwrap().add_event(Remit::FileEvent::new((*info).Action, format!("{}\\{}", (*thread_path.lock().unwrap()).get_windows_path(), filename),
                                                                                                        filename.clone()));
                                }
                                if (*info).NextEntryOffset == 0 {
                                    break;
                                }
                                index += (*info).NextEntryOffset;
                            }
                        }

                        // TODO needed to zero buffer every run?
                        for x in &mut buffer {
                            *x = 0u8;
                        }
                }
                }
                return Ok(());
            });
            return Ok(());
        }
    }
}