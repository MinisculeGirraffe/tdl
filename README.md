# tdl
tdl is a rust implementation of the Python Script [Tidal-Media-Downloader](https://github.com/yaronzz/Tidal-Media-Downloader).

## Overview 

tdl offers significant performance improvements over the original python script by utilizing asynchronous multi-threaded concurrency while also being 50-100 times more CPU efficient.

## Usage

- Get a single item
```
tdl get <URL>
tdl get https://tidal.com/browse/album/129835816
```

- Get multiple items
```
tdl get <URL1> <URL2> 
tdl get https://tidal.com/browse/album/129835816 https://tidal.com/browse/album/147102710  
```

### Benchmarks

Benchmarks were performed on a Linode Nanode 1GB with a 40gbps downlink.

#### Download files (Multi/Single Threaded):

tdl get https://tidal.com/browse/artist/5416094 -w 1 -d 3  7.25s user 9.75s system 25% cpu 1:06.83 total

#### Legend
- `real`: The actual time spent in running the process from start to finish, as if it was measured by a human with a stopwatch
- `user`: The cumulative time spent by all the CPUs during the computation. Time spent waiting for I/O does not count towards this counter
- `sys`: The cumulative time spent by all the CPUs during system-related tasks such as memory allocation.
- `real speedup`: Multiplier of total time saved.
- `user speedup`: Multiplier of total CPU time saved. 


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
