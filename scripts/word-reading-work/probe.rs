// Temporary research instrumentation; removed before production validation.
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io::Write;

thread_local! {
    static COUNTS: RefCell<Option<BTreeMap<String, usize>>> = const { RefCell::new(None) };
}

pub(super) fn charged(location: &'static std::panic::Location<'static>) {
    COUNTS.with_borrow_mut(|counts| {
        if let Some(counts) = counts {
            *counts.entry(format!("charge:{}:{}", location.file(), location.line())).or_default() += 1;
        }
    });
}

pub(super) fn operation(kind: &str, operation: super::numeric::PureOptimization) {
    COUNTS.with_borrow_mut(|counts| {
        if let Some(counts) = counts {
            *counts.entry(format!("{kind}:{operation:?}")).or_default() += 1;
        }
    });
}

fn snapshot() -> BTreeMap<String, usize> {
    COUNTS.with_borrow(|counts| counts.clone().unwrap())
}

#[derive(Default)]
struct Output {
    bytes: Vec<u8>,
    boundaries: Vec<BTreeMap<String, usize>>,
}
impl Write for Output {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.bytes.extend(bytes);
        for _ in bytes.iter().filter(|&&byte| byte == b'\n') {
            self.boundaries.push(snapshot());
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
}

#[test]
fn measure_word_reader_work() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let original = root.join("tests/fixtures/cpu-read-bounds/unresolved/word-edges.bend");
    let source = std::fs::read_to_string(&original).unwrap();
    let ops: Vec<_> = source.lines().filter(|line| line.contains("IO.print"))
        .map(|line| line.trim().trim_start_matches("Unit <- ")).collect();
    let directory = std::env::temp_dir().join(format!("teamy-bend-word-measure-{}",std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    for (name, indices) in (0..ops.len()).map(|i| (format!("individual-{:02}", i + 1),vec![i]))
        .chain([4,5,6,16].map(|n| (format!("prefix-{n:02}"), (0..n).collect()))) {
        let path = if name == "prefix-16" { original.clone() } else {
            let mut generated = format!("import Base\nimport {} as R\n\ndef main() -> IO(Unit):\n  do IO<Unit>:\n",original.parent().unwrap().parent().unwrap().join("original/read_bounds.bend").display());
            for (i, index) in indices.iter().enumerate() {
                generated.push_str(&format!("    {}{}\n", if i+1<indices.len(){"Unit <- "} else {""},ops[*index]));
            }
            let path = directory.join(format!("{name}.bend"));
            std::fs::write(&path,generated).unwrap();
            path
        };
        let checked = crate::kernel::check_executable(&crate::syntax::load_executable(&path).unwrap()).unwrap();
        for rep in [1,2] {
            COUNTS.with_borrow_mut(|counts| *counts=Some(BTreeMap::new()));
            let mut out = Output::default();
            let mut err=Vec::new();
            let result=checked.run_main(&mut out,&mut err,&|| false);
            let counts=snapshot();
            COUNTS.with_borrow_mut(|counts| *counts=None);
            eprintln!("RESULT|{name}|{rep}|{result:?}|{:?}|{:?}",String::from_utf8(out.bytes).unwrap(),String::from_utf8(err).unwrap());
            for (boundary,counts) in out.boundaries.iter().enumerate().map(|(i,c)| (format!("line{}",i+1),c)).chain(std::iter::once(("final".to_owned(),&counts))) {
                for (key,value) in counts {eprintln!("COUNT|{name}|{rep}|{boundary}|{key}|{value}");}
            }
        }
    }
    std::fs::remove_dir_all(directory).unwrap();
}
