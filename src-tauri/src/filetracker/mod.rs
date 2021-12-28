pub mod rustssh {
    #[allow(unused_imports,dead_code)]
    use windows::Win32::Storage::FileSystem::*;
    use windows::Win32::System::Threading::CreateEventA;
    use windows::Win32::System::IO::{OVERLAPPED, GetOverlappedResult};
    use windows::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE, GetLastError, BOOL, CloseHandle};
    use crate::systempaths::rustssh::SystemPath;
    use std::io::{Error, ErrorKind};
    use std::thread::spawn;
    use std::sync::{Arc, Mutex};
    type IOError = std::io::Error;
    type IOErrorKind = std::io::ErrorKind;
    use std::env::current_dir;
    use std::collections::HashMap;
    const KILL_THREAD: u32 = 1u32;
    const RESUME_THREAD: u32 = 0u32;

    pub struct DirectoryTracker {
        path: SystemPath,
        handles: Arc<Mutex::<Vec::<FindChangeNotificationHandle>>>,
        thread_control: Arc<Mutex::<u32>>,
        dir_handle: Arc<HANDLE>
    }

    impl DirectoryTracker {
        pub fn new(path: SystemPath) -> DirectoryTracker {
            return DirectoryTracker{path: path,
                                    handles: Arc::new(Mutex::new(Vec::<FindChangeNotificationHandle>::new())),
                                    thread_control: Arc::new(Mutex::new(RESUME_THREAD)),
                                    dir_handle: Arc::new(INVALID_HANDLE_VALUE)};
        }

        /// TODO add error handling
        pub fn stop_tracking(&mut self) {
            let mut control = self.thread_control.lock().unwrap();
            *control = KILL_THREAD;
           // let handles = self.handles.lock().unwrap();
            /*unsafe {
                for x in &*handles {
                    FindCloseChangeNotification(x);
                }
            }*/
        }

        fn filename_from_notify_obj(obj: &FILE_NOTIFY_INFORMATION) -> String{
            let mut buffer: Vec::<u16> = Vec::new();
            let buffer_length = obj.FileNameLength/2;
            buffer.resize(buffer_length as usize, 0);
            println!("size: {}", buffer.len());
            let trav_ptr = &obj.FileName as *const u16;
            unsafe {
                for x in 0..buffer_length {
                    buffer[x as usize] = *(trav_ptr.offset(x as isize));
                }
            }
            return String::from_utf16_lossy(&buffer);
        }

        fn set_dir_handle(&mut self, path: &SystemPath) -> Result<(), IOError> {
            // build absolute path on windows
            let track_path = format!("{}\\{}", current_dir().unwrap().to_str().unwrap(), path.get_windows_path());

            // create file handle to directory we're tracking
            unsafe {
                self.dir_handle = Arc::new(CreateFileA(track_path.clone(), FILE_GENERIC_READ | SYNCHRONIZE | FILE_LIST_DIRECTORY | FILE_GENERIC_WRITE , FILE_SHARE_READ | FILE_SHARE_WRITE, std::ptr::null(), 
                                            OPEN_EXISTING, FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OVERLAPPED, HANDLE(0)));  
                if *self.dir_handle.as_ref() == INVALID_HANDLE_VALUE || (*self.dir_handle.as_ref()).0 as isize == 0 as isize{
                    return Err(Error::new(ErrorKind::Other, format!("Error opening handle with CreateFileA {}", GetLastError().0)));
                } 
            }
            return Ok(());
        }

        /// TODO add error handling
        pub fn start_tracking(&mut self, path: &mut SystemPath) -> Result<(), IOError> {
            self.set_dir_handle(path)?;

            // set variables for multithreading
            *self.thread_control.lock().unwrap() = RESUME_THREAD;
            self.path = path.clone();
            let thread_flag = self.thread_control.clone();
            let shared_dir_handle = self.dir_handle.clone();

            spawn(move || {
                let mut buffer: Vec::<u8> = Vec::new();
                buffer.resize(2048, 0);
                let mut bytes_out:  u32 = 0u32;
                let mut operation_in_progress: bool = false;
                // TODO on finish close handles and clean up
                let mut overlap: OVERLAPPED = OVERLAPPED::default();
                let sleep_time = std::time::Duration::from_millis(1000);
                unsafe {
                    while *thread_flag.lock().unwrap() == RESUME_THREAD {
                        std::thread::sleep(sleep_time);
                        if !operation_in_progress {
                            // start a new readdirectory operation by zeroeing the overlap object and
                            // starting an async ReadDirectoryChangesW function
                            overlap = OVERLAPPED::default();
                            overlap.hEvent = CreateEventA(std::ptr::null(), false, true, "remit_dir_poll");
                            if ReadDirectoryChangesW(shared_dir_handle.as_ref(), buffer.as_mut_ptr() as *mut _ as *mut std::ffi::c_void, 
                                                            buffer.len() as u32,  true, 
                                                            FILE_NOTIFY_CHANGE_LAST_WRITE | FILE_NOTIFY_CHANGE_CREATION |
                                                            FILE_NOTIFY_CHANGE_DIR_NAME | FILE_NOTIFY_CHANGE_FILE_NAME, 
                                                            &mut bytes_out, 
                                                            &mut overlap, 
                                                            None).0 == 0 {
                                println!("Error During ReadDirectory Changes: {}", GetLastError().0);
                            }
                            operation_in_progress = true;
                        } 
                        // process operation in progress here
                        else {
                            if GetOverlappedResult(shared_dir_handle.as_ref(), &mut overlap,&mut bytes_out, false).0 == 0 as i32 {
                                let error = GetLastError().0;
                                if error != 996 {
                                    println!("error during overlapped result: {}", error);
                                }
                            } else {
                                // set operation is finished
                                operation_in_progress = false;

                                // process all info
                                let mut index = 0u32;
                                #[allow(while_true)]
                                while true && bytes_out != 0 {
                                    let info: *const FILE_NOTIFY_INFORMATION = &buffer[index as usize] as *const u8 as *const _ as *const FILE_NOTIFY_INFORMATION; 
                                    let filename = DirectoryTracker::filename_from_notify_obj(&*info);
                                    match (*info).Action {
                                        FILE_ACTION_ADDED => println!("{} added", filename),
                                        FILE_ACTION_REMOVED => println!("{} Deleted", filename),
                                        FILE_ACTION_RENAMED_NEW_NAME => println!("{} rename new", filename),
                                        FILE_ACTION_RENAMED_OLD_NAME => println!("{} renamed old", filename),
                                        FILE_ACTION_MODIFIED => println!("{} modified", filename),
                                        _=> println!("Error")
                                    }
                                    if (*info).NextEntryOffset == 0 {
                                        break;
                                    }
                                    index += (*info).NextEntryOffset;
                                }
                                for x in &mut buffer {
                                    *x = 0u8;
                                }
                            }
                        }
                    }
                    println!("ending thread");
                }
            });
            return Ok(());
        }
    }
}