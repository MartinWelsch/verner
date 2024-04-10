use anyhow::Result;
use gix::{ObjectId, Repository};
use verner_core::{semver::{SemVersion, SemVersionInc}, VersionInc};



struct LogIterator<'a>
{
    cfg: &'a Config,
    repo: &'a gix::Repository,
    rev_walk: gix::revision::Walk<'a>
}



impl<'a> LogIterator<'a>
{
    pub fn new(cfg: &'a Config, repo: &'a gix::Repository, start_at: gix::hash::ObjectId) -> Result<Self, gix::revision::walk::Error>
    {
        Ok(Self {
            cfg,
            repo,
            rev_walk: repo.rev_walk([start_at]).all()?
        })
    }


    fn solve_inc_for_commit(&self, rev: &gix::revision::walk::Info<'a>) -> VersionInc<SemVersion, SemVersionInc>
    {
        VersionInc::Add(SemVersionInc::Build(1))
    }
}

impl Iterator for LogIterator<'_>
{
    type Item = VersionInc<SemVersion, SemVersionInc>;

    fn next(&mut self) -> Option<Self::Item>
    {
        match self.rev_walk.next()
        {
            Some(e) => match e
            {
                Ok(ref rev) => Some(self.solve_inc_for_commit(rev)),
                Err(_) => None,
            },
            None => None,
        }
    }
}



#[derive(Default)]
struct Config
{
    branch_tags: Vec<(String, String)>
}

impl Config
{

    fn get_branch_tag(&self, branch_name: &str) -> Result<Option<String>>
    {
        for (re, tag) in self.branch_tags.as_slice()
        {
            let regex = regex::Regex::new(&re)?;
            if regex.is_match(branch_name)
            {
                return Ok(Some(tag.to_string()));
            }
        }

        Ok(None)
    }
}

fn main()
{
    let mut cfg = Config::default();

    cfg.branch_tags.push(("git-semver.*".into(), "g".into()));


    let repo = gix::discover(std::env::current_dir().unwrap()).unwrap();
    let origin_id: ObjectId = repo.head_id().unwrap().into();
    let mut log_iter = LogIterator::new(&cfg, &repo, origin_id).unwrap();
    let mut v = verner_core::resolve_version(&mut log_iter);
    
    if let Ok(Some(head_ref)) = repo.head_ref()
    {
        let tag = cfg.get_branch_tag(&head_ref.name().as_bstr().to_string());
        
        if let Ok(Some(tag)) = tag
        {
            v = v.tag(&tag);
        }
    
    }
    
    
    println!("Version: {}", v);
}

