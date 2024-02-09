use std::{collections::HashMap, fmt::Display};

use anyhow::Result;
use gix::Repository;
use serde::{Serialize, Deserialize};

use crate::walker::{Clue, ClueWalker};


extern crate gix;

#[derive(PartialEq, Serialize, Deserialize)]
pub struct Config
{
    #[serde(default)]
    origins: Vec<String>,
    #[serde(default)]
    branches: HashMap<String, BranchConfig>
}

#[derive(PartialEq, Serialize, Deserialize)]
pub struct BranchConfig
{
    regex: String,
}


struct TagInfo
{
    name: String,
    version: semver::Version,
}

impl Display for TagInfo
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Tag(")?;
        f.write_str(&self.name)?;
        f.write_str("): ")?;
        f.write_str(&self.version.to_string())?;
        Ok(())
    }
}

impl TagInfo
{
    fn new(reference: &gix::Reference, version: semver::Version) -> Self
    {
        TagInfo { 
            name: reference.name().shorten().to_string(),
            version: version
        }
    }
}

struct Trace
{
    enabled: bool
}

impl Trace
{
    fn found_tag(&self, tag: &TagInfo)
    {
        if !self.enabled
        {
            return;
        }

        println!("found tag {}", tag);
    }
}

pub struct GitLogWalker<'walk>
{
    repo: &'walk gix::Repository,
    log_walk: gix::revision::Walk<'walk>,
    config: &'walk Config,
    tags: HashMap<gix::hash::ObjectId, TagInfo>,
    trace: &'walk Trace
}

impl<'walk> GitLogWalker<'walk>
{
    fn create(config: &'walk Config, repo: &'walk gix::Repository, start_commit: gix::Id<'walk>, trace: &'walk Trace) -> Result<GitLogWalker<'walk>>
    {
        let mut walker = GitLogWalker
        {
            repo: repo,
            log_walk: repo.rev_walk([start_commit])
                            .use_commit_graph(true)
                            .all()?,
            config: config,
            tags: HashMap::new(),
            trace: trace
        };
        walker.load_tags()?;
        Ok(walker)
    }

    fn load_tags(&mut self) -> anyhow::Result<()>
    {
        for tag_res in self.repo.references()?.tags()?
        {
            if let Ok(tag) = tag_res
            {
                if let Some(id) = tag.try_id()
                {
                    self.tags.insert(id.object()?.id, TagInfo::new(&tag, semver::Version { major: 0, minor: 0, patch: 0, pre: semver::Prerelease::EMPTY, build: semver::BuildMetadata::EMPTY }));
                }
            }
        }

        Ok(())
    }
}

impl<'walk> ClueWalker<'walk> for GitLogWalker<'walk>
{
    fn next_clue(&mut self) -> Result<Clue<'walk>>
    {
        match self.log_walk.next()
        {
            Some(res) =>
            {
                let rev = res?;
                
                

                Ok(Clue::Inc(crate::version::Increment::Patch))
            },
            None => Ok(Clue::None),
        }
    }
}

pub struct ClueBox<'a>
{
    config: &'a crate::config::Config,
    repo: Repository,
    trace: Trace
}

pub fn search_clues(config: &crate::config::Config) -> Result<ClueBox>
{
    Ok(ClueBox {
        config: config,
        repo: gix::discover(std::env::current_dir()?)?,
        trace: Trace { enabled: true }
    })
}

impl<'a> ClueBox<'a>
{
    pub fn walk(&'a mut self) -> Result<GitLogWalker<'a>>
    {
        let start_commit = self.repo.head_commit()?.id();
        
        Ok(
            GitLogWalker::create(
                &self.config.git,
                &self.repo,
                start_commit,
                &self.trace
                )?
        )
    }
}