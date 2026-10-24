// SPDX-License-Identifier: MPL-2.0
#[path = "support/normalization_shape.rs"]
mod normalization_shape;

use normalization_shape::shape;
use normalization_shape::shape_in_book;
use std::path::Path;
use teamy_bend::compiler::compile_executable_c;
use teamy_bend::compiler::compile_executable_javascript;
use teamy_bend::kernel as normalization_kernel;
use teamy_bend::kernel::ExecutableEntry;
use teamy_bend::kernel::check_book;
use teamy_bend::kernel::check_executable;
use teamy_bend::syntax::load;
use teamy_bend::syntax::load_executable;
use teamy_bend::syntax::parse;
use teamy_bend::syntax::parse_term;

const FIXTURES: [&str; 9] = [
    "page/compile_no_main.bend",
    "page/motive_binder_name.bend",
    "proof/ackermann_lower_bound.bend",
    "proof/constructor_discriminate.bend",
    "proof/function_congruence.bend",
    "proof/gadt_refutation.bend",
    "proof/map_fusion.bend",
    "state/clean_typecheck_000.bend",
    "state/clean_typecheck_001.bend",
];

#[test]
fn no_main_modules_check_and_normalize_without_becoming_standalone_programs() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-no-main");
    let mut definitions = 0;
    for fixture in FIXTURES {
        let path = root.join(fixture);
        let book = load(&path).unwrap();
        let checked = check_book(&book).unwrap();
        checked.normalize_definition("main").expect_err("no entry");
        for line in include_str!("fixtures/cpu-no-main/normal-forms.tsv").lines() {
            let mut fields = line.splitn(3, '\t');
            let source = fields.next().unwrap();
            let entry = fields.next().unwrap();
            let expected = fields.next().unwrap();
            if source != fixture {
                continue;
            }
            let value = checked.normalize_definition(entry).unwrap();
            assert_eq!(value.to_string(), expected, "{fixture}:{entry}");
            assert_eq!(
                shape_in_book(&value, &book).unwrap(),
                shape_in_book(&parse_term(expected).unwrap(), &book).unwrap(),
                "{fixture}:{entry}"
            );
            definitions += 1;
        }
        let executable = check_executable(&load_executable(&path).unwrap()).unwrap();
        assert_eq!(executable.entry_kind().unwrap(), ExecutableEntry::Missing);
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        assert_eq!(
            executable
                .run_main(&mut stdout, &mut stderr, &|| false)
                .unwrap(),
            0
        );
        assert_eq!(stdout, b"All terms check.\n");
        assert!(stderr.is_empty());
        for result in [
            compile_executable_c(&executable),
            compile_executable_javascript(&executable),
        ] {
            assert_eq!(result.unwrap_err().to_string(), "no main to run");
        }
    }
    assert_eq!(definitions, 30);
}

#[test]
fn multiplicities_remain_distinct_in_structural_comparisons() {
    for (left, right) in [("&0", "&1"), ("&1", "&2"), ("&0", "&2")] {
        assert_ne!(
            shape(&parse_term(left).unwrap()).unwrap(),
            shape(&parse_term(right).unwrap()).unwrap()
        );
        assert_ne!(
            shape(&parse_term(&format!("List<{left}, U32>")).unwrap()).unwrap(),
            shape(&parse_term(&format!("List<{right}, U32>")).unwrap()).unwrap()
        );
    }
    shape(&parse_term("Type").unwrap()).expect_err("unqualified types remain unsupported");
}

#[test]
fn false_gadt_and_fusion_proofs_still_reject() {
    for (original, before, after) in [
        (
            include_str!("fixtures/cpu-no-main/proof/gadt_refutation.bend"),
            "Even<S{S{Z{}}}>\n",
            "Even<S{Z{}}>\n",
        ),
        (
            include_str!("fixtures/cpu-no-main/proof/map_fusion.bend"),
            "U32.add(U32.mul(x, 2), 1)",
            "U32.add(U32.mul(x, 2), 2)",
        ),
    ] {
        assert_eq!(original.matches(before).count(), 1);
        let source = original
            .replace(before, after)
            .replace("import Base\n", include_str!("../src/syntax/base.bend"));
        let book = parse(&source).expect("false proof still parses");
        check_book(&book).expect_err("false proof must reject");
    }
}
