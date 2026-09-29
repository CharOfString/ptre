// Copyright (C) 2026 CharOfString <root@charofstring.cc>
//
//
// This software is free software: you can redistribute it and/or modify it under the terms of the
// GNU General Public License as published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This software is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
// without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with this software. If
// not, see <https://www.gnu.org/licenses/>.

use super::super::{diagnostics::Severity, plugin::parse};
use super::*;
use std::{fs, time::Instant};

fn wait_for<T>(client: &mut Client, mut done: impl FnMut(&mut Client) -> Option<T>) -> T {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if let Some(value) = done(client) {
            return value;
        }
        assert!(client.is_alive(), "server exited");
        assert!(Instant::now() < deadline, "server timed out");
        thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn missing_server_fails_to_start() {
    let plugin = parse("c", "command = ptre-no-such-server").unwrap();
    assert!(Client::start(&plugin, Path::new("main.c")).is_err());
}

#[test]
fn clangd_completes_after_changes() {
    let plugin = parse("c", include_str!("../../../plugins/lsp/c.conf")).unwrap();
    let dir = std::env::temp_dir().join(format!("ptre-lsp-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("main.c");
    let before = "static int value_one;\nint main(void) { }\n";
    fs::write(&file, before).unwrap();
    let Ok(mut client) = Client::start(&plugin, &file) else {
        eprintln!("clangd not installed; skipping");
        return fs::remove_dir_all(&dir).unwrap();
    };
    client.open(&file, "c", before).unwrap();
    wait_for(&mut client, |client| {
        client.poll();
        client.is_ready().then_some(())
    });

    // The request must see the edited text, not the text from didOpen.
    let after = "static int value_one;\nint main(void) { valu }\n";
    client.change(after);
    // `rfind`: the first "valu" is inside `value_one` on line 1.
    let cursor = after.rfind("valu").unwrap() + 4;
    assert!(client.is_trigger('.') && !client.is_trigger('v'));
    assert!(client.complete(cursor - 4..cursor, None));
    let completions = wait_for(&mut client, Client::poll);
    let value = completions
        .candidates
        .iter()
        .find(|candidate| candidate.text == "value_one")
        .expect("value_one offered");
    assert_eq!(value.range, cursor - 4..cursor);
    assert_eq!(client.command(), ["clangd"]);

    // clangd reports the undeclared name against the edited version.
    let mut latest = Vec::new();
    let error = wait_for(&mut client, |client| {
        client.poll();
        if let Some(diagnostics) = client.take_diagnostics() {
            latest = diagnostics;
        }
        let fresh = client.diagnostics_fresh();
        let error = latest.iter().find(|d| d.severity == Severity::Error);
        error.filter(|_| fresh).cloned()
    });
    assert_eq!(error.range, cursor - 4..cursor);
    assert!(error.message.contains("valu"), "{}", error.message);

    // A close misspelling gets the declared name as a quick fix.
    let typo = "static int value_one;\nint main(void) { return value_on; }\n";
    client.change(typo);
    let start = typo.find("value_on;").unwrap();
    wait_for(&mut client, |client| {
        client.poll();
        client.take_diagnostics();
        client.diagnostics_fresh().then_some(())
    });
    assert!(client.request_fixes(start + 3));
    let answer = wait_for(&mut client, |client| {
        client.poll();
        client.take_fixes()
    });
    assert_eq!(answer.text, typo);
    let expected = [(start..start + 8, "value_one".to_owned())];
    let fix = answer
        .fixes
        .iter()
        .find(|fix| fix.edits == expected)
        .unwrap_or_else(|| panic!("{:#?}", answer.fixes));
    assert!(fix.title.contains("value_one"), "{}", fix.title);
    assert!(
        !client.request_fixes(0),
        "nothing to fix at the start of the file"
    );
    drop(client);
    fs::remove_dir_all(&dir).unwrap();
}
