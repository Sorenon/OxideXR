use openxr::sys as xr;

//TODO mess around a bit more with this and decide if its worth keeping or scrapping

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct InteractionProfilePath(pub xr::Path);

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TopLevelUserPath(pub xr::Path);

pub type SubactionPath = TopLevelUserPath;