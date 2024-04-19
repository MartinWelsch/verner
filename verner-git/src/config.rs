use std::{collections::HashMap, fmt::Display};

use anyhow::bail;
use git2::Oid;
use regex::Regex;
use serde::{Deserialize, Serialize};
use verner_core::semver::{SemVersion, SemVersionInc};

use crate::cli::ConfigPreset;

#[derive(Serialize, Deserialize)]
pub struct RawBranchConfig
{
    /// regex that matches branch short names (excluding origin)
    pub regex: String,

    /// label that is added to the version if solving for this branch
    #[serde(default)]
    #[serde(skip_serializing_if="Option::is_none")]
    pub label: Option<String>,

    /// the base version of this branch
    #[serde(default)]
    #[serde(skip_serializing_if="Option::is_none")]
    pub base_version: Option<String>,

    /// list of tracked branches that influence the version on the current branch
    #[serde(default)]
    #[serde(skip_serializing_if="Vec::is_empty")]
    pub tracked: Vec<String>,

    /// list of all possible source branches
    #[serde(default)]
    #[serde(skip_serializing_if="Vec::is_empty")]
    pub sources: Vec<String>,
    
    /// vNext rule for the current branch - what is incremented after sovling the base version?
    #[serde(default)]
    #[serde(skip_serializing_if="Option::is_none")]
    pub v_next: Option<SemVersionInc>,

    /// max soving depth
    #[serde(default)]
    #[serde(skip_serializing_if="Option::is_none")]
    pub max_depth: Option<u32>
}
impl RawBranchConfig {
    pub fn parse(self, r#type: String) -> anyhow::Result<BranchConfig> {
        Ok(
            BranchConfig
            {
                r#type,
                regex: Regex::new(&self.regex)?,
                raw: self
            }
        )
    }
}


#[derive(Serialize, Deserialize)]
pub struct RawConfig
{
    /// list of all remotes to consider
    #[serde(default)]
    #[serde(skip_serializing_if="Vec::is_empty")]
    pub tracked_remotes: Vec<String>,

    /// list of tags to parse
    #[serde(default)]
    #[serde(skip_serializing_if="HashMap::is_empty")]
    pub tags: HashMap<String, RawTagConfig>,
    
    /// branch configurations
    pub branches: HashMap<String, RawBranchConfig>
}

impl RawConfig
{
    pub fn parse(self) -> anyhow::Result<Config>
    {
        Ok(
            Config
            {
                tracked_remotes: self.tracked_remotes,
                tags: self.tags.into_iter().map(|(k, v)| v.parse(&k)).collect::<anyhow::Result<Vec<TagConfig>>>()?,
                branches: self.branches.into_iter().map(|e| e.1.parse(e.0)).collect::<anyhow::Result<Vec<BranchConfig>>>()?,
            }
        )
    }
}

impl TryInto<Config> for RawConfig
{
    type Error = anyhow::Error;

    fn try_into(self) -> std::prelude::v1::Result<Config, Self::Error>
    {
        self.parse()
    }
}

pub struct BranchConfig
{
    r#type: String,
    raw: RawBranchConfig,
    regex: Regex,
}

impl BranchConfig {
    pub fn raw(&self) -> &RawBranchConfig {
        &self.raw
    }
    
    pub fn regex(&self) -> &Regex {
        &self.regex
    }
    

    pub fn try_match<'a>(&'a self, short_name: &str, tip: Oid) -> anyhow::Result<Option<BranchMatch<'a>>>
    {
        if let Some(captures) = self.regex().captures(short_name)
        {
            return Ok(Some(BranchMatch::create(tip, captures, &self)?));
        }

        Ok(None)
    }
    
    pub fn r#type(&self) -> &str {
        &self.r#type
    }
}
pub struct Config
{
    pub tracked_remotes: Vec<String>,
    pub tags: Vec<TagConfig>,
    pub branches: Vec<BranchConfig>
}

#[derive(Clone)]
pub struct BranchMatch<'a>
{
    config: &'a BranchConfig,
    tag: Option<String>,
    tip: Oid,
    base_version: Option<SemVersion>
}
impl<'a> BranchMatch<'a> {
    fn create(tip: Oid, captures: regex::Captures<'_>, config: &'a BranchConfig) -> anyhow::Result<Self>
    {
        let tag = if let Some(ref label_template) = config.raw.label
        {
            let tip_hash = tip.to_string();
            let mut t = String::new();
            let tpl = label_template.replace("$hash_short", &tip_hash[..8])
                                            .replace("$hash", &tip_hash);
            
            captures.expand(&tpl, &mut t);
            Some(t)
        }
        else { None };
        
        let mut base_version = if let Some(ref template) = config.raw().base_version
        {
            let mut base_version_str = String::new();
            captures.expand(&template, &mut base_version_str);
            let Some(parsed) = SemVersion::parse(&base_version_str) else { bail!("'{base_version_str}' is an invalid version string"); };
            Some(parsed)
        }
        else { None };
        
        if let Some(ref tag) = tag
        {
            base_version = base_version.map(|v|v.with_label(Some(tag.into())));
        }

        Ok(Self
        {
            tip,
            config,
            tag,
            base_version
        })
    }
    
    
    pub fn tip(&self) -> Oid {
        self.tip
    }
    
    pub fn config(&self) -> &BranchConfig {
        self.config
    }
    
    pub fn tag(&self) -> Option<&str> {
        self.tag.as_ref().map(|x| x.as_str())
    }
    
