use clap::{
    arg,
    builder::{
        BoolishValueParser, EnumValueParser, NonEmptyStringValueParser, PossibleValuesParser,
        RangedU64ValueParser,
    },
    value_parser, Arg, ArgMatches, Command,
};

use crate::{api::models::AudioQuality, config::CONFIG};

pub fn cli() -> Command<'static> {
    Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand_required(true)
        .subcommand(get())
        .subcommand(search())
        .subcommand(
            Command::new("login").about("Login or re-authenticates with the current access token"),
        )
        .subcommand(
            Command::new("logout").about("Logout via the TIDAL API and resets the login config"),
        )
}

fn get() -> Command<'static> {
    Command::new("get")
        .about("Downloads files from the provided TIDAL links")
        .arg(
            arg!(<URL>)
                .multiple_values(true)
                .min_values(1)
                .required(true)
                .value_parser(NonEmptyStringValueParser::new())
                .help("One or multiple space separated URLs to download"),
        )
        .arg(
            Arg::new("downloads")
                .short('d')
                .long("downloads")
                .display_order(0)
                .required(false)
                .takes_value(true)
                .value_parser(RangedU64ValueParser::<u8>::new().range(1..11))
                .value_name("number")
                .help("Number of Simultaneous file downloads"),
        )
        .arg(
            Arg::new("workers")
                .short('w')
                .long("workers")
                .display_order(0)
                .required(false)
                .takes_value(true)
                .value_parser(RangedU64ValueParser::<u8>::new().range(1..256))
                .value_name("number")
                .help("Number of in-flight requests"),
        )
        .arg(
            Arg::new("quality")
                .short('q')
                .long("quality")
                .display_order(1)
                .required(false)
                .takes_value(true)
                .value_parser(EnumValueParser::<AudioQuality>::new())
                .help("Requested audio quality of tracks"),
        )
        .arg(
            Arg::new("progress")
                .short('p')
                .long("show-progress")
                .required(false)
                .takes_value(true)
                .display_order(2)
                .value_parser(BoolishValueParser::new())
                .value_name("boolish")
                .help("Display the progress bar when downloading files"),
        )
        .arg(
            Arg::new("singles")
                .short('s')
                .long("include-singles")
                .required(false)
                .takes_value(true)
                .display_order(3)
                .value_parser(BoolishValueParser::new())
                .value_name("boolish")
                .help("Include singles with getting lists of albums"),
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
                .value_name("type")
                .takes_value(true)
                .help("Type of results to return from search"),
        )
        .arg(
            Arg::new("max")
                .long("max")
                .short('m')
                .takes_value(true)
                .value_parser(value_parser!(u32))
                .value_name("number")
                .help("Maximum number of items to return"),
        )
}

pub async fn parse_config_flags(matches: &ArgMatches) {
    let mut config = CONFIG.write().await;
    let flags = ["downloads", "workers", "progress", "singles", "quality"];
    for flag in flags {
        match flag {
            "downloads" => set_val::<u8>(&mut config.downloads, flag, matches),
            "workers" => set_val::<u8>(&mut config.workers, flag, matches),
            "progress" => set_val::<bool>(&mut config.show_progress, flag, matches),
            "singles" => set_val::<bool>(&mut config.include_singles, flag, matches),
            "quality" => set_val::<AudioQuality>(&mut config.audio_quality, flag, matches),
            _ => continue,
        };
    }
}

fn set_val<'a, T>(dst: &mut T, flag: &str, matches: &'a ArgMatches)
where
    T: Send + Sync + Copy + Clone + 'static,
{
    if let Ok(Some(v)) = matches.try_get_one::<T>(flag) {
        let _ = std::mem::replace(dst, *v);
    }
}
