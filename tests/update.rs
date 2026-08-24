use std::fs;
use std::time::{Duration, Instant};

use lazyinstall::install::discover;
use lazyinstall::install::tracked::{TrackedTarget, UpdateState};
use tempfile::TempDir;

/// Boucle de pompage jusqu'à ce que la mise à jour quitte l'état `Updating`,
/// avec un garde-fou temporel pour ne jamais bloquer la suite de tests.
fn pump_until_done(tracked: &mut TrackedTarget) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while tracked.is_updating() && Instant::now() < deadline {
        tracked.pump();
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn it_should_stream_output_and_succeed_on_a_zero_exit_script() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("update-demo.sh"),
        "#!/usr/bin/env bash\necho 'première ligne'\necho 'mis à jour'\nexit 0\n",
    )
    .unwrap();

    let target = discover::discover(tmp.path())
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    let mut tracked = TrackedTarget::new(target);

    tracked.launch().unwrap();
    pump_until_done(&mut tracked);

    assert_eq!(tracked.state(), &UpdateState::Succeeded);
    let joined = tracked
        .logs()
        .iter()
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");
    assert!(joined.contains("première ligne"), "logs = {joined:?}");
    assert!(joined.contains("mis à jour"), "logs = {joined:?}");
}

#[test]
fn it_should_mark_the_target_as_failed_on_a_non_zero_exit_script() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("update-demo.sh"),
        "#!/usr/bin/env bash\necho 'boom' >&2\nexit 3\n",
    )
    .unwrap();

    let target = discover::discover(tmp.path())
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    let mut tracked = TrackedTarget::new(target);

    tracked.launch().unwrap();
    pump_until_done(&mut tracked);

    assert!(matches!(tracked.state(), UpdateState::Failed(_)));
    // stderr est fusionné dans la sortie.
    let joined = tracked
        .logs()
        .iter()
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");
    assert!(joined.contains("boom"), "logs = {joined:?}");
}

#[test]
fn it_should_detect_a_password_prompt_and_inject_the_answer() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("update-secret.sh"),
        "#!/usr/bin/env bash\nread -s -p 'Password: ' pw\necho\necho \"recu:$pw\"\nexit 0\n",
    )
    .unwrap();

    let target = discover::discover(tmp.path())
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    let mut tracked = TrackedTarget::new(target);
    tracked.launch().unwrap();

    // On attend que le script réclame le mot de passe (le PTY donne un vrai TTY).
    let deadline = Instant::now() + Duration::from_secs(10);
    while !tracked.is_awaiting_password() && tracked.is_updating() && Instant::now() < deadline {
        tracked.pump();
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(tracked.is_awaiting_password(), "aucune invite détectée");

    tracked.provide_password("secret".to_string());
    pump_until_done(&mut tracked);

    assert_eq!(tracked.state(), &UpdateState::Succeeded);
    let joined = tracked
        .logs()
        .iter()
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");
    assert!(joined.contains("recu:secret"), "logs = {joined:?}");
}