    pub fn base_version(&self) -> Option<&SemVersion> {
        self.base_version.as_ref()
    }
}

impl Config
{
    pub fn find_branches<'a>(&'a self, r#type: &'a str) -> impl Iterator<Item = &'a BranchConfig> + 'a
    {
        self.branches.iter().filter(move |e| e.r#type == r#type)
    }

    pub fn try_match_branch<'a>(&'a self, short_name: &str, id: Oid) -> anyhow::Result<Option<BranchMatch<'a>>>
    {
        for c in self.branches.iter()
        {
            
            if let Some(m) = c.try_match(short_name, id)?
            {
                return Ok(Some(m));
            }
        }

        Ok(None)
    }

    pub fn find_type_branch_config_for<'a>(&'a self, short_name: &str, id: Oid, r#type: &str) -> anyhow::Result<Option<BranchMatch<'a>>>
    {
        for c in self.branches.iter().filter(|p| p.r#type == r#type)
        {
            if let Some(m) = c.try_match(short_name, id)?
            {
                return Ok(Some(m));
            }
        }

        Ok(None)
    }
    
    pub(crate) fn by_type<'a>(&'a self, r#type: &str) -> Option<&'a BranchConfig>
    {
        self.branches.iter().filter(|e| e.r#type == r#type).next()
    }

    /// removes the first occurence of "refs/heads/" or "refs/remotes/<tracked_origin>/"
    /// if nothing could be removed `ref_name` is returned
    pub fn reference_name_to_branch_name<'a>(&self, ref_name: &'a str) -> &'a str
    {
        let mut result = ref_name;
        let mut pattern = String::from("refs/heads/");

        let mut try_remove = |pat: &str|
        {
            let pre_len = result.len();
            result = result.trim_start_matches(&pat);
            result.len() < pre_len
        };

        // remove "refs/heads"
        if try_remove(&pattern)
        {
            return result;
        }

        // remove "refs/remotes/<origin>/"
        for origin in self.tracked_remotes.iter()
        {
            pattern.clear();
            pattern.push_str("refs/remotes/");
            pattern.push_str(&origin);
            pattern.push('/');

            if try_remove(&pattern)
            {
                return result;
            }
        }

        // nothing was removed, return unchanged
        result
    }
}

#[derive(Serialize, Deserialize)]
pub struct RawTagConfig
{
    pub regex: String,
    pub version: String
}

impl RawTagConfig
{
    pub fn parse(self, r#type: &str) -> anyhow::Result<TagConfig>
    {
        let regex = Regex::new(&self.regex)?;
        Ok(TagConfig
        {
            r#type: r#type.into(),
            regex,
            raw: self
        })
    }
}

pub struct TagConfig
{
    r#type: String,
    regex: Regex,
    raw: RawTagConfig
}
impl TagConfig {
    pub fn try_match<'a>(&'a self, tag: &str, id: Oid) -> anyhow::Result<Option<TagMatch<'a>>>
    {
        let Some(captures) = self.regex.captures(tag) else { return Ok(None) };
        let mut version_string = String::new();
        let id_hash = id.to_string();
        let tpl = self.raw.version.replace("$hash_short", &id_hash[..8]).replace("$hash", &id_hash);
        captures.expand(&tpl, &mut version_string);
        let Some(version) = SemVersion::parse(&version_string) else { bail!("{version_string} is an invalid version string") };
        Ok(Some(TagMatch{
            config: &self,
            tag: tag.into(),
            version
        }))
    }
}

pub struct TagMatch<'a>
{
    config: &'a TagConfig,
    tag: String,
    version: SemVersion
}

impl<'a> TagMatch<'a> {
    pub fn config(&self) -> &TagConfig {
        self.config
    }
    
    pub fn version(&self) -> &SemVersion {
        &self.version
    }
}

impl Display for TagMatch<'_>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.write_fmt(format_args!("(tag: {}, type: {}, version: {})", &self.tag, &self.config.r#type, &self.version))?;
        Ok(())
    }
}


pub fn preset_config(preset: &ConfigPreset) -> anyhow::Result<RawConfig>
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
                    label: Some("feat-$name".into()),
                    tracked: vec![],
                    sources: vec!["main".into(), "release".into()],
                    base_version: None,
                    v_next: None,
                    max_depth: None
                }),
                ("fix".into(), RawBranchConfig
                {
                    regex: r#"^(?:bux)?fix/(?<name>.+)$"#.into(),
                    label: Some("fix-$name".into()),
                    tracked: vec![],
                    sources: vec!["main".into(), "release".into()],
                    base_version: None,
                    v_next: None,
                    max_depth: None
                }),
                ("main".into(), RawBranchConfig
                {
                    regex: r#"^main$"#.into(),
                    label: Some("SNAPSHOT".into()),
                    tracked: vec!["release".into()],
                    sources: vec![],
                    base_version: Some("0.1.0".into()),
                    v_next: Some(SemVersionInc::Minor(1)),
                    max_depth: None
                }),
                ("release".into(), RawBranchConfig
                {
                    regex: r#"^release/(?<major>\d+)\.(?<minor>\d+)(?:\.x)?$"#.into(),
                    label: Some("rc".into()),
                    tracked: vec![],
                    sources: vec!["main".into()],
                    base_version: Some("$major.$minor.0".into()),
                    v_next: Some(SemVersionInc::Patch(1)),
                    max_depth: None
                })
            ]),
        },
    })
}

