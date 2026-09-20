// SPDX-License-Identifier: Apache-2.0
// Foreign assembly derived from Bend 2.0.5, Copyright 2026 HigherOrderCO.
// Rust translation and changes: TeamDman. See NOTICE and licenses/Apache-2.0.txt.
//! Canonical C imports, scoped constructor aliases and explicit initialization.

use super::CompileError;
use super::executable_c_layout::ConstructorTable;
use super::executable_c_layout::cid_macro;
use super::executable_foreign::canonical_source;
use super::executable_foreign::reference_order;
use crate::kernel::elaborate::DefinitionBody;
use crate::kernel::elaborate::ExecutableProgram;
use crate::syntax::executable::BuiltinForeign;
use crate::syntax::executable::ForeignTarget;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt::Write;

pub(super) struct ForeignAssembly {
    pub(super) source: String,
    /// Call these once in order, inside the runtime's guarded initializer.
    pub(super) initializers: Vec<String>,
    /// Every request must have registered an Effect before execution begins.
    pub(super) requests: BTreeMap<String, u16>,
}

pub(super) fn assemble(
    program: &ExecutableProgram,
    table: &ConstructorTable,
) -> Result<ForeignAssembly, CompileError> {
    let mut assembly = ForeignAssembly {
        source: String::new(),
        initializers: Vec::new(),
        requests: BTreeMap::new(),
    };
    let mut seen = BTreeSet::new();
    let mut initializers = BTreeSet::new();
    let request_macros = reference_order(program)
        .into_iter()
        .filter(|name| matches!(program.definitions[*name].body, DefinitionBody::Foreign(_)))
        .map(|name| table.get(name).map(|entry| entry.macro_name.clone()))
        .collect::<Result<BTreeSet<_>, _>>()?;
    for name in reference_order(program) {
        let DefinitionBody::Foreign(foreign) = &program.definitions[name].body else {
            continue;
        };
        assembly
            .requests
            .insert(name.to_owned(), table.get(name)?.cid);
        if let Some(builtin) = foreign.builtin {
            require_builtin(builtin, name)?;
            continue;
        }
        let Some(source) = canonical_source(name, foreign, ForeignTarget::C, &mut seen)? else {
            continue;
        };
        let (source, starts) = portable_initializers(&source)
            .map_err(|error| CompileError::new(format!("C import for {name}: {error}")))?;
        for start in starts {
            if !initializers.insert(start.clone()) {
                return Err(CompileError::new(format!(
                    "duplicate C initializer {start}"
                )));
            }
            assembly.initializers.push(start);
        }
        let namespace = namespace(name, &foreign.local_symbol)?;
        let mut aliases = BTreeMap::new();
        if !namespace.is_empty() {
            for (constructor, local) in &program.constructor_tags {
                if constructor.starts_with(namespace) {
                    let Ok(entry) = table.get(constructor) else {
                        continue;
                    };
                    let alias = cid_macro(local);
                    if request_macros.contains(&alias)
                        || ["CID_ARITY_T", "FID_ARITY_T", "FID_FLAG_T", "FID_RESW_T"]
                            .contains(&alias.as_str())
                    {
                        return Err(CompileError::new(format!(
                            "C constructor alias {alias} collides with a request or runtime table",
                        )));
                    }
                    if let Some(previous) = aliases.insert(alias.clone(), entry.cid)
                        && previous != entry.cid
                    {
                        return Err(CompileError::new(format!(
                            "C constructor aliases collide at {alias}"
                        )));
                    }
                }
            }
        }
        for (alias, cid) in &aliases {
            // Another local alias can shadow this constructor's global macro.
            // A validated numeric ID cannot be redirected by that second alias.
            writeln!(
                assembly.source,
                "#pragma push_macro(\"{alias}\")\n#undef {alias}\n#define {alias} {cid}"
            )
            .unwrap();
        }
        for allocator in ["malloc", "calloc", "realloc", "free"] {
            writeln!(assembly.source, "#pragma push_macro(\"{allocator}\")\n#undef {allocator}\n#define {allocator} tb_host_{allocator}").unwrap();
        }
        assembly.source.push_str(&source);
        assembly.source.push('\n');
        for allocator in ["free", "realloc", "calloc", "malloc"] {
            writeln!(assembly.source, "#pragma pop_macro(\"{allocator}\")").unwrap();
        }
        for alias in aliases.keys().rev() {
            writeln!(assembly.source, "#pragma pop_macro(\"{alias}\")").unwrap();
        }
    }
    Ok(assembly)
}

