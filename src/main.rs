use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "")]
    server_name: String,
}

fn main() {
    let args = Args::parse();

    if args.server_name.is_empty() {
        println!("running as a server");
        return;
    }
    else {
        println!("running as a client");
        return;
    }
}