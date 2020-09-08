//! Types related to the S.M.A.R.T. data of drives.

use crate::utils::*;
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;
use std::time::Duration;

pub(crate) const DEST: &str = "org.freedesktop.UDisks2.Drive.Ata";
pub(crate) const UPDATE: &str = "SmartUpdate";
pub(crate) const GET_ATTRS: &str = "SmartGetAttributes";
pub(crate) const ENABLED: &str = "SmartEnabled";
pub(crate) const SUPPORTED: &str = "SmartSupported";
pub(crate) const UPDATED: &str = "SmartUpdated";
pub(crate) const FAILING: &str = "SmartFailing";
pub(crate) const TIME_POWER_ON: &str = "SmartPowerOnSeconds";
pub(crate) const TEMPERATURE: &str = "SmartTemperature";
pub(crate) const FAILING_ATTRS_COUNT: &str = "SmartNumAttributesFailing";
pub(crate) const PAST_FAILING_ATTRS_COUNT: &str = "SmartNumAttributesFailedInThePast";
pub(crate) const BAD_SECTORS: &str = "SmartNumBadSectors";
pub(crate) const STATUS: &str = "SmartSelftestStatus";
pub(crate) type RawSmartAttribute = (u8, String, u16, i32, i32, i32, i64, i32, KeyVariant);

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
#[non_exhaustive]
/// The status of a S.M.A.R.T. test.
pub enum SmartStatus {
    /// Last self-test was a success (or never ran).
    Success,
    /// Last self-test was aborted.
    Aborted,
    /// Last self-test was interrupted.
    Interrupted,
    /// Last self-test did not complete.
    Fatal,
    /// Last self-test failed (Unknown).
    UnknownError,
    /// Last self-test failed (Electrical).
    ElectricalError,
    /// Last self-test failed (Servo).
    ServoError,
    /// Last self-test failed (Read).
    ReadError,
    /// Last self-test failed (Damage).
    HandlingError,
    /// Self-test is currently in progress.
    InProgress,
    /// Unknown status
    Unknown,
}

impl FromStr for SmartStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "success" => Ok(SmartStatus::Success),
            "aborted" => Ok(SmartStatus::Aborted),
            "interrupted" => Ok(SmartStatus::Interrupted),
            "fatal" => Ok(SmartStatus::Fatal),
            "error_unknown" => Ok(SmartStatus::UnknownError),
            "error_electrical" => Ok(SmartStatus::ElectricalError),
            "error_servo" => Ok(SmartStatus::ServoError),
            "error_read" => Ok(SmartStatus::ReadError),
            "error_handling" => Ok(SmartStatus::HandlingError),
            "inprogress" => Ok(SmartStatus::InProgress),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug)]
/// Whether a drive supports S.M.A.R.T. or not.
pub enum SmartValue {
    /// The drive does not support S.M.A.R.T.
    NotSupported,
    /// The drive supports S.M.A.R.T., but it's not enabled.
    NotEnabled,
    /// S.M.A.R.T. is supported and enabled, but it's never been read. Call
    /// [`smart_update`][crate::UDisks2::smart_update]
    /// ([async version][crate::AsyncUDisks2::smart_update]).
    NotUpdated,
    Enabled(SmartData),
}

#[derive(Clone, Debug)]
/// The S.M.A.R.T. data of a drive.
pub struct SmartData {
    pub attributes: Vec<SmartAttribute>,
    /// The point in time (seconds since the Unix Epoch) that the SMART status was updated.
    pub updated: u64,
    /// Set to `true` if disk is about to fail.
    ///
    /// This value is read from the disk itself and does not include any interpretation.
    pub failing: bool,
    /// The amount of time the disk has been powered on (according to SMART data) or 0 if unknown.
    pub time_powered_on: u64,
    /// The temperature (in Kelvin) of the disk according to SMART data or 0 if unknown.
    pub temperature: f64,
    /// The number of attributes failing right now or -1 if unknown.
    pub failing_attrs_count: i32,
    /// The number of attributes that have failed in the past or -1 if unknown.
    pub past_failing_attrs_count: i32,
    /// The number of bad sectors (ie. pending and reallocated) or -1 if unknown.
    pub bad_sectors: i64,
    /// The status of the last self-test.
    pub status: SmartStatus,
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive, Copy, Clone, Hash)]
#[repr(u8)]
#[non_exhaustive]
pub enum PrettyUnit {
    Dimensionless = 1,
    Milliseconds,
    Sectors,
    Millikelvin,
}

#[derive(Eq, PartialEq, Copy, Clone, Hash)]
pub struct PrettyValue {
    pub value: i64,
    pub unit: PrettyUnit,
}

impl fmt::Debug for PrettyValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} / ", self)?;
        f.debug_struct("PrettyValue")
            .field("value", &self.value)
            .field("unit", &self.unit)
            .finish()
    }
}

impl fmt::Display for PrettyValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.unit {
            PrettyUnit::Dimensionless => write!(f, "{}", self.value),
            PrettyUnit::Milliseconds => write!(f, "{:?}", Duration::from_millis(self.value as u64)),
            PrettyUnit::Sectors => write!(f, "{} sectors", self.value),
            PrettyUnit::Millikelvin => {
                write!(f, "{:.1} degrees C", self.value as f32 / 1000. - 273.15)
            }
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum SmartAssessment {
    Failing,
    FailedInPast,
    Ok,
}

#[derive(Clone, Debug)]
/// A S.M.A.R.T. attribute.
pub struct SmartAttribute {
    /// Attribute Identifier
    pub id: u8,
    /// The identifier as a string.
    pub name: String,
    /// 16-bit attribute flags (bit 0 is prefail/oldage, bit 1 is online/offline).
    pub flags: u16,
    /// The current value or -1 if unknown.
    pub normalized: i32,
    /// The worst value of -1 if unknown.
    pub worst: i32,
    /// The threshold or -1 if unknown.
    pub threshold: i32,
    /// An interpretation of the value
    pub pretty: Option<PrettyValue>,
}

impl SmartAttribute {
    /// Whether this attribute determines if the drive is failing (`true`) or simply old (`false`).
    pub fn pre_fail(&self) -> bool {
        self.flags & 0x01 != 0
    }
    pub fn online(&self) -> bool {
        self.flags & 0x02 != 0
    }
    pub fn assessment(&self) -> SmartAssessment {
        if self.normalized > 0 && self.threshold > 0 && self.normalized <= self.threshold {
            SmartAssessment::Failing
        } else if self.worst > 0 && self.threshold > 0 && self.worst <= self.threshold {
            SmartAssessment::FailedInPast
        } else {
            SmartAssessment::Ok
        }
    }
}

impl From<RawSmartAttribute> for SmartAttribute {
    fn from(
        (id, name, flags, value, worst, threshold, pretty_value, pretty_unit, _expansion): RawSmartAttribute,
    ) -> Self {
        let pretty = PrettyUnit::try_from(pretty_unit as u8)
            .map(|unit| PrettyValue {
                value: pretty_value,
                unit,
            })
            .ok();
        SmartAttribute {
            id,
            name,
            flags,
            normalized: value,
            worst,
            threshold,
            pretty,
        }
    }
}
