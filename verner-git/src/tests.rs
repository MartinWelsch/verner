#[cfg(test)]
mod test
{
    use std::collections::HashMap;

    use anyhow::bail;
    use git2::Repository;
    use verner_core::semver::{SemVersion, SemVersionInc};
    use crate::{config::{Config, RawBranchConfig, RawConfig, RawTagConfig}, BranchSolver};

    fn get_config() -> RawConfig
    {
        RawConfig
        {
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
                    regex: r#"^feature/(?<name>.+)$"#.into(),
                    tag: Some("feat-{{ name }}".into()),
                    tracked: vec![],
                    origin: vec!["main".into(), "rc".into()],
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
        }
    }


    fn solve_repo_version(repo_name: &str) -> anyhow::Result<SemVersion>
    {
        let cfg = get_config();
        let cfg: Config = cfg.parse()?;
        let repo = Repository::open(std::env::current_dir()?.join(format!("../test_data/releaseflow/{repo_name}")))?;
        let head = repo.head()?;
        let current_branch = cfg.find_branch_config_for(&head)?;

        if current_branch.is_none()
        {
            bail!("current branch could not be matched to any configured branch");
        }

        let current_branch = current_branch.unwrap();
        let mut solver = BranchSolver::new(&cfg, &repo, &current_branch)?;
        Ok(solver.solve()?)
    }

    macro_rules! repo_test {
        ($major:expr, $minor:expr, $patch:expr, $tag:expr, $build:expr) =>
        {
            ::paste::paste! {
                #[test]
                #[allow(non_snake_case)]
                fn [<check_generated_version_$major _$minor _$patch _$tag _$build>]()
                {
                    let version = SemVersion::parse(&format!("{}.{}.{}-{}.{}", $major, $minor, $patch, $tag, $build)).unwrap();
                    let solved = solve_repo_version(&version.to_string()).unwrap();
                    assert!(version == solved, "expected: {} / actual: {}", version, solved);
                }
            }
        };
    }
    

    repo_test!(1, 0, 0, "rc", 0);
    repo_test!(1, 0, 0, "rc", 1);
    repo_test!(0, 1, 0, "SNAPSHOT", 2);
    repo_test!(1, 1, 0, "SNAPSHOT", 1);
    repo_test!(1, 0, 0, "", 0);
    repo_test!(1, 0, 1, "rc", 1);
}