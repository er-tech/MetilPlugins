use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256};

use super::model_file_contract::ModelFile;

/// Downloads `model` to `target`, checking its size and its sum, and puts it in place only when
/// both are right: a download that stops, or a file that is not the one pinned, leaves nothing.
/// While it goes on, `progress` is told how far it is, at most once in `every`: a host that hears
/// nothing for a while takes a plugin for stuck.
pub fn download_model(
    model: &ModelFile,
    target: &Path,
    progress: &dyn Fn(&str),
    every: Duration,
) -> Result<(), String> {
    let folder = target
        .parent()
        .ok_or_else(|| format!("{} has no folder", target.display()))?;
    std::fs::create_dir_all(folder).map_err(|error| format!("{}: {error}", folder.display()))?;
    let partial = target.with_extension("partial");
    let outcome = fetch(model, &partial, progress, every);
    if outcome.is_err() {
        let _ = std::fs::remove_file(&partial);
    }
    outcome?;
    std::fs::rename(&partial, target).map_err(|error| format!("{}: {error}", target.display()))
}

fn fetch(
    model: &ModelFile,
    partial: &Path,
    progress: &dyn Fn(&str),
    every: Duration,
) -> Result<(), String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(900)))
        .build()
        .into();
    let mut response = agent
        .get(model.url)
        .call()
        .map_err(|error| format!("{}: {error}", model.url))?;
    let mut reader = response.body_mut().as_reader();
    let mut file =
        File::create(partial).map_err(|error| format!("{}: {error}", partial.display()))?;
    let mut hash = Sha256::new();
    let mut buffer = vec![0u8; 64 * 1024];
    let mut bytes = 0u64;
    let mut told = Instant::now();
    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|error| format!("{}: {error}", model.url))?;
        if read == 0 {
            break;
        }
        hash.update(&buffer[..read]);
        file.write_all(&buffer[..read])
            .map_err(|error| format!("{}: {error}", partial.display()))?;
        bytes += read as u64;
        if told.elapsed() >= every {
            told = Instant::now();
            progress(&format!(
                "downloading {}: {} of {} MB",
                model.file,
                bytes / 1_000_000,
                model.bytes / 1_000_000
            ));
        }
    }
    file.flush()
        .map_err(|error| format!("{}: {error}", partial.display()))?;
    if bytes != model.bytes {
        return Err(format!(
            "{} gave {bytes} bytes, expected {}",
            model.url, model.bytes
        ));
    }
    let sum = format!("{:x}", hash.finalize());
    if sum != model.sha256 {
        return Err(format!(
            "{} is not the file expected (sha256 {sum}, expected {})",
            model.url, model.sha256
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    // The socket methods below need these traits on some systems and not on others.
    #[allow(unused_imports)]
    use std::io::{Read as _, Write as _};
    use std::net::TcpListener;

    use sha2::{Digest, Sha256};

    use super::*;

    /// A server on this computer that answers one request with `chunks` of a megabyte of the
    /// letter `a`, a tenth of a second apart, so a download of it takes a moment.
    fn slow_server(chunks: usize) -> (String, ModelFile) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url: &'static str = Box::leak(
            format!("http://{}/model.bin", listener.local_addr().unwrap()).into_boxed_str(),
        );
        let body = vec![b'a'; 1_000_000];
        let mut sum = Sha256::new();
        for _ in 0..chunks {
            sum.update(&body);
        }
        let sha256: &'static str = Box::leak(format!("{:x}", sum.finalize()).into_boxed_str());
        std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            let mut request = [0u8; 2048];
            let _ = socket.read(&mut request);
            let head = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                chunks * body.len()
            );
            socket.write_all(head.as_bytes()).unwrap();
            for _ in 0..chunks {
                socket.write_all(&body).unwrap();
                socket.flush().unwrap();
                std::thread::sleep(Duration::from_millis(100));
            }
        });
        (
            url.to_string(),
            ModelFile {
                file: "model.bin",
                url,
                sha256,
                bytes: (chunks * 1_000_000) as u64,
            },
        )
    }

    #[test]
    fn a_download_that_takes_a_while_says_how_far_it_is_and_ends_with_the_right_file() {
        let (_, model) = slow_server(5);
        let target = std::env::temp_dir()
            .join(format!("model-cache-progress-{}", std::process::id()))
            .join("model.bin");
        let told = RefCell::new(Vec::<String>::new());

        download_model(
            &model,
            &target,
            &|message| told.borrow_mut().push(message.to_string()),
            Duration::from_millis(150),
        )
        .unwrap();

        assert_eq!(std::fs::metadata(&target).unwrap().len(), 5_000_000);
        let told = told.into_inner();
        assert!(told.len() >= 2, "it said {told:?}");
        assert!(told[0].starts_with("downloading model.bin: "), "{told:?}");
        assert!(told.last().unwrap().ends_with("of 5 MB"), "{told:?}");
    }

    #[test]
    fn a_file_that_is_not_the_one_pinned_is_refused_and_leaves_nothing() {
        let (_, mut model) = slow_server(1);
        model.sha256 = "0000000000000000000000000000000000000000000000000000000000000000";
        let target = std::env::temp_dir()
            .join(format!("model-cache-refused-{}", std::process::id()))
            .join("model.bin");

        let error = download_model(&model, &target, &|_| {}, Duration::from_secs(1)).unwrap_err();

        assert!(error.contains("not the file expected"), "{error}");
        assert!(!target.exists());
        assert!(!target.with_extension("partial").exists());
    }
}
