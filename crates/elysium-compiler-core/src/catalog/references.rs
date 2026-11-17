//! Database rows remain native reference records in the catalog, with no recipe projection.
pub fn project(rows:&[crate::domain::references::GeneticReferenceRow])->Vec<crate::domain::references::GeneticReferenceRow>{rows.to_vec()}
