mod support;

use opsis_capture::film::draws_webgl;
use opsis_compile::nodes::output::Gl;
use opsis_engine::paths::downloaded_browser;
use support::engine_dir;

#[tokio::test]
#[ignore = "needs the dev engine"]
async fn finds_webgl_drawing_where_the_browser_has_it() {
    let temp = tempfile::tempdir().unwrap();
    let browser = downloaded_browser(&engine_dir()).unwrap();
    assert!(draws_webgl(&browser, temp.path(), Gl::Swangle).await);
}
