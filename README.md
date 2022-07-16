# tdl

Tdl is a rust implementation of the python script [Tidal-Media-Downloader](https://github.com/yaronzz/Tidal-Media-Downloader).

tdl offers significant performance improvements over the original python script by utilizing async and green threads for downloading files.

Time Comparisons:

`tdl --url https://tidal.com/browse/album/197298621 --concurrent 3`
 1.94s user 3.24s system 31% cpu 16.291 total

`tidal-dl -l https://tidal.com/browse/album/197298621`
 12.96s user 5.38s system 35% cpu 51.731 total

## Config Setup
Configs are stored in `~/.config/tidal-dl/config.toml`
`download_path` will expand env variables along with shell accelerators such at `~`. In addition to specify the format to save tracks in, you can use the following tokens
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


`audio_quality` expects one of the following values ordered in descending quality:
- `HI_RES` (24bit/96kHz MQA encoded FLAC)
- `LOSSLESS` (1411kbps|16bit/44.1kHz FLAC/ALAC)
- `HIGH` (320kbps AAC)
- `LOW` (96kbps AAC)


`download_cover` Download a cover.jpg in an album folder