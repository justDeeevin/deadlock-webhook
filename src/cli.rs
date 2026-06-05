#[derive(clap::Parser)]
#[command()]
pub struct Args {
    // FIXME: Is the first entry always the latest?
    #[arg(default_value_t = 0)]
    pub index: usize,
    #[arg(long)]
    pub dry: bool,
    #[arg(short, long)]
    pub force: bool,
}
