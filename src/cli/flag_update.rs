use reqwest::blocking::Client;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const LATEST_RELEASE_URL: &str = "https://api.github.com/repos/dexoron/dcr/releases/latest";

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Deserialize)]
struct ReleaseAsset {
    name: String,
    browser_download_url: String,
}

pub fn flag_update(args: &[String]) -> i32 {
    if !args.is_empty() {
        println!("Ошибка: команда не поддежривает аргументы");
        return 1;
    }

    let current_version = env!("CARGO_PKG_VERSION");
    let target = env!("DCR_TARGET");
    let client = match Client::builder().user_agent("dcr-updater").build() {
        Ok(client) => client,
        Err(_) => {
            println!("Ошибка: не удалось инициализировать HTTP клиент");
            return 1;
        }
    };

    let release = match fetch_latest_release(&client) {
        Ok(release) => release,
        Err(err) => {
            println!("Ошибка: не удалось проверить обновления: {err}");
            return 1;
        }
    };

    let latest_version = release.tag_name.trim_start_matches('v');
    if latest_version == current_version {
        println!("Установлена последняя версия: {current_version}");
        return 0;
    }

    let candidate_names = asset_candidates(target);
    let Some(asset) = release
        .assets
        .iter()
        .find(|asset| candidate_names.iter().any(|name| name == &asset.name))
    else {
        println!("Ошибка: не найден бинарник для таргета {target}");
        return 1;
    };

    let bytes = match download_asset(&client, &asset.browser_download_url) {
        Ok(bytes) => bytes,
        Err(err) => {
            println!("Ошибка: не удалось скачать обновление: {err}");
            return 1;
        }
    };

    let current_exe = match std::env::current_exe() {
        Ok(path) => path,
        Err(_) => {
            println!("Ошибка: не удалось определить путь текущего бинарника");
            return 1;
        }
    };
    let temp_path = temp_binary_path(&current_exe);

    if fs::write(&temp_path, &bytes).is_err() {
        println!("Ошибка: не удалось записать временный бинарник");
        return 1;
    }
    set_executable_permissions(&temp_path);

    if self_replace::self_replace(&temp_path).is_err() {
        let _ = fs::remove_file(&temp_path);
        println!("Ошибка: не удалось заменить текущий бинарник");
        return 1;
    }

    let _ = fs::remove_file(&temp_path);
    println!("Обновление завершено: {current_version} -> {latest_version}");
    0
}

fn fetch_latest_release(client: &Client) -> Result<Release, String> {
    let response = client
        .get(LATEST_RELEASE_URL)
        .send()
        .map_err(|_| "запрос к GitHub API завершился ошибкой".to_string())?;

    if !response.status().is_success() {
        return Err(format!("GitHub API вернул статус {}", response.status()));
    }

    response
        .json::<Release>()
        .map_err(|_| "ответ GitHub API имеет неожиданный формат".to_string())
}

fn download_asset(client: &Client, url: &str) -> Result<Vec<u8>, String> {
    let response = client
        .get(url)
        .send()
        .map_err(|_| "запрос на скачивание завершился ошибкой".to_string())?;

    if !response.status().is_success() {
        return Err(format!("скачивание вернуло статус {}", response.status()));
    }

    response
        .bytes()
        .map(|bytes| bytes.to_vec())
        .map_err(|_| "не удалось прочитать скачанные данные".to_string())
}

fn asset_candidates(target: &str) -> Vec<String> {
    let mut names = vec![format!("dcr-{target}")];

    match target {
        "x86_64-pc-windows-msvc" => {
            names.push("dcr-x86_64-pc-windows-msvc.exe".to_string());
        }
        _ => {}
    }

    names
}

fn temp_binary_path(current_exe: &Path) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis())
        .unwrap_or(0);
    let mut extension = format!("new-{stamp}");
    if cfg!(windows) {
        extension.push_str(".exe");
    }
    current_exe.with_extension(extension)
}

fn set_executable_permissions(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = fs::metadata(path) {
            let mut perms = meta.permissions();
            perms.set_mode(0o755);
            let _ = fs::set_permissions(path, perms);
        }
    }
}
