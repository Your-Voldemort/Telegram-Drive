//! Opt-in native backend process used exclusively by tests/native_e2e.rs.
#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    if std::env::current_exe()
        .ok()
        .and_then(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().starts_with("heic-helper"))
        })
        .unwrap_or(false)
    {
        let result = heic_helper();
        std::process::exit(if result.is_ok() { 0 } else { 1 });
    }
    if std::env::args().nth(1).as_deref() == Some("-nostdin") {
        let result = (|| -> Result<(), String> {
            let root = std::env::current_exe()
                .map_err(|e| e.to_string())?
                .parent()
                .ok_or("No fixture root")?
                .to_path_buf();
            if std::fs::read_to_string(root.join(".native-e2e-fixture"))
                .map_err(|e| e.to_string())?
                != "telegram-drive-synthetic-e2e\n"
            {
                return Err("Not a fixture".into());
            }
            let args: Vec<_> = std::env::args().collect();
            let output = std::path::PathBuf::from(args.last().ok_or("No output")?);
            if !output.starts_with(&root) {
                return Err("Output outside fixture".into());
            }
            let mode =
                std::fs::read_to_string(root.join("ffmpeg-mode")).map_err(|e| e.to_string())?;
            std::fs::write(root.join("ffmpeg-pid"), std::process::id().to_string())
                .map_err(|e| e.to_string())?;
            match mode.trim() {
                "wait" => loop {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                },
                "frame-wait" => {
                    std::fs::copy(root.join("ffmpeg-frame.jpg"), &output)
                        .map_err(|e| e.to_string())?;
                    std::fs::write(root.join("ffmpeg-frame-ready"), b"ready")
                        .map_err(|e| e.to_string())?;
                    loop {
                        std::thread::sleep(std::time::Duration::from_millis(50));
                    }
                }
                "fail" => return Err("Controlled codec failure".into()),
                _ => {
                    std::fs::copy(root.join("ffmpeg-frame.jpg"), output)
                        .map_err(|e| e.to_string())?;
                }
            }
            Ok(())
        })();
        std::process::exit(if result.is_ok() { 0 } else { 1 });
    }
    if let Err(error) = app_lib::native_e2e::run().await {
        eprintln!("Native E2E driver failed: {error}");
        std::process::exit(1);
    }
}

// Controlled process fixture; this does not decode HEIF. Real-tool runs are separate.
fn heic_helper() -> Result<(), String> {
    let root = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .parent()
        .ok_or("Missing root")?
        .to_path_buf();
    if std::fs::read_to_string(root.join(".native-e2e-fixture")).map_err(|e| e.to_string())?
        != "telegram-drive-synthetic-e2e\n"
    {
        return Err("Not a fixture".into());
    }
    let mode = std::fs::read_to_string(root.join("heic-mode")).unwrap_or_else(|_| "success".into());
    let args: Vec<_> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("-version") {
        let version = std::fs::read_to_string(root.join("heic-version")).unwrap_or_else(|_| {
            if mode.trim() == "old" {
                "8.0.1".into()
            } else {
                "8.1.2".into()
            }
        });
        println!("ffmpeg version {}", version.trim());
        return Ok(());
    }
    std::fs::write(root.join("heic-pid"), std::process::id().to_string())
        .map_err(|e| e.to_string())?;
    match mode.trim() {
        "gate" => {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
            while !root.join("heic-release").is_file() {
                if std::time::Instant::now() >= deadline {
                    return Err("Fixture deadline".into());
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
        "fail" => return Err("Controlled failure".into()),
        "wait" => loop {
            std::thread::sleep(std::time::Duration::from_millis(20));
        },
        "memory" => {
            let mut memory: Vec<Vec<u8>> = Vec::new();
            loop {
                const CHUNK: usize = 8 * 1024 * 1024;
                let mut block = Vec::new();
                if block.try_reserve_exact(CHUNK).is_err() {
                    std::fs::write(
                        root.join("heic-memory-denied"),
                        ((memory.len() + 1) * CHUNK).to_string(),
                    )
                    .map_err(|e| e.to_string())?;
                    return Err("Controlled allocation refused by process limit".into());
                }
                block.resize(CHUNK, 0u8);
                for byte in block.iter_mut().step_by(4096) {
                    *byte = 1;
                }
                memory.push(block);
                std::hint::black_box(&memory);
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
        _ => {}
    }
    let output = std::path::PathBuf::from(args.last().ok_or("Missing output")?);
    if !output.starts_with(&root) {
        return Err("Output outside fixture".into());
    }
    if mode.trim() == "oversize" {
        std::fs::File::create(&output)
            .map_err(|e| e.to_string())?
            .set_len(9 * 1024 * 1024)
            .map_err(|e| e.to_string())?;
        return Ok(());
    }
    let frame = root.join("heic-frame.jpg");
    let source = if frame.is_file() {
        std::fs::read(frame).map_err(|e| e.to_string())?
    } else {
        include_bytes!("fixtures/heic/tiled-12mp-reference.jpg").to_vec()
    };
    let dimensions = args
        .windows(2)
        .find(|pair| pair[0] == "-s")
        .and_then(|pair| pair[1].split_once('x'))
        .and_then(|(w, h)| Some((w.parse::<u32>().ok()?, h.parse::<u32>().ok()?)));
    let size = image::ImageReader::new(std::io::Cursor::new(&source))
        .with_guessed_format()
        .map_err(|e| e.to_string())?
        .into_dimensions()
        .map_err(|e| e.to_string())?;
    if matches!(mode.trim(), "success" | "gate") && dimensions == Some(size) {
        return std::fs::write(output, source).map_err(|e| e.to_string());
    }
    let mut image = image::load_from_memory(&source).map_err(|e| e.to_string())?;
    if mode.trim() == "tile" {
        image = image.crop_imm(0, 0, 512.min(image.width()), 512.min(image.height()));
    } else if let Some((width, height)) = dimensions {
        image = image.thumbnail(width, height);
    }
    image
        .save_with_format(output, image::ImageFormat::Jpeg)
        .map_err(|e| e.to_string())
}