fn require_builtin(builtin: BuiltinForeign, name: &str) -> Result<(), CompileError> {
    match builtin {
        BuiltinForeign::Print
        | BuiltinForeign::Write
        | BuiltinForeign::PrintErr
        | BuiltinForeign::Spawn
        | BuiltinForeign::Sleep
        | BuiltinForeign::Now
        | BuiltinForeign::ChanNew
        | BuiltinForeign::ChanSend
        | BuiltinForeign::ChanRecv
        | BuiltinForeign::ChanClose
        | BuiltinForeign::GetEnv
        | BuiltinForeign::FileOpen
        | BuiltinForeign::FileRead
        | BuiltinForeign::FileReadBytes
        | BuiltinForeign::FileWrite
        | BuiltinForeign::FileClose => Ok(()),
        _ => Err(CompileError::new(format!(
            "executable C effect {name} is not implemented"
        ))),
    }
}

fn namespace<'a>(name: &'a str, local_symbol: &str) -> Result<&'a str, CompileError> {
    let starts = std::iter::once(0).chain(name.match_indices('.').map(|(index, _)| index + 1));
    for start in starts {
        if name[start..].to_lowercase().replace(['.', '/'], "_") == local_symbol {
            return Ok(&name[..start]);
        }
    }
    Err(CompileError::new(format!(
        "cannot recover C import namespace for {name}"
    )))
}

#[derive(Debug)]
struct Token<'a> {
    text: &'a str,
    start: usize,
    end: usize,
}

/// Only constructor annotations on a zero-argument void function are adapted.
/// Other attributes are retained. Comments and literals cannot register code.
fn portable_initializers(source: &str) -> Result<(String, Vec<String>), CompileError> {
    let tokens = c_tokens(source)?;
    let mut removals = Vec::new();
    let mut initializers = Vec::new();
    let mut index = 0;
    let mut conditional_depth = 0_usize;
    while index < tokens.len() {
        if tokens[index].text == "#" {
            match tokens.get(index + 1).map(|token| token.text) {
                Some("if" | "ifdef" | "ifndef") => conditional_depth += 1,
                Some("endif") => conditional_depth = conditional_depth.saturating_sub(1),
                _ => {}
            }
        }
        if tokens[index].text != "__attribute__" {
            index += 1;
            continue;
        }
        let Some(end) = attribute_end(&tokens, index) else {
            return Err(CompileError::new("malformed C attribute"));
        };
        let attribute = tokens[index..=end]
            .iter()
            .map(|token| token.text)
            .collect::<Vec<_>>();
        if !attribute.contains(&"constructor") {
            index = end + 1;
            continue;
        }
        if attribute != ["__attribute__", "(", "(", "constructor", ")", ")"] {
            return Err(CompileError::new(
                "unsupported C constructor attribute; expected constructor without priority or combined attributes",
            ));
        }
        if conditional_depth != 0 || in_directive(source, tokens[index].start) {
            return Err(CompileError::new(
                "conditional or macro-defined C constructor initializers are not supported",
            ));
        }
        let name = initializer_name(&tokens, index, end)?;
        removals.push((tokens[index].start, tokens[end].end));
        initializers.push(name.to_owned());
        index = end + 1;
    }
    let mut output = String::new();
    let mut copied = 0;
    for (start, end) in removals {
        output.push_str(&source[copied..start]);
        // Preserve physical lines for imported compiler diagnostics.
        for character in source[start..end].chars() {
            output.push(if character == '\n' { '\n' } else { ' ' });
        }
        copied = end;
    }
    output.push_str(&source[copied..]);
    Ok((output, initializers))
}

fn in_directive(source: &str, at: usize) -> bool {
    let prefix = &source[..at];
    let mut start = prefix.rfind('\n').map_or(0, |index| index + 1);
    // A backslash continues a preprocessor directive across physical lines.
    while start > 0 && source[..start - 1].trim_end_matches('\r').ends_with('\\') {
        start = source[..start - 1].rfind('\n').map_or(0, |index| index + 1);
    }
    source[start..at].trim_start().starts_with('#')
}

fn attribute_end(tokens: &[Token<'_>], at: usize) -> Option<usize> {
    if tokens.get(at + 1)?.text != "(" {
        return None;
    }
    let mut depth = 0_usize;
    for (offset, token) in tokens[at + 1..].iter().enumerate() {
        match token.text {
            "(" => depth += 1,
            ")" => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(at + 1 + offset);
                }
            }
            _ => {}
        }
    }
    None
}

