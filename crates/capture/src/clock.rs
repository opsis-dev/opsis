use opsis_compile::nodes::browser::ClockMode;
use opsis_runner::context::now_ms;
use serde_json::Value;

use crate::browser::Session;
use crate::cdp::Result;
use crate::playwright::CLOCK_SOURCE;

#[derive(Default)]
pub struct Clock {
    installed: bool,
}

fn clock_script() -> String {
    format!(
        "(() => {{\nconst module = {{}};\n{CLOCK_SOURCE}\nif (!globalThis.__pwClock)\n  globalThis.__pwClock = (module.exports.inject())(globalThis, \"chromium\");\n}})();"
    )
}

impl Clock {
    async fn install_if_needed(&mut self, session: &Session) -> Result<()> {
        if self.installed {
            return Ok(());
        }
        let script = clock_script();
        session.add_init_script(&script).await?;
        session.tab.evaluate_main::<Value>(&script).await?;
        self.installed = true;
        Ok(())
    }

    async fn call(&mut self, session: &Session, method: &str, time_ms: i64) -> Result<()> {
        self.install_if_needed(session).await?;
        session
            .add_init_script(&format!(
                "globalThis.__pwClock.controller.log('{method}', {}, {time_ms})",
                now_ms() as i64
            ))
            .await?;
        session
            .tab
            .evaluate_main::<Value>(&format!(
                "globalThis.__pwClock.controller.{method}({time_ms})"
            ))
            .await?;
        Ok(())
    }

    pub async fn freeze(&mut self, session: &Session, mode: ClockMode, time_ms: i64) -> Result<()> {
        match mode {
            ClockMode::Fixed => self.call(session, "setFixedTime", time_ms).await,
            ClockMode::Paused => {
                self.call(session, "install", time_ms).await?;
                self.call(session, "pauseAt", time_ms).await
            }
        }
    }
}
