use serde::Deserialize;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::{error::Error, time::Duration};
use tokio::runtime::Runtime;
use tokio::{fs, io::AsyncWriteExt};

#[derive(Debug, Deserialize)]
pub struct FileDetails {
    #[serde(rename = "URL")]
    pub url: String,
    #[serde(rename = "CreatedDate")]
    pub created_date: u64,
}

#[derive(Debug, Deserialize)]
pub struct ModDetail {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "GameBananaId")]
    pub gamebanana_id: u64,
    #[serde(rename = "Files")]
    pub files: Vec<FileDetails>,
    #[serde(rename = "CategoryName")]
    pub category_name: String,
}

async fn download_mod(
    client: &reqwest::Client,
    mod_detail: &ModDetail,
) -> Result<(), Box<dyn Error>> {
    let Some(latest_file) = mod_detail.files.iter().max_by_key(|file| file.created_date) else {
        return Err("No files available".into());
    };

    let content = client.get(&latest_file.url).send().await?.bytes().await?;
    let mut zip_file = fs::File::create(format!("mods/{}.zip", &mod_detail.gamebanana_id)).await?;
    zip_file.write_all(&content).await?;

    Ok(())
}

pub fn download_maps() -> Result<(), Box<dyn Error>> {
    let mut downloaded_ids = HashSet::new();
    for file in std::fs::read_dir("mods")? {
        let file = file?;
        if file.path().is_file() && file.path().extension() == Some(OsStr::new("zip")) {
            let id = file
                .file_name()
                .to_str()
                .ok_or("Not a string")?
                .split('.')
                .next()
                .ok_or("Empty name")?
                .parse::<u64>()?;

            let file = zip::ZipArchive::new(std::fs::File::open(file.path())?);
            if let Ok(file) = file {
                if !file.is_empty() {
                    downloaded_ids.insert(id);
                }
            }
        }
    }

    let mut number_of_mods = 0;

    let rt = Runtime::new()?;
    rt.block_on(async {
        if !fs::try_exists("mods").await? {
            let _ = fs::create_dir("mods").await;
        }

        let client = reqwest::ClientBuilder::new()
            .timeout(Duration::from_secs(3 * 60))
            .build()?;

        let mods_list = if fs::try_exists("mods/mods_list.yaml").await? {
            fs::read_to_string("mods/mods_list.yaml").await?
        } else {
            let mods_list = client
                .get("https://maddie480.ovh/celeste/mod_search_database.yaml")
                .send()
                .await?
                .text()
                .await?;
            fs::write("mods/mods_list.yaml", &mods_list).await?;
            mods_list
        };

        let mods_list: Vec<ModDetail> = serde_yaml::from_str(&mods_list)?;
        let mut mods_list_iter = mods_list.into_iter();

        const DOWNLOADS_AT_TIME: usize = 100;
        let mut started_all_downloads = false;
        while !started_all_downloads {
            let mut downloads = vec![];
            while downloads.len() < DOWNLOADS_AT_TIME {
                let Some(mod_detail) = mods_list_iter.next() else {
                    started_all_downloads = true;
                    break;
                };

                if mod_detail.category_name == "Maps" {
                    number_of_mods += 1;
                    if !downloaded_ids.contains(&mod_detail.gamebanana_id) {
                        let client = client.clone();
                        downloads.push(tokio::spawn(async move {
                            for i in 0..3 {
                                if let Err(err) = download_mod(&client, &mod_detail).await {
                                    if i == 2 {
                                        eprintln!(
                                            "Mod {}({}) - Download error - {}",
                                            mod_detail.name, mod_detail.gamebanana_id, err
                                        );
                                    }
                                } else {
                                    println!(
                                        "Downloaded mod {}({})",
                                        mod_detail.name, mod_detail.gamebanana_id
                                    );
                                    break;
                                }
                            }
                        }));
                    }
                }
            }

            for download in downloads {
                download.await?;
            }
        }

        Ok::<(), Box<dyn Error>>(())
    })?;

    let mut number_of_downloaded_mods = 0;
    for file in std::fs::read_dir("mods")? {
        let file = file?;
        if file.path().is_file() && file.path().extension() == Some(OsStr::new("zip")) {
            let file = zip::ZipArchive::new(std::fs::File::open(file.path())?);
            if let Ok(file) = file {
                if !file.is_empty() {
                    number_of_downloaded_mods += 1;
                }
            }
        }
    }

    println!(
        "{} out of {} mods have been downloaded.",
        number_of_downloaded_mods, number_of_mods
    );

    Ok(())
}
