#[cfg(test)] mod tests;

mod config;
pub mod cli;

pub use config::{RawConfig, preset_config};

use std::{collections::HashMap, path::Path};

use anyhow::{bail, Result};
use config::{BranchMatch, Config, TagMatch};
use git2::{Oid, Reference, Repository, Revwalk};
use verner_core::{output::{ConsoleWriter, LogLevel}, semver::{SemVersion, SemVersionInc}, VersionHint, VersionInc};


struct BranchSolveContext
{
    depth: u32,
    max_depth: u32
}

impl BranchSolveContext
{
    pub fn try_descend(&self) -> Option<Self>
    {
        if self.depth == self.max_depth
        {
            None
        }
        else
        {
            Some(Self
            {
                depth: self.depth + 1,
                max_depth: self.max_depth
            })
        }
    }
}

struct BranchSolver<'a>
{
    current_branch: BranchMatch<'a>,
    version_bases: HashMap<Oid, SemVersion>,
    branch_roots: HashMap<Oid, Option<BranchSolver<'a>>>,
    tags: HashMap<Oid, TagMatch<'a>>,
    rev_walk: Revwalk<'a>
}

impl<'a> BranchSolver<'a>
{
    pub fn new(ctx: BranchSolveContext, cfg: &'a Config, repo: &'a Repository, branch: BranchMatch<'a>) -> Result<Self>
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
            current_branch: branch.clone(),
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
                let Some(tag_match) = cfg.tags.iter().find_map(|e|e.try_match(name, id).transpose()) else { continue; };
                let tag_match = tag_match?;
                solver.tags.insert(id, tag_match);
            }
            else
            {
                // find start of the current branch
                for origin in branch.config().raw().sources.iter()
                {
                    if let Some(origin_cfg) = cfg.by_type(origin)
                    {
                        let Some(name) = reference.name() else { continue; };
                        let name = &cfg.reference_name_to_branch_name(name);
                        let Some(tip) = reference.target() else { continue; };
                        let merge_base = repo.merge_base(branch.tip(), tip)?;
                        let Some(source_match) = origin_cfg.try_match(name, merge_base)? else { continue };

                        let source_solver = if branch.base_version().is_none()
                        {ctx.try_descend().map(|ctx| BranchSolver::new(ctx, cfg, repo, source_match)).transpose()?}
                        else { None };
                        solver.branch_roots.insert(merge_base, source_solver);
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

    fn solve_inc_for_commit(&mut self, id: Oid) -> anyhow::Result<VersionInc<SemVersion, SemVersionInc>>
    {
        if let Some(tag) = self.tags.get(&id)
        {
            return Ok(VersionInc::Fixed(tag.version().clone())); // fixed since a tagged commit has the tagged version, and the following commits it is vNext
        }

        if let Some(source_solver) = self.branch_roots.get_mut(&id)
        {
            if let Some(source_solver) = source_solver
            {
                let source_version = source_solver.solve()?;
                return Ok(VersionInc::SoftBasis(source_version.erase_build())); // soft basis since the solved value is vNext of the source branch
            }
            else
            {
                return Ok(VersionInc::HardBasis(self.current_branch.base_version().map(|v|v.clone()).unwrap_or_default())); // hard basis since the branch root is already vNext
            }
        }

        if let Some(base) = self.version_bases.get(&id)
        {

            return Ok(VersionInc::SoftBasis(base.clone()));
        }

        Ok(VersionInc::Inc(SemVersionInc::Build(1)))
    }

    pub fn solve(&mut self) -> Result<SemVersion>
    {
        self.solve_raw(self.current_branch.config().raw().v_next.clone())
    }

    fn solve_raw(&mut self, v_next: Option<SemVersionInc>) -> Result<SemVersion>
    {
        let basis = self.current_branch.base_version().map(Clone::clone).unwrap_or_else(|| SemVersion::default());
        let tag = self.current_branch.tag().map(|tag|tag.to_string());
        let (mut version, hint) = verner_core::resolve_version(self, basis, v_next)?;
        if hint != VersionHint::Fixed { version = version.with_label(tag.into()); }
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
                Ok(id) => Some(self.solve_inc_for_commit(id)),
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

/// starting point for resolving a version from a git repository
pub fn solve<C: ConsoleWriter + 'static>(c: &C, cwd: &Path, cfg: RawConfig, args: cli::Args) -> anyhow::Result<SemVersion>
{
    let cfg = cfg.parse()?;
    let repo = args.git_dir.map_or_else(||git2::Repository::discover(cwd), |git_dir|git2::Repository::open(git_dir))?;

    let branch_map_from_ref = |r: Reference|
    {
        let Some(name) = r.name() else { bail!("branch has no name") };
        let Some(target) = r.target() else { bail!("branch has no target") };

        let name = cfg.reference_name_to_branch_name(name);

        let Some(branch) = cfg.try_match_branch(name, target)? else { bail!("could not resolve HEAD or current branch is not configured") };
        Ok(branch)
    };

    let branch =
    if let Some(ref_name) = args.use_ref
    {
        branch_map_from_ref(repo.find_reference(&ref_name)?)?
    }
    else if let Some(override_branch_name) = args.override_branch_name
    {
        let Some(head_id) = repo.head()?.target() else { bail!("HEAD does not point to a commit") };
        let Some(m) = cfg.try_match_branch(&override_branch_name, head_id)? else { bail!("{override_branch_name} does not match any configured branch type") };
        m
    }
    else
    {
        branch_map_from_ref(resolve_current_branch(c, &cfg, &repo)?)?
    };

    
    let mut solver = BranchSolver::new(
        BranchSolveContext
            {
                depth: 0,
                max_depth: branch.config().raw().max_depth.unwrap_or(u32::MAX)
            },
            &cfg,
            &repo,
            branch
        )?;
    Ok(solver.solve()?)
}

