use std::{
    fmt::format,
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, LazyLock},
    vec,
};

use bytes::Bytes;
use pit::{
    http::HttpClient,
    piston::{
        self,
        rule::Features,
        version::{FetchedAssetIndex, FetchedVersion, VersionManifest},
    },
};
use tokio::{io::AsyncWriteExt, time::Instant};

slint::include_modules!();

pub static RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .expect("Failed to build runtime")
});

pub static CLIENT: LazyLock<HttpClient> = LazyLock::new(|| HttpClient::new());

async fn download_to_file(
    path: String,
    url: String,
    sha1: Option<String>,
) -> anyhow::Result<Bytes> {
    let mut full_path = PathBuf::new();
    full_path.push(std::env::current_exe()?.parent().unwrap());
    full_path.push(path);
    if tokio::fs::try_exists(&full_path).await? {
        return Ok(Bytes::from(tokio::fs::read(full_path).await?));
    }

    let data = CLIENT.download(&url, sha1.as_deref()).await?;

    let dir_path = full_path.parent().unwrap();
    if !tokio::fs::try_exists(dir_path).await? {
        tokio::fs::create_dir_all(dir_path).await?;
    }

    let mut file = tokio::fs::File::create(&full_path).await?;
    file.write_all(&data).await?;
    file.flush().await?;

    Ok(Bytes::from(data))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ui = AppWindow::new()?;

    ui.on_gerald(|| {
        RUNTIME.spawn(async {
            let version_manifest = serde_json::from_slice::<VersionManifest>(
                &CLIENT
                    .download(piston::VERSION_MANIFEST_URL, None)
                    .await
                    .unwrap(),
            )
            .unwrap();

            for version in version_manifest.versions {
                if version.id == version_manifest.latest.release {
                    let start_time = Instant::now();

                    let fetched_version = serde_json::from_slice::<FetchedVersion>(
                        &CLIENT
                            .download(&version.url, Some(&version.sha1))
                            .await
                            .unwrap(),
                    )
                    .unwrap();

                    let asset_index = fetched_version.asset_index;
                    let fetched_asset_index = serde_json::from_slice::<FetchedAssetIndex>(
                        &download_to_file(
                            format!("assets/indexes/{}.json", fetched_version.assets),
                            asset_index.url,
                            Some(asset_index.sha1),
                        )
                        .await
                        .unwrap(),
                    )
                    .unwrap();

                    let mut classpath = Vec::new();

                    classpath.push("client.jar".to_string());
                    download_to_file(
                        "client.jar".to_string(),
                        fetched_version.downloads.client.url,
                        Some(fetched_version.downloads.client.sha1),
                    )
                    .await
                    .unwrap();

                    for library in fetched_version.libraries {
                        if library
                            .rules
                            .is_none_or(|rules| rules.iter().all(|rule| rule.test(Features::EMPTY)))
                        {
                            if let Some(artifact) = library.downloads.artifact {
                                let path = format!("libraries/{}", artifact.path);
                                classpath.push(path.clone());
                                download_to_file(path, artifact.url, Some(artifact.sha1.clone()))
                                    .await
                                    .unwrap();
                            }
                        }
                    }

                    for object in fetched_asset_index.objects.values() {
                        let path = format!("{}/{}", &object.hash[..2], object.hash);
                        download_to_file(
                            format!("assets/objects/{}", path),
                            format!("https://resources.download.minecraft.net/{}", path),
                            Some(object.hash.clone()),
                        )
                        .await
                        .unwrap();
                    }

                    println!(
                        "Finished downloading in {} seconds!",
                        start_time.elapsed().as_secs_f32()
                    );

                    println!("Launching...");

                    let arguments = fetched_version.arguments.compile(
                        |placeholder| match placeholder {
                            "version_name" => Some(version.id.clone()),
                            "version_type" => {
                                Some(format!("{:?}", fetched_version.version_type).to_lowercase())
                            }
                            "assets_index_name" => Some(fetched_version.assets.clone()),
                            "launcher_name" => Some("Lychee Launcher".to_string()),
                            "launcher_version" => Some(env!("CARGO_PKG_VERSION").to_string()),
                            "auth_access_token" => Some("NOTYET".to_string()),
                            "game_directory" => Some("run".to_string()),
                            "game_assets" | "assets_root" => Some("assets".to_string()),
                            "natives_directory" => Some("natives".to_string()),
                            "classpath" => Some(classpath.join(":")),
                            _ => None,
                        },
                        Features::EMPTY,
                    );

                    println!("{:#?}", arguments);

                    Command::new("java")
                        .args(arguments.jvm)
                        .arg(fetched_version.main_class)
                        .arg("-jar")
                        .arg("client.jar")
                        .args(arguments.game)
                        .current_dir(std::env::current_exe().unwrap().parent().unwrap())
                        .spawn()
                        .expect("Failed to launch");
                }
            }
        });
    });

    ui.run()?;

    Ok(())
}
