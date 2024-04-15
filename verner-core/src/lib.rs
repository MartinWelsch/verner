use std::fmt::Display;

use anyhow::Result;
pub mod semver;
pub mod output;

#[derive(Debug)]
pub enum VersionInc<Ver, Inc>
{
    /// increment the version
    Inc(Inc),

    /// jump to this version, the next node continues with vNext
    SoftBasis(Ver),

    /// use this as the version, continue on the current version
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

pub fn resolve_version<Ver: VersionOp<Inc> + Display, Inc: Display>(history: &mut HistoryIter<Ver, Inc>, basis: Ver, v_next: Option<Inc>) -> Result<Ver>
{
    let mut version = basis;
    let mut incs: Vec<Inc> = Default::default();

    let mut count = 0;

    while let Some(inc) = history.next()
    {
        let inc = inc?;

        match inc
        {
            VersionInc::Inc(add) =>
            {
                incs.push(add);
            },
            VersionInc::SoftBasis(soft_basis) =>
            {
                version = soft_basis;
                if count > 0
                {
                    v_next.inspect(|i|version.inc(i));
                }
                break;
            },
            VersionInc::HardBasis(hard_basis) =>
            {
                version = hard_basis;
                break;
            },
            VersionInc::Fixed(fix) =>
            {
                return Ok(fix);
            },
            VersionInc::Skip => { },
        }

        count += 1;
    }

    for i in incs.as_slice()
    {
        version.inc(i);
    }
    
    Ok(version)
}
