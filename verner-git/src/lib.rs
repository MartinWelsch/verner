#[cfg(test)] mod tests;

mod config;
pub mod cli;

use cli::ConfigPreset;
pub use config::RawConfig;

use std::{collections::HashMap, path::Path};

use anyhow::{bail, Result};
use config::{BranchMatch, Config, RawBranchConfig, RawTagConfig, TagMatch};
use git2::{Oid, Reference, Repository, Revwalk};
use verner_core::{output::{ConsoleWriter, LogLevel}, semver::{SemVersion, SemVersionInc}, VersionInc};


struct BranchSolver<'a>
{
    current_branch: &'a BranchMatch<'a>,
    version_bases: HashMap<Oid, SemVersion>,
    branch_roots: HashMap<Oid, SemVersion>,
    tags: HashMap<Oid, TagMatch<'a>>,
    rev_walk: Revwalk<'a>
}

impl<'a> BranchSolver<'a>
{
    pub fn new(cfg: &'a Config, repo: &'a Repository, branch: &'a BranchMatch) -> Result<Self>
    {
        let mut branches = Vec::new();
        for b in repo.branches(Some(git2::BranchType::Local))?
        {
            let (b, _) = b?;
            branches.push(b.into_reference());
        }

        let mut rev_walk = repo.revwalk()?;
        rev_walk.push(branch.tip())?;

        let mut solver = Self
        {
            current_branch: branch,
            version_bases: Default::default(),
            branch_roots: Default::default(),
            rev_walk,
            tags: Default::default()
        };

        // find configured tags
        for reference in repo.references()?
        {
            let reference = reference?;
            if reference.is_tag()
            {
                let Some(name) = reference.shorthand() else { continue };
                let Some(id) = reference.target() else { continue };
                let Some(tag_match) = cfg.tags.iter().find_map(|e|e.try_match(name).transpose()) else { continue; };
                let tag_match = tag_match?;
                solver.tags.insert(id, tag_match);
            }
            else
            {
                // find start of the current branch
                for origin in branch.config().raw().origin.iter()
                {
                    if let Some(origin_cfg) = cfg.by_type(origin)
                    {
                        let Some(name) = reference.name() else { continue; };
                        let name = &cfg.reference_name_to_branch_name(name);
                        let Some(target) = reference.target() else { continue; };
                        let Some(origin_match) = origin_cfg.try_match(name, target)? else { continue };
                        let merge_base = repo.merge_base(branch.tip(), origin_match.tip())?;
                        solver.branch_roots.insert(
                            merge_base,
                            branch.base_version().map(Clone::clone)
                                .unwrap_or_else(|| origin_match.base_version().map(Clone::clone)
                                                        .unwrap_or_else(|| SemVersion::default())));
                    }
                    else
                    {
                        bail!("could not find config for origin branch with type: {origin}");
                    }
                }

                for tracked in branch.config().raw().tracked.iter()
                {
                    if let Some(tracked_cfg) = cfg.by_type(tracked)
                    {
                        let Some(name) = reference.name() else { continue; };
                        let name = &cfg.reference_name_to_branch_name(name);
                        let Some(target) = reference.target() else { continue; };
                        let Some(tracked_match) = tracked_cfg.try_match(name, target)? else { continue };
                        let merge_base = repo.merge_base(branch.tip(), tracked_match.tip())?;

                        if let Some(tracked_base) = tracked_match.base_version()
                        {
                            solver.version_bases.insert(merge_base, tracked_base.clone());
                        }
                    }
                    else
                    {
                        bail!("could not find config for tracked branch with type: {tracked}");
                    }
                }
            }
        }

        Ok(solver)
    }

    fn solve_inc_for_commit(&self, id: Oid) -> VersionInc<SemVersion, SemVersionInc>
    {
        if let Some(tag) = self.tags.get(&id)
        {
            return VersionInc::SoftBasis(tag.version().clone()); // basis since a tagged commit has the tagged version, and the following commits it is vNext
        }

        if let Some(base) = self.branch_roots.get(&id)
        {
            return VersionInc::HardBasis(base.clone()); // hard basis since the branch root is already vNext
        }

        if let Some(base) = self.version_bases.get(&id)
        {
            return VersionInc::SoftBasis(base.clone());
        }

        VersionInc::Inc(SemVersionInc::Build(1))
    }

    pub fn solve(&mut self) -> Result<SemVersion>
    {
        let basis = self.current_branch.base_version().map(Clone::clone).unwrap_or_else(|| SemVersion::default());
        let v_next = self.current_branch.config().raw().v_next.clone();
        let tag = self.current_branch.tag();
        let mut version = verner_core::resolve_version(self, basis,  v_next)?;
        if version.build() > 0 { version = version.with_tag(tag); }
        Ok(version)
    }
}

impl<'a> Iterator for BranchSolver<'a>
{
    type Item = Result<VersionInc<SemVersion, SemVersionInc>>;
    
