# lazyinstall — notes d'architecture

TUI Rust (ratatui + crossterm) pour suivre des dossiers d'installation et lancer
leurs scripts de mise à jour. Même famille et mêmes conventions que `lazyvpn`,
`lazycomposer`, `lazykanban` (build.rs avec métadonnées de build, `App` portant
la boucle d'événements, persistance JSON sous `~/.config/`).

## Modèle de domaine (`src/install/`)

La séparation suit le principe « l'objet de domaine dit *ce que c'est*, le wrapper
dit *ce qu'on en fait ici et maintenant* » :

- **`target::InstallTarget`** — objet de domaine **immuable** : un dossier + son
  script + un nom. Il **sait se mettre à jour lui-même** (`update()`), car il
  détient son dossier de travail et son script. `update()` lance le script dans un
  **pseudo-terminal** (`portable-pty`, `LC_ALL=C`) pour que tout `sudo` du script
  voie un vrai TTY ; il détecte l'invite de mot de passe
  (`looks_like_password_prompt`, fonction pure testée seule) et émet
  `UpdateMessage::PasswordPrompt`. Renvoie un `RunningUpdate`, **poignée** vers le
  process vivant : lire sa sortie (`rx`), lui transmettre un mot de passe
  (`provide_password`), l'interrompre (`cancel`). PTY et `Child` restent cachés.
- **`discover::discover()`** — la **factory** qui fait naître les cibles d'un
  dossier (canonicalise le chemin, **une cible par `update-*.sh`**, sinon repli sur
  le premier `*.sh` ; dérive le nom). Renvoie un `Vec<InstallTarget>`. Une cible ne
  fouille pas son propre dossier : elle naît déjà équipée de son script.
- **`tracked::TrackedTarget`** — le **wrapper runtime** (équivalent de `VpnEntry`
  dans lazyvpn) : la cible + l'état volatile `UpdateState` + le flux + les logs +
  l'attente éventuelle d'un mot de passe. Il sait `launch()` (se lancer), `pump()`
  (faire avancer son propre flux sans bloquer), et relaie `provide_password()` /
  `cancel()` à sa poignée.
- **`store`** — charge/sauve la liste des dossiers dans `targets.json`.

## UI (`src/ui/`)

`app::App` tient `Vec<TrackedTarget>`, le curseur, l'overlay d'ajout, le toast,
et la **politique de mot de passe sudo** : un cache de session (`sudo_password`,
demandé une fois puis réutilisé, effacé en fin de `run()`) et l'overlay de saisie
masquée (`password_prompt`, contextualisé par cible). Le domaine ignore tout du mot
de passe. À chaque tour : `pump()` toutes les cibles, traite les demandes de mot de
passe, dessine, lit une touche. Panneaux dans `ui/panels/` (liste, sortie,
statusbar, aide, prompt d'ajout, prompt de mot de passe). Clavier dans `ui/keys.rs`.

## Conventions

- Domaine = code métier : faire valider la conception OO par le skill `dev-gourou`.
- Pas de `null`/`unwrap` dans le domaine ; erreurs via `anyhow::Result`.
- Tests : `tests/discover.rs` (découverte) et `tests/update.rs` (flux complet
  lancement → streaming → état, sans réseau ni TUI).
