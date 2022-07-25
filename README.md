# tdl

## Overview 
tdl is a rust implementation of the python script [Tidal-Media-Downloader](https://github.com/yaronzz/Tidal-Media-Downloader).

tdl offers significant performance improvements over the original python script by utilizing async and multi threaded concurrency. tdl will download multiple files concurrently, while simultaneously pre-processing future queue items in the background to minimize time spent not downloading.


 Download Files:
| command | total | user | system | speedup | 
| ------- | ----- | ---- | ------ | ------- |
| tdl -c 5 | 0m26.355s | 0m3.144s | 0m7.063s | 1008% |
| tidal-dl (max concurrency) | 4m25.765s | 3m51.389s | 0m16.369s | 100% | 


Check Downloaded Files:
| command | total | user | system | speedup | 
| ------- | ----- | ---- | ------ | ------- |
| tdl -c 5 | 0m0.552s | 0m0.030s | 0m0.010s | 2438% |
| tidal-dl (max concurrency) |  0m13.463s | 0m7.813s | 0m0.214s | 100% | 

benchmarks were performed on a Linode Nanode 1GB with a 40gbps downlink

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
