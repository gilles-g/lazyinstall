use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Forme persistée : la liste des dossiers suivis.
#[derive(Debug, Default, Serialize, Deserialize)]
struct StoreData {
    folders: Vec<String>,
}

/// Emplacement du fichier de configuration :
/// `~/.config/lazyinstall/targets.json`.
pub fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("lazyinstall").join("targets.json"))
}

/// Charge la liste des dossiers suivis. Liste vide si le fichier est absent ou
/// illisible — on ne casse jamais le démarrage pour une config corrompue.
pub fn load() -> Vec<PathBuf> {
    let Some(path) = config_path() else {
        return Vec::new();
    };
    let Ok(content) = fs::read_to_string(&path) else {
        return Vec::new();
    };
    let data: StoreData = serde_json::from_str(&content).unwrap_or_default();
    data.folders.into_iter().map(PathBuf::from).collect()
}

/// Enregistre l'ensemble des dossiers suivis. Plusieurs cibles pouvant partager
/// le même dossier, on déduplique ici (en conservant l'ordre) : on stocke un
/// ensemble de dossiers, pas une cible par ligne.
pub fn save(folders: &[PathBuf]) -> Result<()> {
    let path = config_path().context("dossier de configuration introuvable")?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("création de {}", parent.display()))?;
    }
    let mut seen = HashSet::new();
    let data = StoreData {
        folders: folders
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .filter(|folder| seen.insert(folder.clone()))
            .collect(),
    };
    let json = serde_json::to_string_pretty(&data)?;
    fs::write(&path, json).with_context(|| format!("écriture de {}", path.display()))?;
    Ok(())
}
