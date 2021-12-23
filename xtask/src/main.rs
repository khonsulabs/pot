use khonsu_tools::{
    anyhow,
    code_coverage::{self, CodeCoverage},
    Commands,
};
use structopt::StructOpt;

fn main() -> anyhow::Result<()> {
    let command = Commands::from_args();
    match command {
        Commands::GenerateCodeCoverageReport {
            install_dependencies,
        } => CodeCoverage::<CoverageConfig>::execute(install_dependencies),
        Commands::InstallPreCommitHook | Commands::Audit { .. } => todo!(),
    }
}

struct CoverageConfig;

impl code_coverage::Config for CoverageConfig {
    /// The cargo command after `cargo`.
    fn cargo_args() -> Vec<String> {
        vec![
            String::from("+nightly"),
            String::from("test"),
            String::from("--workspace"),
            String::from("--all-features"),
            String::from("--all-targets"),
        ]
    }
}
