use std::{
    env,
    ffi::OsStr,
    fs::{self, File},
    io::Read,
};

use dotenv::dotenv;
use libsql::Builder;
use tokio::runtime::Runtime;

use crate::{
    download::ModDetail,
    parse::parse,
    statistics::{bounding_box, room_details},
};

pub fn upload_stats() {
    // https://nunomaduro.com/load_environment_variables_from_dotenv_files_in_your_rust_program
    dotenv().ok();

    let mods_list: Vec<ModDetail> =
        serde_yaml::from_str(&fs::read_to_string("mods/mods_list.yaml").unwrap()).unwrap();
    let mut maps = vec![];

    for mod_detail in mods_list {
        let zip_archive = File::open(format!("mods/{}.zip", mod_detail.gamebanana_id))
            .map(zip::ZipArchive::new);
        if let Ok(Ok(mut zip_archive)) = zip_archive {
            for i in 0..zip_archive.len() {
                let file = zip_archive.by_index(i).unwrap();
                if file.is_file()
                    && file.enclosed_name().and_then(|name| name.extension())
                        == Some(OsStr::new("bin"))
                {
                    let file_name = file
                        .enclosed_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .split('/')
                        .last()
                        .unwrap()
                        .to_string();
                    let file_content = file.bytes().map(|byte| byte.unwrap()).collect::<Vec<_>>();
                    if let Ok(map) = parse(&file_content) {
                        maps.push((
                            format!("{} / {}", mod_detail.name, file_name),
                            room_details(&map).unwrap(),
                            bounding_box(&map).unwrap(),
                        ));
                    }
                }
            }
        }
    }

    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let url = env::var("LIBSQL_URL").expect("LIBSQL_URL must be set.");
        let token = env::var("LIBSQL_AUTH_TOKEN").unwrap_or_default();

        let db = Builder::new_remote(url, token).build().await.unwrap();
        let conn = db.connect().unwrap();

        for (name, room_detail, bounding_box) in maps {
            let mut stmt = conn
                .prepare("INSERT INTO maps(name, map, width, height) VALUES (?1, ?2, ?3, ?4);")
                .await
                .unwrap();
            stmt.execute((
                name,
                serde_json::to_string(&room_detail).unwrap(),
                bounding_box.width,
                bounding_box.height,
            ))
            .await
            .unwrap();
        }
    });
}
