use khonsu_tools::{
    universal::{anyhow, DefaultConfig},
    Commands,
};
use structopt::StructOpt;

fn main() -> anyhow::Result<()> {
    let command = Commands::from_args();
    command.execute::<DefaultConfig>()
}
