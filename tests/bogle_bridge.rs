//! Framebuffer bridge contract, exercised without a display or graphics dependencies.

#[cfg(feature = "llvm")]
mod native {
    use mdhavers::{parse, LLVMCompiler};
    use std::path::PathBuf;
    use std::process::{Command, Output, Stdio};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{Duration, Instant};

    struct Executable(PathBuf);

    impl Executable {
        fn compile(source: &str) -> Self {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let scratch = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".scratch");
            std::fs::create_dir_all(&scratch).expect("create scratch directory");
            let executable = Self(scratch.join(format!(
                "bogle-bridge-{}-{}{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed),
                std::env::consts::EXE_SUFFIX
            )));
            LLVMCompiler::new()
                .compile_to_native(
                    &parse(source).expect("parse bridge program"),
                    &executable.0,
                    2,
                )
                .expect("compile bridge program");
            executable
        }

        fn run(&self) -> Output {
            let mut command = Command::new(&self.0);
            command.env("MDH_SCREEN", "none");
            bounded_output(command)
        }
    }

    impl Drop for Executable {
        fn drop(&mut self) {
            if self.0.exists() {
                std::fs::remove_file(&self.0).expect("remove native bridge test executable");
            }
        }
    }

    fn bounded_output(mut command: Command) -> Output {
        let mut child = command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("start subprocess");
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            if child.try_wait().expect("poll subprocess").is_some() {
                return child.wait_with_output().expect("read subprocess output");
            }
            if Instant::now() >= deadline {
                child.kill().expect("kill timed out subprocess");
                let output = child.wait_with_output().expect("reap timed out subprocess");
                panic!("bridge subprocess timed out: {output:?}");
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn run_ok(source: &str) -> Output {
        let output = Executable::compile(source).run();
        assert!(output.status.success(), "native bridge failed: {output:?}");
        output
    }

    #[test]
    fn sound_null_lifecycle_counts_scalar_samples_and_rejects_bad_input() {
        let output = run_ok(
            r#"
blether sound_close()
blether sound_queued()
blether sound_open(22050, 2)
ken pcm = bytes(12)
blether sound_queue(pcm)
bytes_set(pcm, 0, 255)
blether sound_queued()
sound_queue(bytes(4))
blether sound_queued()
sound_close()
blether sound_queued()
sound_open(44100, 1)
sound_queue(bytes(2))
blether sound_queued()
sound_close()
"#,
        );
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            "naething\n0\nnaething\nnaething\n6\n8\n0\n1\n"
        );
        assert!(output.stderr.is_empty(), "{output:?}");
        let cases = [
            ("sound_queue(bytes(2))", "sound_queue: nae sound is open"),
            (
                "sound_open(48000, 1)",
                "sound_open wants rate 22050 or 44100 and channels 1 or 2",
            ),
            (
                "sound_open(22050.0, 1)",
                "sound_open wants rate 22050 or 44100 and channels 1 or 2",
            ),
            (
                "sound_open(22050, 3)",
                "sound_open wants rate 22050 or 44100 and channels 1 or 2",
            ),
            (
                "sound_open(22050, 2)\nsound_open(22050, 2)",
                "sound_open: sound is already open",
            ),
            (
                "sound_open(22050, 2)\nsound_queue(bytes(2))",
                "sound_queue wants complete signed16 PCM frames",
            ),
            (
                "sound_open(22050, 1)\nsound_queue(bytes(1))",
                "sound_queue wants complete signed16 PCM frames",
            ),
            (
                "sound_open(22050, 1)\nsound_queue([])",
                "sound_queue wants bytes PCM",
            ),
            (
                "sound_open(22050, 1)\nsound_queue(bytes(1048578))",
                "sound_queue: PCM ring is full",
            ),
            (
                "sound_open(22050, 1)\nsound_queue(bytes(1048576))\nsound_queue(bytes(2))",
                "sound_queue: PCM ring is full",
            ),
        ];
        let mut source = String::new();
        for (call, _) in cases {
            source.push_str(&format!("hae_a_bash {{ {call}\nblether \"UNEXPECTED SUCCESS\" }} gin_it_gangs_wrang e {{ blether e }}\nsound_close()\n"));
        }
        let output = run_ok(&source);
        assert_eq!(
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .collect::<Vec<_>>(),
            cases
                .iter()
                .map(|(_, e)| format!("Jings! Something went awfy wrang: {e}"))
                .collect::<Vec<_>>()
        );
        let output = run_ok(
            r#"
dae check(f) {
    ken t = thread_spawn(f, [])
    hae_a_bash { thread_join(t)
blether "UNEXPECTED SUCCESS" } gin_it_gangs_wrang e { blether e }
}
check(|| sound_open(22050, 1))
check(|| sound_queue(bytes(2)))
check(|| sound_queued())
check(|| sound_close())
"#,
        );
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            "Jings! Something went awfy wrang: Nae sound aff the main thread!\n".repeat(4)
        );
        #[cfg(target_os = "macos")]
        {
            let executable = Executable::compile("sound_open(22050, 1)\nsound_queue(bytes(4))\nblether sound_queued()\nsound_close()\n");
            let mut sound_only = Command::new(&executable.0);
            sound_only.env_remove("MDH_SCREEN").env("MDH_SOUND", "none");
            let output = bounded_output(sound_only);
            assert!(output.status.success(), "{output:?}");
            assert_eq!(String::from_utf8_lossy(&output.stdout), "2\n");
            let mut command = Command::new("otool");
            command.arg("-L").arg(&executable.0);
            let output = bounded_output(command);
            let libraries = String::from_utf8_lossy(&output.stdout);
            assert!(libraries.contains("AudioToolbox"), "{libraries}");
            for forbidden in ["AppKit", "QuartzCore", "CoreGraphics", "libobjc"] {
                assert!(
                    !libraries.contains(forbidden),
                    "unexpected {forbidden}: {libraries}"
                );
            }
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn sound_ring_callback_copies_wraps_drains_and_zero_fills() {
        let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let executable = Executable(
            repo.join(".scratch")
                .join(format!("bogle-sound-ring-{}", std::process::id())),
        );
        let mut command = Command::new("clang");
        command
            .arg("-Wall")
            .arg("-Wextra")
            .arg("-Werror")
            .arg(repo.join("tests/fixtures/bogle_sound_ring.c"))
            .args(["-framework", "AudioToolbox", "-o"])
            .arg(&executable.0);
        let output = bounded_output(command);
        assert!(output.status.success(), "ring harness build: {output:?}");
        let output = executable.run();
        assert!(output.status.success(), "ring harness: {output:?}");
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            "sound ring copy/wrap/in-flight/underrun OK\n"
        );
    }

    fn fnv1a32(bytes: &[u8]) -> u32 {
        bytes.iter().fold(2_166_136_261, |hash, byte| {
            (hash ^ u32::from(*byte)).wrapping_mul(16_777_619)
        })
    }

    #[test]
    fn offscreen_bridge_converts_frames_and_exposes_every_name() {
        let output = run_ok(
            r#"
blether screen_close()
blether screen_open(256, 2, "Bogle bridge")
blether screen_width()
blether screen_height()
blether screen_should_close()
blether key_down("a")
blether key_pressed("escape")
blether mouse_x()
blether mouse_y()
blether mouse_dx()
blether mouse_dy()
blether mouse_down(0)
blether mouse_down(1)
blether mouse_down(2)
blether screen_grab_mouse(aye)
blether screen_grab_mouse(nae)
blether screen_focused()
ken palette = bytes(768)
ken pixels = bytes(512)
ken i = 0
whiles i < 256 {
    bytes_set(palette, i * 3, i)
    bytes_set(palette, i * 3 + 1, (i * 3 + 17) % 256)
    bytes_set(palette, i * 3 + 2, 255 - i)
    bytes_set(pixels, i, i)
    bytes_set(pixels, 256 + i, 255 - i)
    i = i + 1
}
blether screen_shaw(pixels, palette)
ken raw = bytes(2048)
i = 0
whiles i < 2048 {
    bytes_set(raw, i, (i * 13 + 7) % 256)
    i = i + 1
}
blether screen_shaw(raw, naething)
blether screen_close()
blether screen_close()
blether screen_open(1, 1, "Reopen")
blether screen_shaw(bytes(4), naething)
blether screen_close()
"#,
        );
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            "naething\nnaething\n256\n2\nnae\nnae\nnae\n0\n0\n0\n0\nnae\nnae\nnae\nnaething\nnaething\naye\nnaething\nnaething\nnaething\nnaething\nnaething\nnaething\nnaething\n"
        );
        // Compute expected BGRA independently of the bridge, including all 256 colours.
        let mut bgra = Vec::new();
        for index in (0u16..256).chain((0u16..256).rev()) {
            bgra.extend_from_slice(&[
                (255 - index) as u8,
                ((index * 3 + 17) % 256) as u8,
                index as u8,
                255,
            ]);
        }
        let raw: Vec<u8> = (0..2048).map(|i| ((i * 13 + 7) % 256) as u8).collect();
        assert_eq!(
            String::from_utf8_lossy(&output.stderr),
            format!(
                "mdh_screen frame 1 {:08x}\nmdh_screen frame 2 {:08x}\nmdh_screen frame 1 {:08x}\n",
                fnv1a32(&bgra),
                fnv1a32(&raw),
                fnv1a32(&[0; 4])
            )
        );
    }

    #[test]
    fn bridge_rejects_invalid_arguments_and_window_lifecycle() {
        let cases = [
            (
                "screen_shaw(bytes(1), bytes(768))",
                "screen_shaw: nae windae is open",
            ),
            (
                "screen_open(0, 2, \"x\")",
                "screen_open wants positive integer dimensions",
            ),
            (
                "screen_open(2, -1, \"x\")",
                "screen_open wants positive integer dimensions",
            ),
            (
                "screen_open(2.5, 2, \"x\")",
                "screen_open wants positive integer dimensions",
            ),
            (
                "screen_open(2, \"2\", \"x\")",
                "screen_open wants positive integer dimensions",
            ),
            ("screen_open(2, 2, 7)", "screen_open wants a string title"),
            ("key_down(7)", "key_down wants a string key"),
            ("key_pressed(nae)", "key_pressed wants a string key"),
            ("mouse_down(-1)", "mouse_down wants a button frae 0 tae 2"),
            ("mouse_down(3)", "mouse_down wants a button frae 0 tae 2"),
            (
                "mouse_down(\"0\")",
                "mouse_down wants a button frae 0 tae 2",
            ),
            ("mouse_down(1.0)", "mouse_down wants a button frae 0 tae 2"),
            ("screen_grab_mouse(1)", "screen_grab_mouse wants aye or nae"),
            ("screen_focused()", "Nae windae! Open the screen first"),
            (
                "screen_open(2, 2, \"again\")",
                "screen_open: a windae is already open",
            ),
            (
                "screen_shaw(bytes(3), bytes(768))",
                "screen_shaw: pixel length doesnae match the windae",
            ),
            (
                "screen_shaw(bytes(15), naething)",
                "screen_shaw: pixel length doesnae match the windae",
            ),
            (
                "screen_shaw(bytes(4), bytes(767))",
                "screen_shaw wants a 768-byte palette or naething",
            ),
            (
                "screen_shaw(bytes(4), bytes(769))",
                "screen_shaw wants a 768-byte palette or naething",
            ),
            (
                "screen_shaw([0, 0, 0, 0], bytes(768))",
                "screen_shaw wants bytes pixels",
            ),
            (
                "screen_shaw(bytes(4), \"palette\")",
                "screen_shaw wants a 768-byte palette or naething",
            ),
        ];
        let mut source = String::new();
        for (index, (call, _)) in cases.iter().enumerate() {
            if index == 14 {
                source.push_str("screen_open(2, 2, \"Arguments\")\n");
            }
            source.push_str(&format!(
                "hae_a_bash {{ {call}\nblether \"UNEXPECTED SUCCESS {index}\" }} gin_it_gangs_wrang e {{ blether e }}\n"
            ));
        }
        source.push_str("screen_close()\n");
        let output = run_ok(&source);
        assert_eq!(
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .collect::<Vec<_>>(),
            cases
                .iter()
                .map(|(_, expected)| format!("Jings! Something went awfy wrang: {expected}"))
                .collect::<Vec<_>>()
        );
        assert!(output.stderr.is_empty(), "{output:?}");
    }

    #[test]
    fn bridge_rejects_calls_from_worker_threads() {
        // Worker entry catches an uncaught hurl; thread_join rethrows it on the main thread.
        let mut source = String::from(
            "screen_open(1, 1, \"Main thread\")\n\
             dae worker_check(f) {\n\
                 ken worker = thread_spawn(f, [])\n\
                 hae_a_bash {\n\
                     thread_join(worker)\n\
                     blether \"UNEXPECTED WORKER SUCCESS\"\n\
                 } gin_it_gangs_wrang e { blether e }\n\
             }\n",
        );
        let calls = [
            "screen_open(1, 1, \"worker\")",
            "screen_shaw(bytes(1), bytes(768))",
            "screen_should_close()",
            "screen_close()",
            "screen_width()",
            "screen_height()",
            "key_down(\"a\")",
            "key_pressed(\"a\")",
            "mouse_x()",
            "mouse_y()",
            "mouse_dx()",
            "mouse_dy()",
            "mouse_down(0)",
            "screen_grab_mouse(aye)",
            "screen_focused()",
        ];
        for call in calls {
            source.push_str(&format!("worker_check(|| {call})\n"));
        }
        source.push_str("screen_close()\n");
        let output = run_ok(&source);
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            "Jings! Something went awfy wrang: Nae windae aff the main thread!\n"
                .repeat(calls.len())
        );
        assert!(output.stderr.is_empty(), "{output:?}");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn programs_without_bridge_calls_do_not_link_window_frameworks() {
        let executable = Executable::compile("blether 42\n");
        let mut command = Command::new("otool");
        command.arg("-L").arg(&executable.0);
        let output = bounded_output(command);
        assert!(output.status.success(), "otool failed: {output:?}");
        let libraries = String::from_utf8_lossy(&output.stdout);
        for forbidden in [
            "AppKit",
            "QuartzCore",
            "CoreGraphics",
            "libobjc",
            "AudioToolbox",
        ] {
            assert!(
                !libraries.contains(forbidden),
                "unexpected {forbidden}: {libraries}"
            );
        }
    }
}