fn initializer_name<'a>(
    tokens: &'a [Token<'a>],
    start: usize,
    end: usize,
) -> Result<&'a str, CompileError> {
    let left = tokens[..start]
        .iter()
        .rposition(|token| matches!(token.text, ";" | "{" | "}" | "#"))
        .map_or(0, |index| index + 1);
    let right = tokens[end + 1..]
        .iter()
        .position(|token| matches!(token.text, ";" | "{" | "}"))
        .map(|index| end + 1 + index)
        .ok_or_else(|| CompileError::new("C constructor initializer has no function body"))?;
    if tokens[right].text != "{" {
        return Err(CompileError::new(
            "C constructor initializer must annotate a function definition",
        ));
    }
    let signature = tokens[left..right]
        .iter()
        .enumerate()
        .filter(|(offset, _)| {
            let index = left + offset;
            index < start || index > end
        })
        .map(|(_, token)| token.text)
        .collect::<Vec<_>>();
    let Some(open) = signature.iter().rposition(|token| *token == "(") else {
        return Err(CompileError::new(
            "unsupported C constructor initializer signature",
        ));
    };
    let valid_args = signature[open..] == ["(", "void", ")"] || signature[open..] == ["(", ")"];
    if open < 2
        || signature[open - 2] != "void"
        || !valid_args
        || !c_identifier(signature[open - 1])
    {
        return Err(CompileError::new(
            "C constructor initializer must be a zero-argument void function",
        ));
    }
    Ok(signature[open - 1])
}

