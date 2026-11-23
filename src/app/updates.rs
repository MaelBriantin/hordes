//! A newer version of the game, if one is out: asked of the releases once, as the game starts,
//! out of the way of the title screen; not a word if it can't be told (no network, no release).
//! On the title screen, `U` then quits and installs it, as `hordes update` does.

use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

use axoupdater::{AxoUpdater, ReleaseSource, ReleaseSourceType};

/// Where the releases are: the game's repository on GitHub.
const OWNER: &str = "MaelBriantin";
const REPOSITORY: &str = "hordes";
const APP: &str = "hordes";
/// How long the question may take before it is given up.
const TIMEOUT: Duration = Duration::from_secs(5);

/// A version: major, minor, patch.
pub type Version = (u64, u64, u64);

/// This version of the game.
pub fn current() -> Version {
    parse(env!("CARGO_PKG_VERSION")).unwrap_or_default()
}

/// A version as `0.10.1` or `v0.10.1` writes it.
pub fn parse(text: &str) -> Option<Version> {
    let mut parts = text.trim().trim_start_matches('v').split('.');
    let mut next = || parts.next()?.split(['-', '+']).next()?.parse().ok();
    Some((next()?, next()?, next()?))
}

/// The latest version out, if newer than `current`.
pub fn newer(latest: Version, current: Version) -> Option<Version> {
    (latest > current).then_some(latest)
}

/// Asks, on a thread of its own, for the latest version out: the answer, a newer version or
/// none, comes on the receiver once there.
pub fn ask() -> Receiver<Option<Version>> {
    let (send, receive) = mpsc::channel();
    thread::spawn(move || {
        let _ = send.send(latest().and_then(|latest| newer(latest, current())));
    });
    receive
}

/// The latest version out, as the releases tell it; none if they can't be asked.
fn latest() -> Option<Version> {
    let mut updater = AxoUpdater::new_for(APP);
    updater.set_release_source(ReleaseSource {
        release_type: ReleaseSourceType::GitHub,
        owner: OWNER.to_owned(),
        name: REPOSITORY.to_owned(),
        app_name: APP.to_owned(),
    });
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .ok()?;
    let found = runtime.block_on(async {
        tokio::time::timeout(TIMEOUT, updater.query_new_version())
            .await
            .ok()?
            .ok()?
            .map(ToString::to_string)
    })?;
    parse(&found)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_version_is_read_with_or_without_its_v_and_its_suffix() {
        assert_eq!(parse("0.10.1"), Some((0, 10, 1)));
        assert_eq!(parse("v0.11.0"), Some((0, 11, 0)));
        assert_eq!(parse("1.2.3-beta.1"), Some((1, 2, 3)));
        assert_eq!(parse("nonsense"), None);
    }

    #[test]
    fn only_a_newer_version_is_told() {
        assert_eq!(newer((0, 11, 0), (0, 10, 0)), Some((0, 11, 0)));
        assert_eq!(newer((0, 10, 1), (0, 10, 0)), Some((0, 10, 1)));
        assert_eq!(newer((0, 10, 0), (0, 10, 0)), None);
        assert_eq!(newer((0, 9, 2), (0, 10, 0)), None, "older: not a word");
    }

    #[test]
    #[ignore = "asks GitHub's releases: run with --ignored"]
    fn the_latest_release_is_found_out_there() {
        let latest = latest().expect("an answer from the releases");
        assert!(latest >= (0, 9, 2), "{latest:?}");
    }
}