#[cfg(not(feature = "graphics"))]
#[test]
fn interpreter_without_graphics_reports_nae_windae_for_every_bridge_name() {
    use mdhavers::{parse, Interpreter};

    for call in [
        "screen_open(1, 1, \"x\")",
        "screen_shaw(bytes(1), bytes(768))",
        "screen_should_close()",
        "screen_close()",
        "screen_width()",
        "screen_height()",
        "key_down(\"a\")",
        "key_pressed(\"a\")",
        "mouse_x()",
        "mouse_y()",
        "mouse_dx()",
        "mouse_dy()",
        "mouse_down(0)",
        "screen_grab_mouse(aye)",
        "screen_focused()",
    ] {
        let program = parse(call).expect("parse interpreter bridge call");
        let error = Interpreter::new()
            .interpret(&program)
            .expect_err(call)
            .to_string();
        assert!(error.contains("Nae windae"), "{call}: {error}");
    }
}

#[test]
fn sound_interpreter_null_backend_is_per_interpreter_and_bounded() {
    use mdhavers::{parse, Interpreter};
    let mut interpreter = Interpreter::new();
    for source in [
        "sound_close()\nsound_open(22050, 2)\nsound_queue(bytes(12))\ngin sound_queued() != 6 { hurl \"scalar sample count\" }",
        "sound_close()\nsound_close()\ngin sound_queued() != 0 { hurl \"close reset\" }\nsound_open(44100, 1)\nsound_queue(bytes(2))\ngin sound_queued() != 1 { hurl \"mono count\" }",
    ] {
        interpreter.interpret(&parse(source).unwrap()).unwrap();
    }
    let error = interpreter
        .interpret(&parse("sound_queue(bytes(1048576))").unwrap())
        .unwrap_err();
    assert!(error.to_string().contains("PCM ring is full"), "{error}");
    let mut other = Interpreter::new();
    other
        .interpret(&parse("gin sound_queued() != 0 { hurl \"shared state\" }").unwrap())
        .unwrap();
    let error = other
        .interpret(&parse("sound_queue(bytes(2))").unwrap())
        .unwrap_err();
    assert!(error.to_string().contains("nae sound is open"), "{error}");
}
