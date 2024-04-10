
pub mod semver;


pub enum VersionInc<Ver, Inc>
{
    Add(Inc),
    Basis(Ver),
    Skip
}

pub type HistoryIter<'a, Ver, Inc> = dyn Iterator<Item = VersionInc<Ver, Inc>> + 'a;

pub trait VersionOp<Inc>
{
    fn inc(&mut self, i: &Inc);
}

pub fn resolve_version<Ver: VersionOp<Inc> + Default, Inc>(history: &mut HistoryIter<Ver, Inc>) -> Ver
{
    let mut incs: Vec<Inc> = Default::default();
    let mut basis = Ver::default();

    while let Some(inc) = history.next()
    {
        match inc
        {
            VersionInc::Add(add) =>
            {
                incs.push(add);
            },
            VersionInc::Basis(basis_override) =>
            {
                basis = basis_override;
                break;
            },
            VersionInc::Skip => {},
        }
    }

    for i in incs.as_slice()
    {
        basis.inc(i);
    }
    
    basis
}
