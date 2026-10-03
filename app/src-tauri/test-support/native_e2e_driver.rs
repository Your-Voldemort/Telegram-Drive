//! Opt-in native backend process used exclusively by tests/native_e2e.rs.
#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
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
