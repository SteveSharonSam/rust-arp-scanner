use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    #[arg(short, long)]
    pub list: bool,

    #[arg(short, long, default_value_t = 0)]
    pub iface: usize,
}
