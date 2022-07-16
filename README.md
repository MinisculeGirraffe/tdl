# tdl

tdl is a rust implementation of the python script [Tidal-Media-Downloader](https://github.com/yaronzz/Tidal-Media-Downloader).

tdl offers significant performance improvements over the original python script by utilizing async and green threads for downloading files.

Time Comparisons:
 
| command | total | user | system |
| ------- | ----- | ---- | ------ |
| tdl concurrency 5 | 12.857 | 1.75s | 2.80s |
| tdl concurrency 3 | 16.291s | 3.24s | 1.94s |
| tdl concurrency 1 | 30.902s | 1.76s | 2.82s |
| tidal-dl | 51.731 | 12.960s | 5.38s |


## Config Setup

Configs are stored in `~/.config/tidal-dl/config.toml`

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
        - `1`..`255`


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
    - `0`..`255`