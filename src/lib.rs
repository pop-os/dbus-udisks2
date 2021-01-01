//! You probably want to look at [`UDisks2`] or [`AsyncUDisks2`].

use std::collections::HashMap;
use std::ops::Deref;
use std::time::Duration;

use dbus::arg::Variant;
use dbus::blocking;
use dbus::blocking::stdintf::org_freedesktop_dbus::{ObjectManager, Properties};

use crate::smart::{RawSmartAttribute, SmartData, SmartStatus, SmartValue};
pub use block::*;
pub use disks::*;
pub use drive::*;
#[cfg(feature = "futures")]
pub use nonblock::*;
use utils::*;

mod block;
mod disks;
mod drive;
#[cfg(feature = "futures")]
mod nonblock;
pub mod smart;
mod utils;

const DEST: &str = "org.freedesktop.UDisks2";
const PATH: &str = "/org/freedesktop/UDisks2";
const NO_WAKEUP: &str = "nowakeup";

#[derive(Default)]
struct DiskCache(HashMap<dbus::Path<'static>, DbusObjects>);

impl DiskCache {
    fn get_object<T: ParseFrom>(&self, path: &str) -> Option<T> {
        self.0
            .iter()
            .flat_map(|object| {
                if object.0.deref() == path {
                    T::parse_from(&object.0, &object.1)
                } else {
                    None
                }
            })
            .next()
    }

    /// Find the drive that corresponds to the given dbus object path.
    fn get_drive(&self, path: &str) -> Option<Drive> {
        self.get_object::<Drive>(path)
    }

    /// An iterator of `Drive` objects fetched from the inner cached managed objects.
    fn get_drives<'a>(&'a self) -> impl Iterator<Item = Drive> + 'a {
        self.0
            .iter()
            .flat_map(|object| Drive::parse_from(&object.0, &object.1))
    }

    /// Find the block that corresponds to the given dbus object path.
    fn get_block(&self, path: &str) -> Option<Block> {
        self.get_object::<Block>(path)
    }

    /// An iterator of `Block` objects fetched from the inner cached managed objects.
    fn get_blocks<'a>(&'a self) -> impl Iterator<Item = Block> + 'a {
        self.0
            .iter()
            .flat_map(|object| Block::parse_from(&object.0, &object.1))
    }
}

pub struct UDisks2 {
    conn: blocking::Connection,
    cache: DiskCache,
}

#[derive(Debug)]
pub enum MountError {
    /// This block device has no file system
    NoFS,
    DBUS(dbus::Error),
}

impl From<dbus::Error> for MountError {
    fn from(e: dbus::Error) -> Self {
        MountError::DBUS(e)
    }
}

impl std::fmt::Display for MountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MountError::NoFS => write!(f, "No filesystem found on block device"),
            MountError::DBUS(e) => write!(f, "Could not (un)mount file system: {}", e),
        }
    }
}

impl std::error::Error for MountError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MountError::NoFS => None,
            MountError::DBUS(e) => Some(e),
        }
    }
}

impl UDisks2 {
    pub fn new() -> Result<Self, dbus::Error> {
        let mut udisks2 = Self {
            conn: blocking::Connection::new_system()?,
            cache: Default::default(),
        };

        udisks2.update()?;
        Ok(udisks2)
    }