fn c_identifier(text: &str) -> bool {
    text.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
        && text.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn c_tokens(source: &str) -> Result<Vec<Token<'_>>, CompileError> {
    let bytes = source.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index].is_ascii_whitespace() {
            index += 1;
            continue;
        }
        if bytes[index..].starts_with(b"//") {
            index += bytes[index..]
                .iter()
                .position(|byte| *byte == b'\n')
                .unwrap_or(bytes.len() - index);
            continue;
        }
        if bytes[index..].starts_with(b"/*") {
            index += 2;
            index += source[index..]
                .find("*/")
                .ok_or_else(|| CompileError::new("unterminated C comment"))?
                + 2;
            continue;
        }
        let start = index;
        if matches!(bytes[index], b'\'' | b'"') {
            let quote = bytes[index];
            index += 1;
            loop {
                let Some(byte) = bytes.get(index) else {
                    return Err(CompileError::new("unterminated C literal"));
                };
                if *byte == b'\\' {
                    index += 2;
                } else {
                    index += 1;
                    if *byte == quote {
                        break;
                    }
                }
            }
        } else if bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_' {
            index += 1;
            while index < bytes.len()
                && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
            {
                index += 1;
            }
        } else {
            index += source[index..]
                .chars()
                .next()
                .expect("nonempty suffix")
                .len_utf8();
        }
        tokens.push(Token {
            text: &source[start..index],
            start,
            end: index,
        });
    }
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::assemble;
    use super::portable_initializers;
    use crate::compiler::executable_c_layout::ConstructorTable;
    use crate::kernel::check_executable;
    use crate::syntax::load_executable;
    use std::path::PathBuf;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    struct Fixture(PathBuf);

    impl Fixture {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "teamy-bend-c-foreign-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }

        fn write(&self, name: &str, source: &str) {
            std::fs::write(self.0.join(name), source).unwrap();
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn portable_registration_preserves_body_order_and_nonconstructor_attributes() {
        let source = "// __attribute__((constructor)) fake(void){}\nstatic void __attribute__((constructor)) first(void){io_eff(CID_FIRST, run, 0);}\nstatic void second(void) __attribute__((constructor)){io_eff(CID_SECOND, run, IO_READ);}\nstatic int __attribute__((aligned(16))) kept;\n";
        let (adapted, names) = portable_initializers(source).unwrap();
        assert_eq!(names, ["first", "second"]);
        assert!(adapted.contains("io_eff(CID_FIRST, run, 0)"));
        assert!(adapted.contains("__attribute__((aligned(16)))"));
        assert!(adapted.starts_with("// __attribute__((constructor)) fake(void){}"));
    }

    #[test]
    fn ambiguous_or_parameterized_initializers_are_refused() {
        for source in [
            "static void __attribute__((constructor(101))) init(void){}",
            "static int __attribute__((constructor)) init(void){return 0;}",
            "static void __attribute__((constructor)) init(int x){}",
            "static void __attribute__((constructor)) init(void);",
            "static void __attribute__((constructor,used)) init(void){}",
            "#if ON\nstatic void __attribute__((constructor)) init(void){}\n#endif",
            "#define START __attribute__((constructor))\nstatic void START init(void){}",
        ] {
            assert!(portable_initializers(source).is_err(), "{source}");
        }
    }

    #[test]
    fn canonical_c_imports_retain_scoped_aliases_and_single_initialization() {
        let fixture = Fixture::new();
        fixture.write("effect.c", "static int shared_state;\nstatic void __attribute__((constructor)) register_both(void){io_eff(CID_TICK_BUMP, bump, 0);io_eff(CID_TICK_PEEK, peek, 0);}\n");
        fixture.write("ignored.c", "must_not_appear");
        fixture.write("module.bend", "import Base\ntype Token is Data:\n  Local.Tag{}\ndef Tick.bump() -> IO(Token):\n  import \"./effect.c\"\n  import \"./ignored.c\"\ndef Tick.peek() -> IO(Token):\n  import \"././effect.c\"\n");
        fixture.write("main.bend", "import Base\nimport ./module.bend as M\ndef main() -> IO(M.Token):\n  do IO<M.Token>:\n    x : M.Token <- M.Tick.bump()\n    M.Tick.peek()\n");
        let source = load_executable(fixture.0.join("main.bend")).unwrap();
        let program = check_executable(&source)
            .unwrap()
            .lower_for_compilation()
            .unwrap();
        let table = ConstructorTable::build(&program).unwrap();
        let assembly = assemble(&program, &table).unwrap();
        assert_eq!(
            assembly.source.matches("static int shared_state").count(),
            1
        );
        assert!(!assembly.source.contains("must_not_appear"));
        assert!(assembly.source.contains(&format!(
            "#define CID_LOCAL_TAG {}\n",
            table.get("module.Local.Tag").unwrap().cid,
        )));
        assert!(
            assembly
                .source
                .contains("#pragma pop_macro(\"CID_LOCAL_TAG\")")
        );
        assert!(assembly.source.contains("#define malloc tb_host_malloc"));
        assert_eq!(assembly.initializers, ["register_both"]);
        assert_eq!(table.get("module.Tick.bump").unwrap().arity, 1);
        assert_eq!(assembly.requests.len(), 2);
    }

    fn aliases_for(constructors: &str) -> Result<(String, ConstructorTable), super::CompileError> {
        let fixture = Fixture::new();
        fixture.write("effect.c", "static void __attribute__((constructor)) register_effect(void){io_eff(CID_TICK_BUMP, bump, 0);}\n");
        fixture.write("module.bend", &format!("import Base\ntype Token is Data:\n{constructors}\ndef Tick.bump() -> IO(Token):\n  import \"./effect.c\"\n"));
        fixture.write(
            "main.bend",
            "import Base\nimport ./module.bend as M\ndef main() -> IO(M.Token):\n  M.Tick.bump()\n",
        );
        let source = load_executable(fixture.0.join("main.bend")).unwrap();
        let program = check_executable(&source)
            .unwrap()
            .lower_for_compilation()
            .unwrap();
        let table = ConstructorTable::build(&program)?;
        let assembly = assemble(&program, &table)?;
        Ok((assembly.source, table))
    }

    #[test]
    fn aliases_cannot_redirect_each_other_through_shadowed_global_macros() {
        let (source, table) = aliases_for("  Tag{}\n  module.Tag{}").unwrap();
        let plain = table.get("module.Tag").unwrap().cid;
        let nested = table.get("module.module.Tag").unwrap().cid;
        assert_ne!(plain, nested);
        assert!(source.contains(&format!("#define CID_TAG {plain}\n")));
        assert!(source.contains(&format!("#define CID_MODULE_TAG {nested}\n")));
    }

    #[test]
    fn aliases_refuse_request_and_reserved_table_meaning_collisions() {
        for constructors in ["  Tick.Bump{}", "  Arity.T{}"] {
            let error = aliases_for(constructors).err().unwrap().to_string();
            assert!(
                error.contains("collides with a request or runtime table"),
                "{error}"
            );
        }
    }
}
