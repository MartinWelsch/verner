extern crate verner_core;
extern crate gix;


struct LogIterator<'a>
{
    repo: &'a gix::Repository,

    rev_walk: gix::revision::Walk<'a>
}



impl<'a> LogIterator<'a>
{
    pub fn new(repo: &'a gix::Repository, start_at: gix::hash::ObjectId) -> Result<LogIterator<'a>, gix::revision::walk::Error>
    {
        Ok(LogIterator { repo: repo, rev_walk: repo.rev_walk([start_at]).all()? })
    }
}

impl Iterator for LogIterator<'_>
{
    type Item = verner_core::history::Entry;

    fn next(&mut self) -> Option<Self::Item>
    {
        match self.rev_walk.next() {
            Some(e) => match e {
                Ok(rev) => Some(verner_core::history::Entry::Info(verner_core::history::Info
                {
                    name: rev.id.to_string(),
                    version_hint: verner_core::version::Hint::Hard(semver::Version { major: 0, minor: 1, patch: 0, pre: semver::Prerelease::EMPTY, build: semver::BuildMetadata::EMPTY })
                })),
                Err(_) => None,
            },
            None => None,
        }
    }
}
