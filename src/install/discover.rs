use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Result};

use crate::install::target::InstallTarget;

/// Fait naître les cibles d'installation d'un dossier : trouve ses scripts de
/// mise à jour et en dérive un nom lisible pour chacun.
///
/// C'est la sage-femme du domaine : une cible déjà construite *a* son script,
/// elle ne le cherche pas elle-même. Un dossier peut accoucher de plusieurs
/// cibles — une par script `update-*.sh`. Le chemin est canonicalisé pour que
/// les comparaisons (doublons) et la persistance reposent sur des chemins
/// absolus.
///
/// Règle de découverte : chaque script `update-*.sh` (convention des dossiers
/// d'install, ex. `update-lazygit.sh`) donne naissance à une cible distincte ;
/// à défaut de tout `update-*.sh`, une unique cible sur le premier `*.sh`.
pub fn discover(folder: &Path) -> Result<Vec<InstallTarget>> {
    if !folder.is_dir() {
        bail!("{} is not a directory", folder.display());
    }
    let folder = fs::canonicalize(folder)
        .map_err(|e| anyhow!("cannot resolve {}: {e}", folder.display()))?;

    let scripts = find_scripts(&folder);
    if scripts.is_empty() {
        bail!("no *.sh script found in {}", folder.display());
    }

    Ok(scripts
        .into_iter()
        .map(|script| {
            let name = derive_name(&script, &folder);
            InstallTarget::new(name, folder.clone(), script)
        })
        .collect())
}

/// Cherche les scripts de mise à jour du dossier : tous les `update-*.sh` par
/// ordre alphabétique, sinon le premier `*.sh` seul.
fn find_scripts(folder: &Path) -> Vec<PathBuf> {
    let mut scripts: Vec<PathBuf> = fs::read_dir(folder)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && has_sh_extension(path))
        .collect();
    scripts.sort();

    let updates: Vec<PathBuf> = scripts
        .iter()
        .filter(|path| starts_with_update(path))
        .cloned()
        .collect();
    if !updates.is_empty() {
        return updates;
    }
    scripts.into_iter().next().into_iter().collect()
}

fn has_sh_extension(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some("sh")
}

fn starts_with_update(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.starts_with("update-"))
        .unwrap_or(false)
}

/// Dérive un nom lisible : `update-lazygit.sh` -> `lazygit`, sinon le nom du
/// dossier (ex. `lazygitinstall`).
fn derive_name(script: &Path, folder: &Path) -> String {
    let stem = script.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    if let Some(rest) = stem.strip_prefix("update-") {
        if !rest.is_empty() {
            return rest.to_string();
        }
    }
    folder
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("?")
        .to_string()
}
