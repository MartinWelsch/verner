use std::fmt::Display;

use anyhow::Result;
pub mod semver;
pub mod output;

#[derive(Debug)]
pub enum VersionInc<Ver, Inc>
{
    /// increment the version
    Inc(Inc),

    /// override the base version
    SoftBasis(Ver),

    /// use this as the version and do not apply vNext
    HardBasis(Ver),

    /// use this as the version, do not apply vNext and stop further calculations
    Fixed(Ver),

    /// do nothing
    Skip,
}

pub type HistoryIter<'a, Ver, Inc> = dyn Iterator<Item = Result<VersionInc<Ver, Inc>>> + 'a;

pub trait VersionOp<Inc>
{
    fn inc(&mut self, i: &Inc);
}

pub trait NodeVersionInfo<Inc>
{
    fn version_effect(&self) -> Inc;
}

pub fn resolve_version<Ver: VersionOp<Inc> + Display, Inc: Display, F: FnOnce(Ver) -> Ver>(history: &mut HistoryIter<Ver, Inc>, basis: Ver, v_next: F) -> Result<Ver>
{
    let mut version = basis;
    let mut incs: Vec<Inc> = Default::default();

    while let Some(inc) = history.next()
    {
        match inc?
        {
            VersionInc::Inc(add) =>
            {
                incs.push(add);
            },
            VersionInc::SoftBasis(hard_basis) =>
            {
                version = hard_basis;
                version = v_next(version);
                break;
            },
            VersionInc::HardBasis(soft_basis) =>
            {
                version = soft_basis;
                break;
            },
            VersionInc::Fixed(fix) =>
            {
                return Ok(fix);
            },
            VersionInc::Skip => {},
        }
    }

    for i in incs.as_slice()
    {
        version.inc(i);
    }
    
    Ok(version)
}
