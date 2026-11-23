pub mod address;
mod folders;
mod real;
mod slashed;

pub use folders::{APP_FOLDER, HostFolders, USER_DATA_ENV, host_folders, user_data};
pub use real::{plain, real_folder};
pub use slashed::slashed;
