use anyhow::Result;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use tokio::net::{UnixDatagram, UnixListener};

pub struct ListenerCleanup<T> {
    listener: T,
    path: PathBuf,
}

pub trait ListenerCleanupBind<T> {
    fn bind(path: &Path) -> Result<T>;
}

impl ListenerCleanupBind<UnixListener> for UnixListener {
    fn bind(path: &Path) -> Result<UnixListener> {
        Ok(UnixListener::bind(path)?)
    }
}

impl ListenerCleanupBind<UnixDatagram> for UnixDatagram {
    fn bind(path: &Path) -> Result<UnixDatagram> {
        Ok(UnixDatagram::bind(path)?)
    }
}

impl<T> ListenerCleanup<T> {
    pub fn bind(path: PathBuf) -> Result<ListenerCleanup<T>>
    where
        T: ListenerCleanupBind<T>,
    {
        let listener = T::bind(&path)?;
        Ok(ListenerCleanup { listener, path })
    }
}

impl<T> Deref for ListenerCleanup<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.listener
    }
}

impl<T> Drop for ListenerCleanup<T> {
    fn drop(&mut self) {
        if let Err(e) = std::fs::remove_file(&self.path) {
            log::error!("{e}");
        }
    }
}
