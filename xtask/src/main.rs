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
        } => CodeCoverage::<code_coverage::DefaultConfig>::execute(install_dependencies),
        Commands::InstallPreCommitHook | Commands::Audit { .. } => todo!(),
    }
}
