use futures::{future, StreamExt};
use std::io::Write;
use structopt::StructOpt;
use uclock::Color;

#[derive(StructOpt)]
struct Opt {
    #[structopt(short, long, default_value = "white")]
    color: Color,
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();

    let out = std::io::stdout();
    let mut out = out.lock();

    uclock::clock_stream(opt.color)
        .for_each(|s| {
            print!("{}", s);
            out.flush().unwrap();
            future::ready(())
        })
        .await;
}
