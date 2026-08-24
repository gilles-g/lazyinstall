# lazyinstall

Un TUI (terminal, style _lazygit_) pour **suivre des dossiers d'installation et lancer leurs mises à jour**.

Chaque dossier suivi contient un script de mise à jour (par convention `update-*.sh`).
Exemple : `~/lazygitinstall/` contient `update-lazygit.sh` et un sous-dossier `current/`
avec le binaire installé. `lazyinstall` liste ces dossiers, lance leur script à la demande
et affiche la sortie en direct.

## Aperçu

```
┌ lazyinstall — dossiers suivis ──────────────────────────────────────────┐
│ ● lazygit          à jour      /home/user/lazygitinstall              │
│ ○ composer         au repos    /home/user/lazycomposerinstall         │
└──────────────────────────────────────────────────────────────────────────┘
┌ Sortie : lazygit ─────────────────────────────────────────────────────────┐
│   Version installée : 0.62.1                                              │
│   Dernière release  : 0.62.2                                              │
│   lazygit mis à jour : 0.62.1 -> 0.62.2                                   │
└──────────────────────────────────────────────────────────────────────────┘
 [Enter/u] màj  [U] tout  [a] ajouter  [d] retirer  [j/k] naviguer  [q] quitter  [?] aide
```

## Installation

```bash
cargo install --path .
# ou, pour développer :
cargo run
```

## Utilisation

| Touche        | Action                                        |
|---------------|-----------------------------------------------|
| `j` / `k`, ↓/↑ | naviguer dans la liste                       |
| `Enter` / `u` | mettre à jour le dossier sélectionné          |
| `U`           | tout mettre à jour                            |
| `a`           | ajouter un dossier à suivre (saisie du chemin)|
| `d`           | retirer le dossier sélectionné                |
| `q` / `Ctrl-C`| quitter                                       |
| `?`           | afficher / masquer l'aide                     |

### Ajouter un dossier

Appuyez sur `a`, saisissez le chemin du dossier (le `~` est développé), puis `Enter`.
Le dossier doit contenir un script `*.sh` : `lazyinstall` privilégie un `update-*.sh`,
sinon prend le premier script shell trouvé. Le nom affiché est dérivé du script
(`update-lazygit.sh` → `lazygit`), à défaut du nom du dossier.

## Configuration

La liste des dossiers suivis est persistée dans :

```
~/.config/lazyinstall/targets.json
```

Les dossiers dont le script a disparu sont retirés automatiquement au démarrage.

## Développement

```bash
cargo test     # tests de découverte + flux de mise à jour
cargo clippy
cargo fmt
```
