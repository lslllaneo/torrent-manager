mod cli;
mod meta;
mod req;

use crate::cli::Cli;
use clap::Parser;
use meta::get_linked_files;

fn main() {
    let cli = Cli::parse();

    println!("cli: {:?}", cli);

    if let Ok(map) = get_linked_files(&cli.source_dir, &cli.dest_dir) {
        let json = serde_json::to_string(&map).unwrap();
        println!("linked files: {}", json);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn http_get() {
        assert_ne!(2 + 2, 5);
    }

    #[test]
    fn add_test() {
        assert_eq!(2 + 2, 5);
    }
}
