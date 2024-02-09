pub enum Clue<'a>
{
    Inc(crate::version::Increment),
    Hard(semver::Version),
    Hop(&'a dyn ClueWalker<'a>),
    None
}

pub trait ClueWalker<'a>
{
    fn next_clue(&mut self) -> anyhow::Result<Clue<'a>>;
}

