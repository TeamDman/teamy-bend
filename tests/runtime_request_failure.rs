// SPDX-License-Identifier: MPL-2.0
#[path = "support/executable_c_compiler.rs"]
mod executable_c_compiler;

use std::cell::RefCell;
use std::fs;
use std::io;
use std::io::Write;
use std::process::Command;
use std::rc::Rc;
use std::time::Duration;
use std::time::Instant;
use teamy_bend::compiler::compile_executable_c;
use teamy_bend::compiler::compile_executable_javascript;
use teamy_bend::kernel::check_executable;
use teamy_bend::syntax::load_executable;

const SOURCE: &str = include_str!("fixtures/cpu-runtime-negatives/request_out_of_band.bend");
const DIAGNOSTIC: &[u8] = b"bend: runtime fail-stop\n";

struct Cleanup(std::path::PathBuf);
impl Drop for Cleanup {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

type Events = Rc<RefCell<Vec<(&'static str, Vec<u8>)>>>;
struct Recorder(&'static str, Events);
impl Write for Recorder {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.1.borrow_mut().push((self.0, buf.to_vec()));
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        self.1.borrow_mut().push((self.0, vec![]));
        Ok(())
    }
}

#[test]
fn original_request_refusal_preserves_exact_streams_status_and_order() {
    let directory = std::env::temp_dir().join(format!("teamy-bend-request-{}", std::process::id()));
    fs::create_dir(&directory).unwrap();
    let _cleanup = Cleanup(directory.clone());
    let source = directory.join("main.bend");
    fs::write(&source, SOURCE).unwrap();
    let book = check_executable(&load_executable(&source).unwrap()).unwrap();

    let events = Events::default();
    assert_eq!(
        book.run_main(
            &mut Recorder("out", Rc::clone(&events)),
            &mut Recorder("err", Rc::clone(&events)),
            &|| false
        )
        .unwrap(),
        1
    );
    assert_eq!(
        *events.borrow(),
        vec![
            ("out", b"3".to_vec()),
            ("out", b"\n".to_vec()),
            ("out", vec![]),
            ("err", DIAGNOSTIC.to_vec()),
            ("err", vec![]),
        ]
    );

    let javascript = directory.join("main.cjs");
    fs::write(&javascript, compile_executable_javascript(&book).unwrap()).unwrap();
    let generated = compile_executable_c(&book).unwrap();
    let executable = executable_c_compiler::compile(&directory, &generated, &[]);
    let mut commands = vec![
        Command::new(env!("CARGO_BIN_EXE_teamy-bend")),
        Command::new(std::env::var_os("TEAMY_BEND_NODE").unwrap_or_else(|| "node".into())),
    ];
    commands[0].arg("run").arg(&source);
    commands[1].arg(&javascript);
    for threads in ["1", "2"] {
        let mut command = Command::new(&executable);
        command.args(["--threads", threads, "--gpu", "off"]);
        commands.push(command);
    }
    for command in &mut commands {
        let output = executable_c_compiler::bounded(command, Duration::from_secs(30));
        assert_eq!(output.status.code(), Some(1), "{command:?}: {output:?}");
        assert_eq!(output.stdout, b"3\n");
        assert_eq!(output.stderr, DIAGNOSTIC);
        // One shared file descriptor makes cross-stream ordering observable,
        // including C's buffered stdout when the diagnostic is emitted.
        let path = directory.join("merged.txt");
        let combined = fs::File::create(&path).unwrap();
        command
            .stdout(combined.try_clone().unwrap())
            .stderr(combined);
        let mut child = command.spawn().unwrap();
        let deadline = Instant::now() + Duration::from_secs(30);
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break status;
            }
            if Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("merged-stream execution exceeded its deadline");
            }
            std::thread::sleep(Duration::from_millis(10));
        };
        assert_eq!(status.code(), Some(1));
        assert_eq!(fs::read(path).unwrap(), b"3\nbend: runtime fail-stop\n");
    }
}
