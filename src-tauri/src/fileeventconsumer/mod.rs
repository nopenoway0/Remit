pub mod rustssh {
    use windows::Win32::Storage::FileSystem::*;
    use crate::syncmanager::rustssh::RCloneManager;
    use crate::systempaths::rustssh::SystemPath;
    use crate::sessionmanager::rustssh::Directory;
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
        thread_status: Arc::<Mutex::<ThreadStatus>>,
        batch_size: Arc::<u32>,
        rclone_instance: Arc::<Mutex::<RCloneManager>>
    }

    fn process_event(instance: &Arc::<Mutex::<RCloneManager>>, event: &FileEvent) {
        println!("processing event for {}", event.file_name);
        match event.event_type {
            FILE_ACTION_ADDED => {
                let mut dir = Directory::new(None);
                dir.path = SystemPath::new();
                dir.path.set_win_path(format!("\\{}", event.file_name.clone()));
                let filename = dir.path.popd();
                let r = instance.lock().unwrap().upload_local_file(&mut dir, filename);
                if r.is_err() {
                    println!("Error uploading file");
                }
            },
            FILE_ACTION_MODIFIED => {
                let mut dir = Directory::new(None);
                dir.path = SystemPath::new();
                dir.path.set_win_path(format!("\\{}", event.file_name.clone()));
                let filename = dir.path.popd();
                let r = instance.lock().unwrap().upload_local_file(&mut dir, filename);
                if r.is_err() {
                    println!("Error uploading file");
                }  
            },
            _ => {

            }
        }
    }

    impl FileEventConsumer {

        pub fn get_data_reference(&mut self) -> Arc::<Mutex::<Vec::<FileEvent>>> {
            return self.consumable.clone();
        }

        pub fn new(rclone_manager: Arc::<Mutex::<RCloneManager>>) -> FileEventConsumer{
            let consumer = FileEventConsumer{consumable: Arc::new(Mutex::new(Vec::new())),
                                            thread_status: Arc::new(Mutex::new(ThreadStatus::Pause)),
                                            batch_size: Arc::new(5u32),
                                            rclone_instance: rclone_manager.clone()};
            let consumable_shared = consumer.consumable.clone();
            let shared_status = consumer.thread_status.clone();
            let shared_rclone_m = consumer.rclone_instance.clone();
            spawn(move || {
                let sleep_time = std::time::Duration::from_millis(1000);
                while *shared_status.lock().unwrap() != ThreadStatus::Kill  {
                    std::thread::sleep(sleep_time);
                    if *shared_status.lock().unwrap() == ThreadStatus::Pause {
                        continue;
                    }
                    let mut local_data = Vec::<FileEvent>::new();
                    {
                        let mut shared_data = consumable_shared.lock().unwrap();
                        let mut loop_size = 5;
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

        pub fn start(&mut self) {
            *self.thread_status.lock().unwrap() = ThreadStatus::Resume;
        }

        pub fn add_event(&mut self, f: FileEvent) {
            self.consumable.lock().unwrap().push(f);
        }

        pub fn clear(&mut self) {
            self.consumable.lock().unwrap().clear();
        }

        pub fn pause(&mut self) {
            *self.thread_status.lock().unwrap() = ThreadStatus::Pause;
        }

        pub fn kill(&mut self) {
            *self.thread_status.lock().unwrap() = ThreadStatus::Kill;
        }
    }

}