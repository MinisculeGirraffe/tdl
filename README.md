# tdl

Tdl is a rust implementation of the python script [Tidal-Media-Downloader](https://github.com/yaronzz/Tidal-Media-Downloader).


Time Comparisons
tidal-dl -l https://tidal.com/browse/album/53172142  16.12s user 6.16s system 45% cpu 48.954 total
./tdl --url https://tidal.com/browse/album/53172142  0.74s user 0.77s system 16% cpu 9.343 total


/tdl --url https://tidal.com/browse/album/192059802  3.04s user 2.76s system 18% cpu 31.713 total
Configs are stored in `~/.config/tidal-dl/config.toml`
## Config Setup

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