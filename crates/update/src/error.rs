#[derive(Debug, thiserror::Error)]
pub enum UpdateError {
    #[error("Couldn't check for updates. Check the internet connection and try again.")]
    Network(#[source] reqwest::Error),
    #[error(
        "This computer has no certificate authorities installed, so it can't make secure connections."
    )]
    NoCertificates,
    #[error("The update server answered with an error ({0}). Try again later.")]
    Status(u16),
    #[error("The update information couldn't be read.")]
    Unreadable(#[source] serde_json::Error),
    #[error("The update information names a version Opsis can't read.")]
    Version(#[source] semver::Error),
    #[error("Version {0} has no update for this computer yet.")]
    NoFile(String),
    #[error("The downloaded update isn't signed by Opsis, so it wasn't kept.")]
    Signature,
    #[error("The update was stopped.")]
    Cancelled,
    #[error("This copy of Opsis can't update itself.")]
    NotSelfUpdating,
    #[error("Couldn't find where Opsis is installed.")]
    NoInstallPath,
    #[error("Couldn't install the update: {0}")]
    Io(#[from] std::io::Error),
}
