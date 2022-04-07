/// create report for different cmds
/// Report here means print a list of flat json output, which can be easily displayed by nushell or
/// dump into a sqlite db for further investigate
use crate::run::RunResult;
use itertools::Itertools;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct RunReportRow {
    toolchain: String,
    profile: String,
    krate: String,
    cmd: String,
    binary_size: u64,
    duration_ms_hist_min: u64,
    duration_ms_hist_p50: u64,
    duration_ms_hist_p90: u64,
    duration_ms_hist_max: u64,
}

/// convert `RunResult` to flat report rows
/// aggregates all durations into histogram
pub fn report_run_results(result: RunResult) -> Vec<RunReportRow> {
    let mut rows = vec![];

    for (_k, group) in &result
        .results
        .into_iter()
        .sorted_by_key(|x| x.cmd.clone())
        .group_by(|it| it.cmd.clone())
    {
        for (_k, group) in &group
            .into_iter()
            .group_by(|it| format!("{}_{}", it.toolchain, it.profile))
        {
            let mut krate: String = String::new();
            let mut toolchain: String = String::new();
            let mut profile: String = String::new();
            let mut histogram = histogram::Histogram::new();
            let mut cmd = String::new();
            let mut binary_size: u64 = 0;

            for it in group.into_iter() {
                histogram.increment(it.duration_ms).unwrap();
                krate = it.krate;
                binary_size = it.binary_size;
                toolchain = it.toolchain;
                profile = it.profile;
                cmd = it.cmd;
            }

            rows.push(RunReportRow {
                toolchain,
                profile,
                krate,
                cmd,
                binary_size,
                duration_ms_hist_min: histogram.minimum().unwrap(),
                duration_ms_hist_p50: histogram.percentile(50.0).unwrap(),
                duration_ms_hist_p90: histogram.percentile(90.0).unwrap(),
                duration_ms_hist_max: histogram.maximum().unwrap(),
            })
        }
    }

    rows
}
