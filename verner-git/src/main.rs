use gix::{ObjectId, Repository};
use verner_core::{semver::{SemVersion, SemVersionInc}, VersionInc};



struct LogIterator<'a>
{
    repo: &'a gix::Repository,
    rev_walk: gix::revision::Walk<'a>
}



impl<'a> LogIterator<'a>
{
    pub fn new(repo: &'a gix::Repository, start_at: gix::hash::ObjectId) -> Result<Self, gix::revision::walk::Error>
    {
        Ok(Self { 
            repo,
            rev_walk: repo.rev_walk([start_at]).all()?
        })
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
                Ok(ref rev) => Some(solve_inc_for_commit(&self.repo, rev)),
                Err(_) => None,
            },
            None => None,
        }
    }
}

fn solve_inc_for_commit<'a>(repo: &Repository, rev: &gix::revision::walk::Info<'a>) -> VersionInc<SemVersion, SemVersionInc>
{
    println!("{}", rev.object().unwrap().message().unwrap().title);
    VersionInc::Add(SemVersionInc::Minor(1))
}


#[derive(Default)]
struct Config;

fn main()
{
    let cfg = Config::default();
    let repo = gix::discover(std::env::current_dir().unwrap()).unwrap();
    let origin_id: ObjectId = repo.head_id().unwrap().into();
    let mut log_iter = LogIterator::new(&repo, origin_id).unwrap();
    let v = verner_core::resolve_version(&mut log_iter);
    println!("Version: {}", v);
}

