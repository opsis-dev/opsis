use opsis_compile::mix::plan::MixPlan;
use opsis_media::mix::graph::{
    Job, LoudnessMeasure, PcmCodec, Plan, Taps, bus_job, channel_job, job_args,
    loudness_measure_job, master_job, measure_args, parse_loudnorm,
};
use pretty_assertions::assert_eq;
use serde_json::Value;

use crate::common::{args, fixture, list, text, typed};

fn taps(pre: &str, post: &str) -> Taps {
    Taps {
        pre: pre.into(),
        post: post.into(),
    }
}

#[test]
fn mix_jobs_and_arguments_match() {
    let f = fixture("media/mix.json");
    for case in list(&f["plans"]) {
        let plan: MixPlan = typed(&case["plan"]);
        let jobs = &case["jobs"];
        let shape = Plan {
            fps: plan.fps,
            duration_in_frames: plan.duration_in_frames,
        };
        for (channel, expected) in plan.channels.iter().zip(list(&jobs["channels"])) {
            let job = channel_job(
                shape,
                channel,
                channel
                    .sounds
                    .iter()
                    .map(|s| format!("/project/{}", s.src))
                    .collect(),
            );
            assert_eq!(
                job,
                typed::<Job>(&expected["job"]),
                "{} {}",
                text(&case["name"]),
                channel.id
            );
            let made = job_args(
                &job,
                "/stems/out.wav",
                PcmCodec::F32,
                Some(&taps("/t/pre.wav", "/t/post.wav")),
                Some(10.5),
            );
            assert_eq!(made, args(&expected["args"]));
        }
        for (bus, expected) in plan.buses.iter().zip(list(&jobs["buses"])) {
            let job = bus_job(shape, bus, &|c| format!("/stems/{c}.wav"));
            assert_eq!(
                job,
                typed::<Job>(&expected["job"]),
                "{} {}",
                text(&case["name"]),
                bus.id
            );
            assert_eq!(
                job_args(&job, "/stems/bus.wav", PcmCodec::F32, None, None),
                args(&expected["args"])
            );
        }
        let bus_files: Vec<String> = plan
            .buses
            .iter()
            .map(|b| format!("/stems/{}.wav", b.id))
            .collect();
        for expected in list(&jobs["master"]) {
            let measured: Option<LoudnessMeasure> = typed(&expected["measured"]);
            let job = master_job(shape, &plan.master, &bus_files, measured.as_ref());
            assert_eq!(
                job,
                typed::<Job>(&expected["job"]),
                "{}",
                text(&case["name"])
            );
            let limit = plan.duration_in_frames / plan.fps + 1.0;
            assert_eq!(
                job_args(
                    &job,
                    "/stems/master.wav",
                    PcmCodec::S16,
                    Some(&taps("/t/a.wav", "/t/b.wav")),
                    Some(limit)
                ),
                args(&expected["args"])
            );
        }
        let measure = loudness_measure_job(shape, &plan.master, &bus_files);
        match &jobs["measure"] {
            Value::Null => assert_eq!(measure, None),
            expected => {
                let job = measure.expect("a loudness measure");
                assert_eq!(job, typed::<Job>(&expected["job"]));
                assert_eq!(measure_args(&job), args(&expected["args"]));
            }
        }
    }
    for case in list(&f["loudnorm"]) {
        assert_eq!(
            parse_loudnorm(text(&case["stderr"])),
            typed::<Option<LoudnessMeasure>>(&case["measured"]),
            "{case}"
        );
    }
}
