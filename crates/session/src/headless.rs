use std::path::PathBuf;

use crate::Session;

pub fn open_project(app: &Session, dir: PathBuf) {
    let app = app.clone();
    crate::runtime::spawn(async move {
        match crate::api::open_and_show(&app, &dir).await {
            Ok(()) => log::info!("headless: serving {}", dir.display()),
            Err(error) => {
                log::error!("headless: {error}");
                app.platform().exit(1);
            }
        }
    });
}
