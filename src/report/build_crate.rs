use crate::build_crate::Artifact;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct BuildCrateReportRow {
    toolchain: String,
    profile: String,
    krate: String,
    binary_size: u64,
    path: String,
}

pub fn report_artifacts(artifacts: &[Artifact]) -> Vec<BuildCrateReportRow> {
    artifacts
        .iter()
        .map(|a| {
            let meta = std::fs::metadata(&a.output_path).unwrap();
            let binary_size = meta.len();

            BuildCrateReportRow {
                toolchain: a.toolchain.clone(),
                profile: a.profile.clone(),
                krate: a.crate_name.clone(),
                binary_size,
                path: a.output_path.to_str().unwrap().to_string(),
            }
        })
        .collect::<Vec<_>>()
}
