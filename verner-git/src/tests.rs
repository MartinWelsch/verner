#[cfg(test)]
mod test
{
    use git2::Oid;
    use verner_core::{output::ConsoleWriter, semver::SemVersion};
    use crate::{config::{RawBranchConfig, RawConfig, RawTagConfig}, solve};

    struct NullWriter;
    impl ConsoleWriter for NullWriter
    {
        fn user_line<D: std::fmt::Display>(&self, _level: verner_core::output::LogLevel, _d: D) {}
        fn output<D: std::fmt::Display>(&self, _d: D) {}
    }


    fn get_config() -> RawConfig
    {
        crate::config::preset_config(&crate::cli::ConfigPreset::Releaseflow).unwrap()
    }


    fn solve_repo_version(repo_name: &str) -> anyhow::Result<SemVersion>
    {
        let cfg = get_config();
        let git_dir = std::env::current_dir()?.join(format!("../test_data/releaseflow/{repo_name}"));
        let null_writer = NullWriter;

        let ver = solve(&null_writer, &git_dir.clone(), cfg, crate::cli::Args
        {
            config_preset: None,
            use_local: false,
            use_ref: None,
            override_branch_name: None,
            git_dir: Some(git_dir)
        })?;

        Ok(ver)
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
    repo_test!(1, 0, 0, "fix-patch-something", 1);
    repo_test!(0, 1, 0, "feat-detached-head", 1);
    repo_test!(1, 1, 0, "feat-depth1", 1);
    repo_test!(1, 0, 0, "feat-on-root", 0);

    #[test]
    fn replace_label_hash()
    {
        let branch_config = RawBranchConfig
        {
            regex: "test".into(),
            label: Some("$hash".into()),
            base_version: None,
            tracked: vec![],
            sources: vec![],
            v_next: None,
            max_depth: None,
        };
        let branch_config = branch_config.parse("test".into()).expect("config did not parse");
        let m = branch_config.try_match("test", Oid::from_str("3e95d253526c821c9e5da1edfeb8d90f7d59aae4").unwrap()).expect("error matching branch").expect("branch did not match");
        assert_eq!(m.tag(), Some("3e95d253526c821c9e5da1edfeb8d90f7d59aae4"));
    }
    #[test]
    fn replace_label_hash_short()
    {
        let branch_config = RawBranchConfig
        {
            regex: "test".into(),
            label: Some("$hash_short".into()),
            base_version: None,
            tracked: vec![],
            sources: vec![],
            v_next: None,
            max_depth: None,
        };
        let branch_config = branch_config.parse("test".into()).expect("config did not parse");
        let m = branch_config.try_match("test", Oid::from_str("3e95d253526c821c9e5da1edfeb8d90f7d59aae4").unwrap()).expect("error matching branch").expect("branch did not match");
        assert_eq!(m.tag(), Some("3e95d253"));
    }
    #[test]
    fn replace_tag_version_hash()
    {
        let tag_config = RawTagConfig
        {
            regex: "test".into(),
            version: "0.0.0-$hash".into()
        };
        let tag_config = tag_config.parse("test".into()).expect("config did not parse");
        let m = tag_config.try_match("test", Oid::from_str("3e95d253526c821c9e5da1edfeb8d90f7d59aae4").unwrap()).expect("error matching branch").expect("branch did not match");
        assert_eq!(m.version().label(), Some("3e95d253526c821c9e5da1edfeb8d90f7d59aae4"));
    }
    #[test]
    fn replace_tag_version_hash_short()
    {
        let tag_config = RawTagConfig
        {
            regex: "test".into(),
            version: "0.0.0-$hash_short".into()
        };
        let tag_config = tag_config.parse("test".into()).expect("config did not parse");
        let m = tag_config.try_match("test", Oid::from_str("3e95d253526c821c9e5da1edfeb8d90f7d59aae4").unwrap()).expect("error matching branch").expect("branch did not match");
        assert_eq!(m.version().label(), Some("3e95d253"));
    }
}