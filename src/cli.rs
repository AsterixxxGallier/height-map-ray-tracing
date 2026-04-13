use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    /// Maximum link length
    #[arg(short, long, default_value_t = f64::INFINITY)]
    pub max_link_length: f64,
}
