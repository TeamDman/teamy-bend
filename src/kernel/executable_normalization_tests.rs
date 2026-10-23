// SPDX-License-Identifier: MPL-2.0
#[path = "../../tests/support/normalization_shape.rs"]
mod normalization_shape;

use crate::kernel as normalization_kernel;
use crate::kernel::ExecutableEntry;
use crate::kernel::check_book;
use crate::kernel::check_executable;
use crate::syntax::load;
use crate::syntax::load_executable;
use crate::syntax::parse_term;
use normalization_shape::shape_in_book;
use std::path::Path;

#[test]
fn float_suffix_functions_preserve_core_structure_without_granting_proof_contracts() {
    for suffix in ["f", "g", "h"] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cpu-normalization")
            .join(format!("chain_suffix_f32_{suffix}.bend"));
        let source = load_executable(&path).unwrap();
        let checked = check_executable(&source).unwrap();
        assert_eq!(checked.entry_kind().unwrap(), ExecutableEntry::Pure);
        let golden = std::fs::read_to_string(&path)
            .unwrap()
            .lines()
            .filter_map(|line| line.strip_prefix("#|"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut engine = checked.engine.clone();
        engine.reset();
        let actual = engine.normalize_definition("main").unwrap();
        let actual_shape = shape_in_book(&actual, &source.book).unwrap();
        assert_eq!(
            actual_shape,
            shape_in_book(&parse_term(&golden).unwrap(), &source.book).unwrap(),
            "{suffix}"
        );
        assert_eq!(actual.to_string(), golden, "{suffix}");
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        assert_eq!(
            checked
                .run_main(&mut stdout, &mut stderr, &|| false)
                .unwrap(),
            0
        );
        assert_eq!(stdout, format!("{golden}\n").as_bytes());
        assert!(stderr.is_empty());
        let error = check_book(&load(&path).unwrap())
            .expect_err("executable numeric contracts cannot grant proof-mode primitives");
        assert!(
            error.to_string().contains("undefined name F32.add"),
            "{error}"
        );
    }
}