    fn proxy_with_timeout<'a>(
        &'a self,
        path: impl Into<dbus::Path<'a>>,
        timeout: Duration,
    ) -> blocking::Proxy<&blocking::Connection> {
        blocking::Proxy::new(DEST, path, timeout, &self.conn)
    }

    fn proxy<'a>(
        &'a self,
        path: impl Into<dbus::Path<'a>>,
    ) -> blocking::Proxy<&blocking::Connection> {
        self.proxy_with_timeout(path, Duration::from_millis(3000))
    }

    /// Refresh the managed objects fetched from the DBus server.
    pub fn update(&mut self) -> Result<(), dbus::Error> {
        self.cache.0 = self.proxy(PATH).get_managed_objects()?;
        Ok(())
    }

    /// Find the drive that corresponds to the given dbus object path.
    pub fn get_drive(&self, path: &str) -> Option<Drive> {
        self.cache.get_drive(path)
    }

    /// An iterator of `Drive` objects fetched from the inner cached managed objects.
    pub fn get_drives<'a>(&'a self) -> impl Iterator<Item = Drive> + 'a {
        self.cache.get_drives()
    }

    /// Find the block that corresponds to the given dbus object path.
    pub fn get_block(&self, path: &str) -> Option<Block> {
        self.cache.get_block(path)
    }

    /// An iterator of `Block` objects fetched from the inner cached managed objects.
    pub fn get_blocks<'a>(&'a self) -> impl Iterator<Item = Block> + 'a {
        self.cache.get_blocks()
    }

    /// Mount the file system on block device `block`. Returns the path where the file system is
    /// mounted.  If `interactive` is true, the user may be interactively asked for a password to get
    /// required privileges.
    pub fn mount(
        &self,
        block: &Block,
        interactive: bool,
        fstype: Option<&str>,
        mount_options: Option<&str>,
        timeout: Duration,
    ) -> Result<std::path::PathBuf, MountError> {
        if !block.has_fs() {
            return Err(MountError::NoFS);
        }
        let proxy = self.proxy_with_timeout(&block.path, timeout);
        let mut options = KeyVariant::<&str>::new();
        if !interactive {
            options.insert("auth.no_user_interaction", Variant(Box::new(false)));
        }
        match fstype {
            Some(t) => {
                options.insert("fstype", Variant(Box::new(t.to_owned())));
            }
            None => (),
        }
        match mount_options {
            Some(o) => {
                options.insert("options", Variant(Box::new(o.to_owned())));
            }
            None => (),
        }
        let (path,): (String,) =
            proxy.method_call("org.freedesktop.UDisks2.Filesystem", "Mount", (options,))?;
        Ok(std::path::PathBuf::from(path))
    }

    /// Unmount the file system on block device `block`.
    /// If `interactive` is true, the user may be interactively asked for a password
    /// to get required privileges.
    /// Fails if the filesystem is busy unless `force` is true.
    pub fn unmount(
        &self,
        block: &Block,
        interactive: bool,
        force: bool,
        timeout: Duration,
    ) -> Result<(), MountError> {
        if !block.has_fs() {
            return Err(MountError::NoFS);
        }
        let proxy = self.proxy_with_timeout(&block.path, timeout);
        let mut options = KeyVariant::<&str>::new();
        if !interactive {
            options.insert("auth.no_user_interaction", Variant(Box::new(false)));
        }
        options.insert("force", Variant(Box::new(force)));
        proxy.method_call("org.freedesktop.UDisks2.Filesystem", "Unmount", (options,))?;
        Ok(())
    }

    /// Eject the corresponding drive.
    /// If `interactive` is true, the user may be interactively asked for a password
    /// to get required privileges.
    pub fn eject(
        &self,
        drive: &Drive,
        interactive: bool,
        timeout: Duration,
    ) -> Result<(), dbus::Error> {
        let proxy = self.proxy_with_timeout(&drive.path, timeout);
        let mut options = KeyVariant::<&str>::new();
        if !interactive {
            options.insert("auth.no_user_interaction", Variant(Box::new(false)));
        }
        proxy.method_call("org.freedesktop.UDisks2.Drive", "Eject", (options,))?;
        Ok(())
    }

    /// Update the S.M.A.R.T. attributes of a drive. You may pass either a `&`[`Drive`] or `&str`
    /// which is a path to a drive, starting with `/org/freedesktop/UDisks2/drives/`.
    pub fn smart_update<'a>(
        &'a self,
        drive: impl Into<dbus::Path<'a>>,
        allow_wakeup: bool,
    ) -> Result<(), dbus::Error> {
        let proxy = self.proxy(drive);
        let mut options = KeyVariant::<&str>::new();
        if !allow_wakeup {
            options.insert(NO_WAKEUP, Variant(Box::new(true)));
        }
        proxy.method_call(smart::DEST, smart::UPDATE, (options,))
    }

    /// Get the S.M.A.R.T. attributes of a drive. You may pass either a `&`[`Drive`] or `&str` which
    /// is a path to a drive, starting with `/org/freedesktop/UDisks2/drives/`.
    pub fn smart_attributes<'a>(
        &'a self,
        drive: impl Into<dbus::Path<'a>>,
        allow_wakeup: bool,
    ) -> Result<SmartValue, dbus::Error> {
        let proxy = self.proxy(drive);
        if !proxy.get::<bool>(smart::DEST, smart::SUPPORTED)? {
            return Ok(SmartValue::NotSupported);
        }
        if !proxy.get::<bool>(smart::DEST, smart::ENABLED)? {
            return Ok(SmartValue::NotEnabled);
        }
        let updated: u64 = proxy.get(smart::DEST, smart::UPDATED)?;
        if updated == 0 {
            return Ok(SmartValue::NotUpdated);
        }
        let mut options = KeyVariant::<&str>::new();
        if !allow_wakeup {
            options.insert(NO_WAKEUP, Variant(Box::new(true)));
        }
        let (attrs,): (Vec<RawSmartAttribute>,) =
            proxy.method_call(smart::DEST, smart::GET_ATTRS, (options,))?;
        Ok(SmartValue::Enabled(SmartData {
            updated,
            failing: proxy.get(smart::DEST, smart::FAILING)?,
            time_powered_on: proxy.get(smart::DEST, smart::TIME_POWER_ON)?,
            temperature: proxy.get(smart::DEST, smart::TEMPERATURE)?,
            failing_attrs_count: proxy.get(smart::DEST, smart::FAILING_ATTRS_COUNT)?,
            past_failing_attrs_count: proxy.get(smart::DEST, smart::PAST_FAILING_ATTRS_COUNT)?,
            bad_sectors: proxy.get(smart::DEST, smart::BAD_SECTORS)?,
            status: proxy
                .get::<String>(smart::DEST, smart::STATUS)?
                .parse()
                .unwrap_or(SmartStatus::Unknown),
            attributes: attrs.into_iter().map(Into::into).collect(),
        }))
    }
}
