use clap::{arg, value_parser, Arg, Command};

pub fn cli() -> Command<'static> {
    Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand_required(true)
        .subcommand(get())
        .subcommand(search())
        .subcommand(Command::new("login"))
        .subcommand(Command::new("logout"))
}

fn get() -> Command<'static> {
    Command::new("get")
        .arg(
            arg!(<URL>)
                .multiple_values(true)
                .required(true)
                .min_values(1)
                .value_parser(clap::builder::NonEmptyStringValueParser::new())
                .help("The Tidal URL to download"),
        )
        .arg(
            arg!(--concurrent <VALUE>)
                .short('c')
                .required(false)
                .help("Number of songs to download concurrently"),
        )
}

fn search() -> Command<'static> {
    Command::new("search")
        .arg(
            Arg::new("query")
                .takes_value(true)
                .required(true)
                .value_parser(clap::builder::NonEmptyStringValueParser::new())
                .help("Term to search for"),
        )
        .arg(
            Arg::new("filter")
                .long("filter")
                .short('f')
                .value_parser(clap::builder::PossibleValuesParser::new([
                    "all", "artist", "album", "track", "playlist",
                ]))
                .takes_value(true)
                .help("Type of results to return from search"),
        )
        .arg(
            Arg::new("max")
                .long("max")
                .short('m')
                .takes_value(true)
                .value_parser(value_parser!(u32))
                .help("Maximum number of items to return"),
        )
}
