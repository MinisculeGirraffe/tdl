use anyhow::Error;
use std::{fmt, str::FromStr};
#[derive(Debug)]
pub struct Action {
    pub kind: ActionKind,
    pub id: usize,
}
impl FromStr for Action {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url_parts: Vec<&str> = s.split('/').collect();
        let [kind, id]: [_; 2] = url_parts[url_parts.len() - 2..].try_into()?;
        Ok(Self {
            kind: ActionKind::from_str(kind)?,
            id: usize::from_str(id)?,
        })
    }
}
#[derive(Debug)]
pub enum ActionKind {
    Track,
    Album,
    Artist,
}
impl FromStr for ActionKind {
    type Err = Error;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "track" => Ok(ActionKind::Track),
            "album" => Ok(ActionKind::Album),
            "artist" => Ok(ActionKind::Artist),
            _ => Err(Error::msg("No action kind for type")),
        }
    }
}

impl fmt::Display for ActionKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let str = match self {
            ActionKind::Track => "track",
            ActionKind::Album => "album",
            ActionKind::Artist => "artist",
        };
        fmt.write_str(str)?;
        Ok(())
    }
}
