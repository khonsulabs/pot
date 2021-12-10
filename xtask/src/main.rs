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
    fn ignore_paths() -> Vec<String> {
        vec![String::from("pot/examples/*")]
    }
}
