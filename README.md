# tdl

Tdl is a rust implementation of the python script [Tidal-Media-Downloader](https://github.com/yaronzz/Tidal-Media-Downloader).


Configs are stored in `~/.config/tidal-dl/config.toml`
## Config Setup

`download_path` will expand env variables along with shell accelerators such at `~`. In addition to specify the format to save tracks in, you can use the following tokens
Artist: 
`{artist}`
`{artist_id}`
Album: 
`{album}`
`{album_id}`

Track:
`{track_num}`
`{track_name}`
`{track_id}`
`{quality}`

Example Values: 
- `$HOME/Music/{artist}/{album}/{track_num} - {track_name}[{track_id}]`

- `$MUSIC_DIR/{artist} - [{artist_id}]/{album}/{track_name}`


`audio_quality` expects one of the following values ordered in descending quality:
- `HI_RES`
- `LOSSLESS`
- `HIGH`
- `LOW`

``