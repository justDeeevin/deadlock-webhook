#[derive(clap::Parser)]
#[command(disable_help_subcommand = true)]
pub struct Args {
    #[arg(default_value_t = 0)]
    pub index: usize,
    #[arg(long)]
    pub dry: bool,
}
