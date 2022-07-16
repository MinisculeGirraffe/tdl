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
`download_path` will expand env variables along with shell accelerators such at `~`.

In addition to specify the format to save tracks in, you can use the following tokens:

- Artist: 
`{artist}`
`{artist_id}`
- Album: 
`{album}`
`{album_id}`

- Track:
`{track_num}`
`{track_name}`
`{track_id}`
`{quality}`

Example Values: 
- `$HOME/Music/{artist}/{album}/{track_num} - {track_name}[{track_id}]`

- `$MUSIC_DIR/{artist} - [{artist_id}]/{album}/{track_name}`

### audio_quality

`audio_quality` expects one of the following values ordered in descending quality:
- `HI_RES` (24bit/96kHz MQA encoded FLAC)
- `LOSSLESS` (1411kbps|16bit/44.1kHz FLAC/ALAC)
- `HIGH` (320kbps AAC)
- `LOW` (96kbps AAC)

### download_cover

`download_cover` Download a cover.jpg in an album folder