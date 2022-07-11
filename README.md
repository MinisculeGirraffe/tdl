# tidal-dl

Tidal-dl is a rust implementation of the python script [Tidal-Media-Downloader](https://github.com/yaronzz/Tidal-Media-Downloader)


Configs are stored in `~/.config/tidal-dl/config.toml`
## Config Setup
`download_path` will expand env variables along with shell accelerators such at `~`. In addition to specify the format to save tracks in, you can use the following tokens

{artist}
{album}
{track_num}
{track_name}

Default value: `$HOME/Music/{artist}/{album}/{track_num} - {track_name}`