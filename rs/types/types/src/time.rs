// DFN-467: clippy complains about the code generated by derive(Arbitrary)
#![cfg_attr(test, allow(clippy::unit_arg))]
//! Defines the [`Time`] type used by the Internet Computer.

use ic_constants::{MAX_INGRESS_TTL, PERMITTED_DRIFT};
#[cfg(test)]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;
use std::time::Duration;

/// Time since UNIX_EPOCH (in nanoseconds). Just like 'std::time::Instant' or
/// 'std::time::SystemTime', [Time] does not implement the [Default] trait.
/// Please use `ic_test_utilities::mock_time` if you ever need such a value.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Time(u64);

/// The unix epoch.
pub const UNIX_EPOCH: Time = Time(0);

impl std::ops::Add<Duration> for Time {
    type Output = Time;
    fn add(self, dur: Duration) -> Time {
        Time::from_duration(Duration::from_nanos(self.0) + dur)
    }
}

impl std::ops::AddAssign<Duration> for Time {
    fn add_assign(&mut self, other: Duration) {
        *self = Time::from_duration(Duration::from_nanos(self.0) + other)
    }
}

impl std::ops::Sub<Time> for Time {
    type Output = std::time::Duration;

    fn sub(self, other: Time) -> std::time::Duration {
        let lhs = Duration::from_nanos(self.0);
        let rhs = Duration::from_nanos(other.0);
        lhs - rhs
    }
}

impl std::ops::Sub<Duration> for Time {
    type Output = Time;

    fn sub(self, other: Duration) -> Time {
        let time = Duration::from_nanos(self.0);
        Time::from_duration(time - other)
    }
}

impl Time {
    /// Number of nanoseconds since UNIX EPOCH
    pub fn as_nanos_since_unix_epoch(self) -> u64 {
        self.0
    }

    pub const fn from_nanos_since_unix_epoch(nanos: u64) -> Self {
        Time(nanos)
    }

    /// A private function to cast from [Duration] to [Time].
    fn from_duration(t: Duration) -> Self {
        Time(t.as_nanos() as u64)
    }
}

impl TryFrom<Duration> for Time {
    type Error = &'static str;
    fn try_from(d: Duration) -> Result<Self, Self::Error> {
        u64::try_from(d.as_nanos())
            .or(Err(
                "Duration is too large to be converted into a u64 of nanoseconds!",
            ))
            .map(Time)
    }
}

impl From<Time> for Duration {
    fn from(val: Time) -> Self {
        Duration::from_nanos(val.0)
    }
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use chrono::{TimeZone, Utc};
        use std::convert::TryInto;

        match self.0.try_into() {
            Ok(signed) => write!(f, "{}", Utc.timestamp_nanos(signed)),
            Err(_) => write!(f, "{}ns", self.0),
        }
    }
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}ns", self.0)
    }
}

/// Returns the current time.
///
/// WARNING: this function should not be used in any deterministic part of the
/// IC as it accesses system time, which is non-deterministic between nodes.
pub fn current_time() -> Time {
    let start = std::time::SystemTime::now();
    let since_epoch = start
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time wrapped around");
    UNIX_EPOCH + since_epoch
}

/// A utility function to help set the expiry time when creating an
/// SignedIngress message from scratch.  Returns the current time and expiry
/// time.  The expiry time is set from the current system time + the maximum
/// amount of time ingress messages are allowed to stay alive for - 60 seconds.
///
/// Subtracting 60 seconds is because this uses the system time and not the
/// block time. The block time is going to lag behind the system time by
/// some amount, so if you don't subtract you have an expiry time that is too
/// far in the future when the expiry time is compared against block_time +
/// MAX_INGRESS_TTL, and the message will be rejected.
///
/// 60 seconds is hopefully enough leeway.
///
///
/// WARNING: this function should not be used in any deterministic part of the
/// IC as it accesses system time, which is non-deterministic between nodes.
//
// This function is made public to be able to use it for testing purposes.
pub fn current_time_and_expiry_time() -> (Time, Time) {
    let start = std::time::SystemTime::now();
    let since_epoch = start
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time wrapped around");
    (
        UNIX_EPOCH + since_epoch,
        UNIX_EPOCH + (since_epoch + MAX_INGRESS_TTL - PERMITTED_DRIFT),
    )
}
