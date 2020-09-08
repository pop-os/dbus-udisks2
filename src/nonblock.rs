use crate::smart::{RawSmartAttribute, SmartData, SmartStatus, SmartValue};
use crate::utils::KeyVariant;
use crate::{smart, Block, DiskCache, Drive, DEST, NO_WAKEUP, PATH};
use dbus::arg::Variant;
use dbus::nonblock;
use dbus::nonblock::stdintf::org_freedesktop_dbus::{ObjectManager, Properties};
use dbus::nonblock::NonblockReply;
use futures_util::join;
use std::ops::Deref;
use std::time::Duration;

/// Async version of [`UDisks2`][crate::UDisks2].
///
/// This requires enabling the `futures` feature flag:
/// ```toml
/// [dependencies]
/// dbus-udisks2 = { version = "0.3", features = ["futures"] }
/// ```
pub struct AsyncUDisks2<C> {
    conn: C,
    pub(crate) cache: DiskCache,
}

impl<'b, C, T> AsyncUDisks2<C>
where
    T: NonblockReply + 'b,
    C: Deref<Target = T>,
{
    /// ```
    /// # tokio::runtime::Runtime::new().unwrap().block_on(async {
    /// // Connect to the D-Bus session bus (this is blocking, unfortunately).
    /// let (resource, conn) = dbus_tokio::connection::new_system_sync().unwrap();
    ///
    /// // The resource is a task that should be spawned onto a tokio compatible
    /// // reactor ASAP. If the resource ever finishes, you lost connection to D-Bus.
    /// tokio::spawn(async {
    ///     let err = resource.await;
    ///     panic!("Lost connection to D-Bus: {}", err);
    /// });
    ///
    /// let udisks2 = dbus_udisks2::AsyncUDisks2::new(conn).await.unwrap();
    /// # });
    /// ```
    pub async fn new(conn: C) -> Result<Self, dbus::Error> {
        let mut udisks2 = Self {
            conn,
            cache: Default::default(),
        };

        udisks2.update().await?;
        Ok(udisks2)
    }

    fn proxy<'a>(&'a self, path: impl Into<dbus::Path<'a>>) -> nonblock::Proxy<&T> {
        nonblock::Proxy::new(DEST, path, Duration::from_millis(3000), &self.conn)
    }

    /// Refresh the managed objects fetched from the DBus server.
    pub async fn update(&mut self) -> Result<(), dbus::Error> {
        self.cache.0 = self.proxy(PATH).get_managed_objects().await?;
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

    /// Update the S.M.A.R.T. attributes of a drive. You may pass either a `&`[`Drive`] or `&str`
    /// which is a path to a drive, starting with `/org/freedesktop/UDisks2/drives/`.
    pub async fn smart_update(
        &'b self,
        drive: impl Into<dbus::Path<'b>>,
        allow_wakeup: bool,
    ) -> Result<(), dbus::Error> {
        let proxy = self.proxy(drive);
        let mut options = KeyVariant::<&str>::new();
        if !allow_wakeup {
            options.insert(NO_WAKEUP, Variant(Box::new(true)));
        }
        proxy
            .method_call(smart::DEST, smart::UPDATE, (options,))
            .await
    }

    /// Get the S.M.A.R.T. attributes of a drive. You may pass either a `&`[`Drive`] or `&str` which
    /// is a path to a drive, starting with `/org/freedesktop/UDisks2/drives/`.
    pub async fn smart_attributes(
        &'b self,
        drive: impl Into<dbus::Path<'b>>,
        allow_wakeup: bool,
    ) -> Result<SmartValue, dbus::Error> {
        let proxy = self.proxy(drive);
        if !proxy.get::<bool>(smart::DEST, smart::SUPPORTED).await? {
            return Ok(SmartValue::NotSupported);
        }
        if !proxy.get::<bool>(smart::DEST, smart::ENABLED).await? {
            return Ok(SmartValue::NotEnabled);
        }
        let updated: u64 = proxy.get(smart::DEST, smart::UPDATED).await?;
        if updated == 0 {
            return Ok(SmartValue::NotUpdated);
        }
        let mut options = KeyVariant::<&str>::new();
        if !allow_wakeup {
            options.insert(NO_WAKEUP, Variant(Box::new(true)));
        }
        let (attrs,): (Vec<RawSmartAttribute>,) = proxy
            .method_call(smart::DEST, smart::GET_ATTRS, (options,))
            .await?;
        let (
            failing,
            time_powered_on,
            temperature,
            failing_attrs_count,
            past_failing_attrs_count,
            bad_sectors,
            status,
        ) = join!(
            proxy.get(smart::DEST, smart::FAILING),
            proxy.get(smart::DEST, smart::TIME_POWER_ON),
            proxy.get(smart::DEST, smart::TEMPERATURE),
            proxy.get(smart::DEST, smart::FAILING_ATTRS_COUNT),
            proxy.get(smart::DEST, smart::PAST_FAILING_ATTRS_COUNT),
            proxy.get(smart::DEST, smart::BAD_SECTORS),
            proxy.get::<String>(smart::DEST, smart::STATUS),
        );
        Ok(SmartValue::Enabled(SmartData {
            updated,
            failing: failing?,
            time_powered_on: time_powered_on?,
            temperature: temperature?,
            failing_attrs_count: failing_attrs_count?,
            past_failing_attrs_count: past_failing_attrs_count?,
            bad_sectors: bad_sectors?,
            status: status?.parse().unwrap_or(SmartStatus::Unknown),
            attributes: attrs.into_iter().map(Into::into).collect(),
        }))
    }
}
