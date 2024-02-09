use anyhow::Result;

use crate::walker::ClueWalker;

pub enum Increment
{
    Major,
    Minor,
    Patch
}

trait VersionEx
{
    fn inc(&self, op: Increment) -> semver::Version;
}

impl VersionEx for semver::Version
{
    fn inc(&self, op: Increment) -> semver::Version
    {
        match op {
            Increment::Major => { semver::Version { major: self.major + 1, minor: 0, patch: 0, pre: self.pre.clone(), build: self.build.clone() } },
            Increment::Minor => { semver::Version { major: self.major, minor: self.minor + 1, patch: 0, pre: self.pre.clone(), build: self.build.clone() } },
            Increment::Patch => { semver::Version { major: self.major, minor: self.minor, patch: self.patch + 1, pre: self.pre.clone(), build: self.build.clone() } },
        }
    }
}

pub struct Detective<'cfg>
{
    config: &'cfg crate::config::Config,
}

impl<'cfg> Detective<'cfg>
{
    pub fn new(config: &'cfg crate::config::Config) -> Detective<'cfg>
    {
        Detective { config: config }
    }

    pub fn find_version<'det>(&'det self, clues: &mut dyn ClueWalker) -> Result<semver::Version>
    {
        let mut version = semver::Version::new(0, 0, 0);
        loop
        {
            match clues.next_clue()?
            {
                crate::walker::Clue::Inc(inc) => version = version.inc(inc),
                crate::walker::Clue::Hard(_) => todo!(),
                crate::walker::Clue::Hop(_) => todo!(),
                crate::walker::Clue::None => break,
            }
        }

        Ok(version)
    }
}