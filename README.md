# tdl
tdl is a rust implementation of the Python Script [Tidal-Media-Downloader](https://github.com/yaronzz/Tidal-Media-Downloader).

## Overview 

tdl offers significant performance improvements over the original python script by utilizing asynchronous multi-threaded concurrency.

## Usage

### Getting Started

To setup an auth token, run the below command, and log in via the link output to the terminal

```
tdl login
```

To get the possible parameters for any command or sub command, run:

```
tdl --help
tdl get --help
```

### Get

Get a single item

```
tdl get <URL>
tdl get https://tidal.com/browse/album/129835816
```

Get multiple items
```
tdl get <URL1> <URL2> 
tdl get https://tidal.com/browse/album/129835816 https://tidal.com/browse/album/147102710  
```


### Autocomplete

tdl will generate an autocompletion file for various shells, that can be output to the proper autocomplete directory on your system

Example:
```
tdl autocomplete -s zsh > $FPATH/tdl.zsh
```

## Config Setup

Configs are stored in `~/.config/tdl/config.toml`, and will auto-generate with the default settings when the executable is ran. 

### download_paths

The `download_paths` section in config is used to decide where files will be placed in the file system.



`base_path` will expand env variables along with shell accelerators such as `~`. This is the parent folder all files will be placed under.

Each of the following keys will create a nested folder structure in the following template:
`{base_path}/{artist}/{album}/{track}`


Examples: 

``` toml
[download_paths]
base_path = '$HOME/Music'
artist = '{artist_name}'
album = '{album_name} [{album_id}] [{album_release_year}]'
track = '{track_num} - {track_name} - {track_volume}'
```
Resulting Naming path:
`/Users/username/Music/100 gecs/1000 gecs [129835816] [2019]/1 - 745 sticky - 1.flac`

You can also specify any token under any key so long as it's not a child. A track will be able to use any key from the album or artist. However an album won't be able to use a track key.

Keys can also be left blank to skip folder creation.

``` toml
[download_paths]
base_path = '$HOME/Music'
artist = ''
album = '{artist_name} - {album_name} [{album_id}] [{album_release_year}]'
track = '[{track_num}] - {artist_name} - {track_name} - {track_volume}'
```

Resulting Naming path:
`/Users/username/Music/100 gecs - 1000 gecs [129835816] [2019]/[1] - 100 gecs - 745 sticky - 1.flac`

Available Keys:

Artist:

|Token | Description | Example |
| ----|-----|--|
| `{artist_name}`| Artist Name| 100 Gecs
| `{artist_id}` |  Unique ID from Tidal | 10828611

Album:
|Token | Description | Example |
| ----|-----|--|
| `{album_name}`| Album Title | 1000 Gecs |
| `{album_id}` | Unique ID from Tidal | 192059802   |
| `{album_duration}` | Duration in seconds of Album |3000 | 
| `{album_tracks}` | Number of tracks in Album | 17
| `{album_explicit}`| Shortcode if album is explicit, empty if false | E |
| `{album_quality}` | String literal of `audio_quality` | HI_RES
| `{album_release}`| YYYY-MM-DD string of album release date | 2020-07-05 |
|`{album_release_year}` | YYYY string of album release | 2020 

Track: 

|Token | Description | Example |
| ----|-----|--|
  | `{track_id}` | Unique ID from Tidal | 129835817
  | `{track_name}` | Name of Track | 745 Sticky 
  | `{track_duration}` | Track Duration in Seconds | 120
  | `{track_num}` | Number track appears on album | 7 
  | `{track_volume}` | Volume number of track, if album includes multiple discs | 1 
  | `{track_isrc}` | International Standard Recording Code of track | DEZ750500205
  | `{track_explicit}` | Shortcode if album is explicit, empty if false  | E
  | `{track_quality}` | String literal of `audio_quality` | HI_RES


### audio_quality

- `audio_quality` 
  - Quality of downloaded tracks
  - Default:
    - `HI_RES`
  - Accepted Values:
    - `HI_RES` 
      - (24bit/96kHz MQA encoded FLAC)
    - `LOSSLESS` 
      - (1411kbps|16bit/44.1kHz FLAC/ALAC)
    - `HIGH` 
      - (320kbps AAC)
    - `LOW` 
      - (96kbps AAC)

### Concurrency

- `downloads`
    - Number of concurrent downloads. Not recommended to set higher than 8.
    - Default:
        - `3`
    - Accepted Values:
        - `1`..`10`


### download_cover

- `download_cover` 
  - Download a cover.jpg in an album folder
  - Default: 
    - `true`
  - Accepted Values: 
  - `true`
  - `false`

### Progress

- `show_progress`
  - Displays a progress bar when downloading files
  - Default: 
    - `true`
  - Accepted Values: 
  - `true`
  - `false`

- `progress_refresh_rate` 
  - Refresh rate in hz of the progress bar, if show_progress is set to true. Reduce this for lower CPU usage. 
  - Default:
    -  `5`
  - Accepted values: 
    - `1`..`255`
