use crate::{level::{level::LevelState, room::Chain}, renderer::texture::TextureId};
use core::result::Result;
use anyhow::Ok;
use cfg_if::cfg_if;
use itertools::Itertools;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap, ffi::OsStr, fs::{self, create_dir, read, read_dir, read_to_string}, ops::ControlFlow, path::{Path, PathBuf}, sync::Arc
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameConfigFile {
    pub level_order: Vec<String>,
}

impl GameConfigFile {
    pub fn new() -> Self {
        Self {
            level_order: vec![],
        }
    }
}

#[derive(Clone, Debug)]
pub struct GameData {
    pub config_file: GameConfigFile,
    pub levels: Vec<String>,
    pub levels_data: HashMap<String, LevelState>,
    pub textures: Vec<(TextureId, Arc<[u8]>, Box<str>)>,
    pub current_level: Option<String>,
}

impl GameData {
    pub fn new() -> Self {
        Self {
            config_file: GameConfigFile::new(),
            levels: vec![],
            levels_data: HashMap::new(),
            textures: vec![],
            current_level: None,
        }
    }
    pub fn update_config(&mut self) {
        self.config_file.level_order = self.levels.clone();
    }

    pub fn update_folder(&self, path: &PathBuf) -> anyhow::Result<()> {
        fs::write(path.clone().chain(|a|{a.push("config.ron")}), ron::ser::to_string_pretty(&self.config_file,PrettyConfig::new())?)?;
        if self.levels_data.iter().try_for_each(|(level_name,level)|{
            match fs::write(path.clone().chain(|a|{a.push(format!("levels/{}.ron",level_name))}), match ron::ser::to_string_pretty(&level,PrettyConfig::new()){
                Result::Ok(a) => a,
                Err(_) => return ControlFlow::Break(()),
            }){
                Result::Ok(_)=>{},
                Err(_)=>return ControlFlow::Break(())
            };
            return ControlFlow::Continue(());
        }).is_break(){
            return Ok(());
        };
        Ok(())
    }

    pub fn generate_new_game_folder(&self, path: PathBuf) -> anyhow::Result<()> {
        fs::remove_dir_all(&path).unwrap();
        fs::create_dir(&path).unwrap();
        let _ = create_dir(path.join("textures"));
        self.textures.iter().try_for_each(|texture| {
            fs::write(
                path.join(format!("textures/{}.{}", texture.0, texture.2)),
                texture.1.as_ref(),
            )?;
            Ok(())
        })?;
        let _ = create_dir(path.join("levels"));
        self.levels.iter().try_for_each(|level| {
            fs::write(
                path.join(format!("levels/{}.ron", level)),
                ron::ser::to_string_pretty(
                    &self.levels_data.get(level).expect(
                        "the only insert was with this very data so something bad happened",
                    ),
                    PrettyConfig::new(),
                )?
                .as_bytes(),
            )?;
            Ok(())
        })?;
        fs::write(
            path.join("config.ron"),
            ron::ser::to_string_pretty(&self.config_file, PrettyConfig::new())?.as_bytes(),
        )?;
        //Self::generate(&path).expect("need to fix folder/file generating function");
        Ok(())
    }
    pub fn generate(path: &PathBuf) -> Option<Self> {
        let mut textures: Vec<(TextureId, Arc<[u8]>, Box<str>)> = vec![];

        read_dir(path.join("textures"))
            .ok()?
            .into_iter()
            .for_each(|entry| {
                if let Some(filepath) = entry.ok() {
                    if filepath.path().has_extension(&["png", "jpg", "jpeg"]) {
                        let texture_fullname = filepath
                            .file_name()
                            .into_string()
                            .expect("non-unicode charecter in file name")
                            .to_string();
                        textures.push((
                            texture_fullname.split(".").collect_vec()[0].into(),
                            read(filepath.path())
                                .expect("file can't be read for some reason :(")
                                .into_boxed_slice()
                                .into(),
                            filepath
                                .path()
                                .extension()
                                .expect("can't get file extension")
                                .to_str()
                                .expect("couldn't turn extextion into &str")
                                .into(),
                        ));
                    }
                }
            });
        let config_file: GameConfigFile =
            ron::from_str(read_to_string(path.join("config.ron")).ok()?.as_str()).ok()?;
        let mut levels: Vec<String> = vec![];
        let mut levels_data: HashMap<String, LevelState> = HashMap::new();
        for level_name in &config_file.level_order {
            levels.push(level_name.clone());
            levels_data.insert(
                level_name.clone(),
                ron::from_str(
                    read_to_string(path.join(format!("levels/{}.ron", level_name)))
                        .ok()?
                        .as_str(),
                )
                .ok()?,
            );
        }
        Some(Self {
            current_level: None,
            config_file,
            textures,
            levels,
            levels_data,
        })
    }
}

pub trait FileExtension {
    fn has_extension<S: AsRef<str>>(&self, extensions: &[S]) -> bool;
}

impl<P: AsRef<Path>> FileExtension for P {
    fn has_extension<S: AsRef<str>>(&self, extensions: &[S]) -> bool {
        if let Some(ref extension) = self.as_ref().extension().and_then(OsStr::to_str) {
            return extensions
                .iter()
                .any(|x| x.as_ref().eq_ignore_ascii_case(extension));
        }

        false
    }
}

pub async fn load_string(file_name: &str) -> anyhow::Result<String> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            log::warn!("Load model on web");

            let url = format_url(file_name);
            let txt = reqwest::get(url)
                .await?
                .text()
                .await?;

            log::warn!("{}", txt);

        } else {
            let path = std::path::Path::new("assets")
                .join(file_name);
            let txt = std::fs::read_to_string(path)?;
        }
    }

    Ok(txt)
}

pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            let data = reqwest::get(url)
                .await?
                .bytes()
                .await?
                .to_vec();
        } else {
            let path = std::path::Path::new("assets")
                .join(file_name);
            let data = std::fs::read(path)?;
        }
    }

    Ok(data)
}
