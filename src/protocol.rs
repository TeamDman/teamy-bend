// SPDX-License-Identifier: MPL-2.0
//! Typed constructor data exchanged with an already checked Bend program.
use crate::kernel::Term;
use crate::kernel::TermRef;
use crate::kernel::term;
use eyre::Result;
use eyre::bail;
use facet::Facet;

/// Maximum constructor nesting, counting the root as depth zero.
pub const MAX_DATA_DEPTH: usize = 96;
/// Maximum constructor nodes across all arguments or one response.
pub const MAX_DATA_NODES: usize = 16_384;

/// A constructor tree, without variables, expressions or host callbacks.
#[derive(Clone, Debug, Eq, Facet, PartialEq)]
#[facet(deny_unknown_fields)]
pub struct DataValue {
    pub constructor: String,
    pub fields: Vec<Self>,
}

/// One request in a persistent session. IDs must strictly increase.
#[derive(Debug, Facet)]
#[facet(deny_unknown_fields)]
pub struct Request {
    pub id: u64,
    pub entry: String,
    pub args: Vec<DataValue>,
}

/// Exactly one of `value` and `error` is populated. Malformed requests have no ID.
#[derive(Debug, Facet)]
pub struct Response {
    pub id: Option<u64>,
    pub value: Option<DataValue>,
    pub error: Option<String>,
}

impl DataValue {
    /// Convert checked runtime data into a transport tree.
    ///
    /// # Errors
    /// Rejects non-constructor results and excessive size or depth.
    pub fn from_term(value: &TermRef) -> Result<Self> {
        let mut budget = MAX_DATA_NODES;
        Self::from_term_at(value, 0, &mut budget)
    }

    fn from_term_at(value: &TermRef, depth: usize, budget: &mut usize) -> Result<Self> {
        charge(depth, budget)?;
        let Term::Ctr { name, args } = value.as_ref() else {
            bail!("data protocol requires a constructor result");
        };
        Ok(Self {
            constructor: name.clone(),
            fields: args
                .iter()
                .map(|child| Self::from_term_at(child, depth + 1, budget))
                .collect::<Result<_>>()?,
        })
    }

    fn to_term_at(&self, depth: usize, budget: &mut usize) -> Result<TermRef> {
        charge(depth, budget)?;
        Ok(term(Term::Ctr {
            name: self.constructor.clone(),
            args: self
                .fields
                .iter()
                .map(|child| child.to_term_at(depth + 1, budget))
                .collect::<Result<_>>()?,
        }))
    }
}

/// Convert all request arguments with a shared node budget.
///
/// # Errors
/// Rejects excessive constructor depth or node count. The kernel must still
/// check these terms against the selected function's argument types.
pub fn arguments(values: &[DataValue]) -> Result<Vec<TermRef>> {
    let mut budget = MAX_DATA_NODES;
    values
        .iter()
        .map(|value| value.to_term_at(0, &mut budget))
        .collect()
}

fn charge(depth: usize, budget: &mut usize) -> Result<()> {
    if depth > MAX_DATA_DEPTH {
        bail!("constructor nesting exceeds {MAX_DATA_DEPTH}");
    }
    if *budget == 0 {
        bail!("constructor node budget exceeded");
    }
    *budget -= 1;
    Ok(())
}
