use opsis_compile::mix::plan::MixPlan;
use opsis_media::mix::graph::{
    LoudnessMeasure, PcmCodec, Plan, Taps, bus_job, channel_job, job_args, loudness_measure_job,
    master_job, measure_args, parse_loudnorm,
};
use serde_json::{Value, json};

use crate::common::{cases, text, typed, with};

const MIX: &str = "media/mix.json";

fn taps(pre: &str, post: &str) -> Taps {
    Taps {
        pre: pre.into(),
        post: post.into(),
    }
}

fn jobs(plan: &MixPlan, stored: &Value) -> Value {
    let shape = Plan {
        fps: plan.fps,
        duration_in_frames: plan.duration_in_frames,
    };
    let channels: Vec<Value> = plan
        .channels
        .iter()
        .map(|channel| {
            let sources = channel
                .sounds
                .iter()
                .map(|s| format!("/project/{}", s.src))
                .collect();
            let job = channel_job(shape, channel, sources);
            let args = job_args(
                &job,
                "/stems/out.wav",
                PcmCodec::F32,
                Some(&taps("/t/pre.wav", "/t/post.wav")),
                Some(10.5),
            );
            json!({ "id": channel.id, "job": job, "args": args })
        })
        .collect();
    let buses: Vec<Value> = plan
        .buses
        .iter()
        .map(|bus| {
            let job = bus_job(shape, bus, &|c| format!("/stems/{c}.wav"));
            let args = job_args(&job, "/stems/bus.wav", PcmCodec::F32, None, None);
            json!({ "id": bus.id, "job": job, "args": args })
        })
        .collect();
    let bus_files: Vec<String> = plan
        .buses
        .iter()
        .map(|b| format!("/stems/{}.wav", b.id))
        .collect();
    let master = cases(&stored["master"], |expected| {
        let measured: Option<LoudnessMeasure> = typed(&expected["measured"]);
        let job = master_job(shape, &plan.master, &bus_files, measured.as_ref());
        let limit = plan.duration_in_frames / plan.fps + 1.0;
        let args = job_args(
            &job,
            "/stems/master.wav",
            PcmCodec::S16,
            Some(&taps("/t/a.wav", "/t/b.wav")),
            Some(limit),
        );
        json!({ "measured": expected["measured"], "job": job, "args": args })
    });
    let measure = loudness_measure_job(shape, &plan.master, &bus_files)
        .map(|job| json!({ "args": measure_args(&job), "job": job }));
    json!({ "channels": channels, "buses": buses, "master": master, "measure": measure })
}

#[test]
fn mix_jobs_and_arguments_are_as_recorded() {
    let f = opsis_golden::read(MIX);
    let plans = cases(&f["plans"], |case| {
        let plan: MixPlan = typed(&case["plan"]);
        with(case, &[("jobs", jobs(&plan, &case["jobs"]))])
    });
    let loudnorm = cases(&f["loudnorm"], |case| {
        let measured = parse_loudnorm(text(&case["stderr"]));
        with(case, &[("measured", json!(measured))])
    });
    opsis_golden::check(MIX, &json!({ "plans": plans, "loudnorm": loudnorm }));
}
