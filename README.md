# tidal-dl

Tidal-dl is a rust implementation of the python script [Tidal-Media-Downloader](https://github.com/yaronzz/Tidal-Media-Downloader)


Configs are stored in `~/.config/tidal-dl/config.toml`
## Config Setup
`download_path` will expand env variables along with shell accelerators such at `~`. In addition to specify the format to save tracks in, you can use the following tokens

{artist}
{artist_id}
{album}
{album_id}
{track_num}
{track_name}
{track_id}
{quality}
Example Values: 
`$HOME/Music/{artist}/{album}/{track_num} - {track_name}[{track_id}]`
`$MUSIC_DIR/{artist} - [{artist_id}]/{album}/{track_name}`

``