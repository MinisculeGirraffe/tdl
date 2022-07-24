use clap::{
    arg,
    builder::{NonEmptyStringValueParser, PossibleValuesParser, RangedU64ValueParser},
    value_parser, Arg, Command,
};

pub fn cli() -> Command<'static> {
    Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand_required(true)
        .subcommand(get())
        .subcommand(search())
        .subcommand(
            Command::new("login").about(
                "Displays the login prompt or re-authenticates with the current access token",
            ),
        )
        .subcommand(
            Command::new("logout").about("Logs out via the TIDAL API and resets the login config"),
        )
}

fn get() -> Command<'static> {
    Command::new("get")
        .about("Downloads files from the provided TIDAL links")
        .arg(
            arg!(<URL>)
                .multiple_values(true)
                .required(true)
                .min_values(1)
                .value_parser(NonEmptyStringValueParser::new())
                .help("The Tidal URL to download"),
        )
        .arg(
            arg!(--concurrent <VALUE>)
                .short('c')
                .required(false)
                .value_parser(RangedU64ValueParser::<u8>::new().range(1..10))
                .help("Number of songs to download concurrently"),
        )
}

fn search() -> Command<'static> {
    Command::new("search")
        .about("Searches the TIDAL API")
        .arg(
            Arg::new("query")
                .takes_value(true)
                .required(true)
                .value_parser(NonEmptyStringValueParser::new())
                .help("Term to search for"),
        )
        .arg(
            Arg::new("filter")
                .long("filter")
                .short('f')
                .value_parser(PossibleValuesParser::new([
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
