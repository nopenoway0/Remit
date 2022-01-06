mod configmanager;
mod fileeventconsumer;
mod filetracker;
mod manager;
mod sessionmanager;
mod syncmanager;
mod systempaths;

pub type RemitManager = manager::rustssh::Manager;
pub type IOError = std::io::Error;
pub type IOErrorKind = std::io::ErrorKind;
pub type RemitConfig = crate::configmanager::rustssh::RemitConfig;

pub mod Remit{
    pub type SystemPath = crate::systempaths::rustssh::SystemPath;
    pub type RCloneManager = crate::syncmanager::rustssh::RCloneManager;
    pub type Directory = crate::sessionmanager::rustssh::Directory;
    pub type ConfigManager = crate::configmanager::rustssh::ConfigManager;
    pub type Config = crate::configmanager::rustssh::RemitConfig;
    pub type DirectoryTracker = crate::filetracker::rustssh::DirectoryTracker;
    pub type SessionManager = crate::sessionmanager::rustssh::SessionManager;
    pub type FileEventConsumer = crate::fileeventconsumer::rustssh::FileEventConsumer;
    pub type FileEvent = crate::fileeventconsumer::rustssh::FileEvent;
}

#[derive(Eq, PartialEq)]
/// Valid thread status for the consumer thread
pub enum ThreadStatus {
    /// pause the consumer thread
    Pause,
    /// have the consumer thread run
    Resume,
    /// end the consumer thread
    Kill
}