    fn next(&mut self) -> Option<Self::Item>
    {
        if let Some(commit) = self.rev_walk.next()
        {
            return match commit {
                Ok(id) => Some(Ok(self.solve_inc_for_commit(id))),
                Err(err) => Some(Err(anyhow::anyhow!(err))),
            }
        }

        None
    }
    
}

fn resolve_current_branch<'repo, C: ConsoleWriter>(c: &C, cfg: &Config, repo: &'repo Repository) -> anyhow::Result<Reference<'repo>>
{
    let head = repo.head()?;
    if repo.head_detached()?
    {
        c.user_line(LogLevel::Info, "HEAD is detached");

        let Some(head_id) = head.target() else { bail!("Head does not point to a commit") };
        let mut found = Vec::new();

        for b in repo.branches(None)?
        {
            let b = b?;
            let r = b.0.into_reference();
            let Some(id) = r.target() else { continue; };
            let Some(name) = r.name() else { continue; };
            if head_id == id
            {
                c.user_line(LogLevel::Info, format!("found reference {name}"));
                found.push(r);
            }
        }

        if found.len() == 1
        {
            let found = found.into_iter().next().unwrap();
            let Some(name) = found.name() else { bail!("Branch has no name") };
            let name = cfg.reference_name_to_branch_name(name);
            c.user_line(LogLevel::Info, format!("Using branch {}", name));
            return Ok(found);
        }
        else
        {
            c.user_line(LogLevel::Error, format!("Found {} branches pointing to {head_id}. Try specifying the branch explicitly.", found.len()));
            bail!("found multiple references pointing to {head_id}");
        }
    }
    else
    {
        Ok(head)
    }
}

pub fn solve<C: ConsoleWriter + 'static>(c: &C, cwd: &Path, cfg: RawConfig, args: cli::Args) -> anyhow::Result<SemVersion>
{
    let cfg = cfg.parse()?;
    let repo = args.git_dir.map_or_else(||git2::Repository::discover(cwd), |git_dir|git2::Repository::open(git_dir))?;

    let branch =
    if let Some(branch_name) = args.branch_name
    {
        match repo.find_branch(&branch_name, git2::BranchType::Local)
        {
            Ok(b) => b.into_reference(),
            Err(_) => match repo.find_branch(&branch_name, git2::BranchType::Remote)
            {
                Ok(b) => b.into_reference(),
                Err(_) => bail!("Could not find branch {branch_name} in remote or local"),
            },
        }
    }
    else
    {
        resolve_current_branch(c, &cfg, &repo)?
    };

    let Some(name) = branch.name() else { bail!("branch has no name") };
    let Some(target) = branch.target() else { bail!("branch has no target") };

    let name = cfg.reference_name_to_branch_name(name);

    let Some(branch) = cfg.try_match_branch(name, target)? else { bail!("could not resolve HEAD or current branch is not configured") };
    let mut solver = BranchSolver::new(&cfg, &repo, &branch)?;
    Ok(solver.solve()?)
}


pub fn preset_config(preset: &ConfigPreset) -> Result<RawConfig>
{
    Ok(match preset
    {
        ConfigPreset::Releaseflow => RawConfig
        {
            tracked_remotes: vec![ "origin".into() ],
            tags: HashMap::from([
                ("release".into(), RawTagConfig
                {
                    regex: r#"^v(?<major>\d+)\.(?<minor>\d+)\.(?<patch>\d+)$"#.into(),
                    version: "$major.$minor.$patch".into()
                })
            ]),
            branches: HashMap::from([
                ("feature".into(), RawBranchConfig
                {
                    regex: r#"^feat(?:ure)?/(?<name>.+)$"#.into(),
                    tag: Some("feat-$name".into()),
                    tracked: vec![],
                    origin: vec!["main".into(), "release".into()],
                    base_version: None,
                    v_next: Some(SemVersionInc::Patch(1))
                }),
                ("fix".into(), RawBranchConfig
                {
                    regex: r#"^(?:bux)?fix/(?<name>.+)$"#.into(),
                    tag: Some("fix-$name".into()),
                    tracked: vec![],
                    origin: vec!["main".into(), "release".into()],
                    base_version: None,
                    v_next: Some(SemVersionInc::Patch(1))
                }),
                ("main".into(), RawBranchConfig
                {
                    regex: r#"^main$"#.into(),
                    tag: Some("SNAPSHOT".into()),
                    tracked: vec!["release".into()],
                    origin: vec![],
                    base_version: Some("0.1.0".into()),
                    v_next: Some(SemVersionInc::Minor(1))
                }),
                ("release".into(), RawBranchConfig
                {
                    regex: r#"^release/(?<major>\d+)\.(?<minor>\d+)(?:\.x)?$"#.into(),
                    tag: Some("rc".into()),
                    tracked: vec![],
                    origin: vec!["main".into()],
                    base_version: Some("$major.$minor.0".into()),
                    v_next: Some(SemVersionInc::Patch(1))
                })
            ]),
        },
    })
}

