use std::cmp::Ordering;

use gix::{Reference, Repository};
use anyhow::Result;
use regex::Regex;
use verner_core::semver::SemVersion;


pub struct Config
{
    main_branch: String,
    release_branch: String
}

fn config_semver(
    repo: &Repository,
    cfg: &Config) -> Result<super::Config>
{
    let release_branch = Regex::new(&cfg.release_branch)?;
    let mut release_branches: Vec<(SemVersion, gix::hash::ObjectId)> = repo.references().unwrap()
        .all().unwrap()
        .filter_map(|reference|
        {
            if let Ok(reference) = reference
            {
                let name = reference.name().shorten().to_string();
                if let Some(captures) = release_branch.captures(&name)
                {
                    println!("{name}");
                    return Some((SemVersion::parse(&format!("{}.{}.0", &captures["major"], &captures["minor"])).unwrap(), reference.id().into()));
                }
            }

            None
        })
        .collect();

    release_branches.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap_or(Ordering::Equal));

    Ok(super::Config
    {
        branch_tags: vec![
            (Regex::new(&cfg.main_branch)?, "SNAPSHOT".into()),
            (Regex::new(&cfg.release_branch)?, "rc".into()),
        ],
        base_version_sources: release_branches.into_iter().map(|(_, id)| id).collect()
    })
}

#[cfg(test)]
mod test
{
    use super::*;
    
    use gix::ObjectId;
    use crate::LogIterator;

    
    #[test]
    fn check_releaseflow_repos()
    {
        let repo = gix::open(std::env::current_dir().unwrap().join("../test_data/semver/1.0.1-SNAPSHOT.1")).unwrap();

        let mut cfg = config_semver(&repo, &Config
        {
            main_branch: "main".into(),
            release_branch: r"^release/(?<major>\d+)\.(?<minor>\d+)(?:\.x)?$".into()
        }).unwrap();
        
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
}