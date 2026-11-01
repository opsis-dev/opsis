use opsis_compile::nodes::browser::ClockMode;
use opsis_runner::context::now_ms;
use serde_json::{Value, json};

use crate::browser::Session;
use crate::cdp::Result;
use crate::page_scripts::{HOLD_PAGE_CLOCK, call};
use crate::playwright::CLOCK_SOURCE;

#[derive(Debug, Default)]
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
                now_ms()
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

    pub async fn hold_for_frames(&mut self, session: &Session) -> Result<()> {
        let time = if self.installed {
            None
        } else {
            let now = now_ms();
            let script = clock_script();
            session
                .add_init_script(&format!(
                    "if (window === window.top) {{ {script}\nglobalThis.__pwClock.controller.log('install', {now}, {now}); }}"
                ))
                .await?;
            session.tab.evaluate_main::<Value>(&script).await?;
            self.installed = true;
            Some(now)
        };
        session
            .tab
            .evaluate_main::<Value>(&call(
                HOLD_PAGE_CLOCK,
                &json!({ "time": time, "keepFixed": true }),
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

pub fn wall_ms() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0.0, |since| since.as_secs_f64() * 1000.0)
}
