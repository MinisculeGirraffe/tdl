# tdl
tdl is a rust implementation of the Python Script [Tidal-Media-Downloader](https://github.com/yaronzz/Tidal-Media-Downloader).

## Overview 

tdl offers significant performance improvements over the original python script by utilizing asynchronous multi-threaded concurrency via tokio the work stealing scheduler.

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

### Search



### Autocomplete

tdl will generate an autocompletion file for various shells, that can be output to the proper autocomplete directory on your system

Example:
```
tdl autocomplete -s zsh > $FPATH/tdl.zsh
```

## Config Setup

Configs are stored in `~/.config/tdl/config.toml`, and will auto-generate with the default settings when the executable is ran. 

### download_path

`download_path` will expand env variables along with shell accelerators such as `~`.

In addition to specify the format to save tracks in, you can use the following tokens:

- Artist: 
  - `{artist}`
    - Artist Name
  - `{artist_id}`
    - Unique ID from the Tidal API
- Album: 
  - `{album}`
    - Album Title
  - `{album_id}`
    - Unique ID from the Tidal API: 
  - `{album_release}`
    - Full YYYY-MM-DD of relase
  - `{album_release_year}`
    - YYYY date of album release

- Track:
  - `{track_num}`
  - `{track_name}`
  - `{track_id}`
  - `{quality}`
    -  String literal of audio_quality

Example Values: 

- `$HOME/Music/{artist}/{album} [{album_id}] [{album_release_year}]/{track_num} - {track_name}`

- `$MUSIC_DIR/{artist} - [{artist_id}]/{album}/{track_name}`

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

- `concurrency`
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
