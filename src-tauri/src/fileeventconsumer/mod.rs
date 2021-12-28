pub mod rustssh {
    use windows::Win32::Storage::FileSystem::*;
    use std::sync::{Arc, Mutex};
    use std::thread::*;
    #[derive(Eq, PartialEq)]
    enum ThreadStatus {
        Pause,
        Resume,
        Kill
    }

    pub struct FileEvent {
        event_type: FILE_ACTION,
        file_name: String
    }

    impl FileEvent {
        pub fn new(event_type: FILE_ACTION, file_name: String) -> FileEvent {
            return FileEvent{event_type: event_type, file_name: file_name};
        }
    }

    pub struct FileEventConsumer {
        consumable: Arc::<Mutex::<Vec::<FileEvent>>>,
        thread_status: Arc::<Mutex::<ThreadStatus>>
    }

    impl FileEventConsumer {
        pub fn start(&mut self) {
            *self.thread_status.lock().unwrap() = ThreadStatus::Resume;
            let consumable_shared = self.consumable.clone();
            let shared_status = self.thread_status.clone();
            spawn(move || {
                let sleep_time = std::time::Duration::from_millis(1000);
                while *shared_status.lock().unwrap() == ThreadStatus::Resume {
                    std::thread::sleep(sleep_time);
                }
            });
        }

        pub fn add_event(&mut self, f: FileEvent) {
            self.consumable.lock().unwrap().push(f);
        }

        pub fn pause(&mut self) {
            *self.thread_status.lock().unwrap() = ThreadStatus::Pause;
        }

        pub fn kill(&mut self) {
            *self.thread_status.lock().unwrap() = ThreadStatus::Kill;
        }
    }

}