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

use super::{
    diagnostics::{self, Diagnostic},
    normalize::{Completions, normalize},
    plugin::Plugin,
    position::{self, Encoding},
    transport, workspace,
};
use serde_json::{Map, Value, json};
use std::{
    io::{self, BufReader},
    ops::Range,
    path::Path,
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
    time::Duration,
};

// The document the server knows about. Every change resends the full text.
struct Document {
    uri: String,
    language_id: &'static str,
    version: i32,
    text: String,
}

// A completion request still waiting for its response.
struct Pending {
    id: i64,
    text: String,
    word: Range<usize>,
}

// One running language server. Messages are read on a background thread and handled on the
// UI thread by `poll`, so the editor never blocks waiting for the server.
pub(crate) struct Client {
    command: Vec<String>,
    child: Child,
    stdin: ChildStdin,
    inbox: Receiver<Value>,
    encoding: Encoding,
    // Characters that make the server offer completions by themselves, such as '.'.
    triggers: Vec<char>,
    ready: bool,
    alive: bool,
    next_id: i64,
    document: Option<Document>,
    pending: Option<Pending>,
    // Diagnostics received since the last `take_diagnostics`, and the document version
    // the latest ones describe.
    diagnostics: Option<Vec<Diagnostic>>,
    diagnostics_version: Option<i32>,
}

impl Client {
    // Launch the plugin's server for the project containing `file` and begin the handshake.
    pub(crate) fn start(plugin: &Plugin, file: &Path) -> io::Result<Self> {
        let root = workspace::find_root(file)?;
        let root_uri = workspace::file_uri(&root)?;
        let (program, args) = plugin
            .command
            .split_first()
            .expect("plugin commands are never empty");
        let mut child = Command::new(program)
            .args(args)
            .current_dir(&root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let stdin = child.stdin.take().expect("stdin is piped");
        let stdout = child.stdout.take().expect("stdout is piped");

        // The channel disconnects once the server closes its output.
        let (sender, inbox) = mpsc::channel();
        thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            while let Ok(Some(message)) = transport::read_message(&mut reader) {
                if sender.send(message).is_err() {
                    break;
                }
            }
        });

        let mut client = Self {
            command: plugin.command.clone(),
            child,
            stdin,
            inbox,
            encoding: Encoding::Utf16,
            triggers: Vec::new(),
            ready: false,
            alive: true,
            next_id: 0,
            document: None,
            pending: None,
            diagnostics: None,
            diagnostics_version: None,
        };
        let name = root
            .file_name()
            .map_or_else(|| "/".into(), |name| name.to_string_lossy());
        client.request(
            "initialize",
            json!({
                "processId": std::process::id(),
                "clientInfo": {"name": "ptre", "version": env!("CARGO_PKG_VERSION")},
                "rootUri": root_uri,
                "workspaceFolders": [{"uri": root_uri, "name": name}],
                "capabilities": {
                    "general": {"positionEncodings": Encoding::OFFERED},
                    // clangd's pre-3.17 spelling of the same negotiation.
                    "offsetEncoding": Encoding::OFFERED,
                    "textDocument": {
                        "synchronization": {"dynamicRegistration": false},
                        "completion": {
                            "completionItem": {
                                "snippetSupport": false,
                                "insertReplaceSupport": true,
                            },
                            "contextSupport": true,
                        },
                        "publishDiagnostics": {"versionSupport": true},
                    },
                },
            }),
        );
        Ok(client)
    }

    // The command this server was started with.
    pub(crate) fn command(&self) -> &[String] {
        &self.command
    }

    // False once the server exited or a write to it failed.
    pub(crate) fn is_alive(&self) -> bool {
        self.alive
    }

    // True once the handshake finished and requests can be sent.
    pub(crate) fn is_ready(&self) -> bool {
        self.alive && self.ready
    }

    // Whether typing `c` should open completion even outside a word.
    pub(crate) fn is_trigger(&self, c: char) -> bool {
        self.is_ready() && self.triggers.contains(&c)
    }

    // Make `path` the open document, closing the previous one. `language_id` is the LSP name
    // of its language.
    pub(crate) fn open(
        &mut self,
        path: &Path,
        language_id: &'static str,
        text: &str,
    ) -> io::Result<()> {
        let uri = workspace::file_uri(path)?;
        if self.ready
            && let Some(old) = self.document.take()
        {
            self.notify(
                "textDocument/didClose",
                json!({"textDocument": {"uri": old.uri}}),
            );
        }
        self.document = Some(Document {
            uri,
            language_id,
            version: 0,
            text: text.to_owned(),
        });
        // Diagnostics of the previous document no longer apply.
        self.diagnostics = Some(Vec::new());
        self.diagnostics_version = None;
        if self.ready {
            self.send_open();
        }
        Ok(())
    }

    // Send the latest text; does nothing when it did not change.
    pub(crate) fn change(&mut self, text: &str) {
        let Some(document) = &mut self.document else {
            return;
        };
        if document.text == text {
            return;
        }
        document.version += 1;
        document.text = text.to_owned();
        let params = json!({
            "textDocument": {"uri": document.uri, "version": document.version},
            "contentChanges": [{"text": text}],
        });
        if self.ready {
            self.notify("textDocument/didChange", params);
        }
    }

    // Ask for completions at `word.end`. `word` is replaced by items without their own edit.
    // `trigger` is the trigger character just typed, if that caused the request. Any earlier
    // unanswered request is superseded.
    pub(crate) fn complete(&mut self, word: Range<usize>, trigger: Option<char>) -> bool {
        if !self.is_ready() {
            return false;
        }
        let Some(document) = &self.document else {
            return false;
        };
        // Servers use the trigger to drop false alarms, such as '>' outside of "->".
        let context = match trigger {
            Some(c) => json!({"triggerKind": 2, "triggerCharacter": c.to_string()}),
            None => json!({"triggerKind": 1}),
        };
        let params = json!({
            "textDocument": {"uri": document.uri},
            "position": position::to_position(&document.text, word.end, self.encoding),
            "context": context,
        });
        let text = document.text.clone();
        let id = self.request("textDocument/completion", params);
        self.pending = Some(Pending { id, text, word });
        true
    }

    // Handle everything the server sent so far. Returns the answer to the latest completion
    // request once it arrives.
    pub(crate) fn poll(&mut self) -> Option<Completions> {
        let mut answer = None;
        loop {
            match self.inbox.try_recv() {
                Ok(message) => answer = self.handle(message).or(answer),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.alive = false;
                    break;
                }
            }
        }
        answer
    }

    // Diagnostics that arrived since the last call, if any.
    pub(crate) fn take_diagnostics(&mut self) -> Option<Vec<Diagnostic>> {
        self.diagnostics.take()
    }

    // True when the latest diagnostics describe the text as it is now, not an older version.
    pub(crate) fn diagnostics_fresh(&self) -> bool {
        let version = self.document.as_ref().map(|document| document.version);
        version.is_some() && version == self.diagnostics_version
    }

    fn handle(&mut self, message: Value) -> Option<Completions> {
        let Some(id) = message.get("id") else {
            // Notifications need no answer; only diagnostics are used so far.
            if message["method"] == "textDocument/publishDiagnostics" {
                self.receive_diagnostics(&message["params"]);
            }
            return None;
        };
        if let Some(method) = message.get("method").and_then(Value::as_str) {
            // A request from the server. It must be answered, or some servers stall.
            self.reply(id.clone(), method, &message["params"]);
            return None;
        }
        // Nothing else is sent before the handshake, so this answers `initialize`.
        if !self.ready {
            self.initialized(&message);
            return None;
        }
        let pending = self
            .pending
            .take_if(|pending| id.as_i64() == Some(pending.id))?;
        let response = message
            .get("result")
            .and_then(|result| serde_json::from_value(result.clone()).ok())
            .flatten();
        Some(normalize(
            response,
            &pending.text,
            pending.word,
            self.encoding,
        ))
    }

    fn receive_diagnostics(&mut self, params: &Value) {
        let Some(document) = &self.document else {
            return;
        };
        let uri = params["uri"].as_str().unwrap_or_default();
        if !workspace::same_uri(uri, &document.uri) {
            return;
        }
        // Servers without `version` describe the text they have seen last.
        let version = params["version"]
            .as_i64()
            .map_or(document.version, |v| v as i32);
        self.diagnostics = Some(diagnostics::parse(params, &document.text, self.encoding));
        self.diagnostics_version = Some(version);
    }

    fn initialized(&mut self, message: &Value) {
        let Some(result) = message.get("result") else {
            self.alive = false;
            return;
        };
        let encoding = result["capabilities"]["positionEncoding"]
            .as_str()
            .or(result["offsetEncoding"].as_str());
        self.encoding = encoding
            .and_then(Encoding::from_name)
            .unwrap_or(Encoding::Utf16);
        let triggers = result["capabilities"]["completionProvider"]["triggerCharacters"].as_array();
        self.triggers = triggers
            .into_iter()
            .flatten()
            .filter_map(|trigger| trigger.as_str()?.chars().next())
            .collect();
        self.ready = true;
        self.notify("initialized", json!({}));
        self.send_open();
    }

    fn reply(&mut self, id: Value, method: &str, params: &Value) {
        let result = match method {
            // One (default) setting per requested item.
            "workspace/configuration" => {
                let count = params["items"].as_array().map_or(0, Vec::len);
                Value::Array(vec![Value::Null; count])
            }
            "workspace/applyEdit" => json!({"applied": false}),
            "client/registerCapability"
            | "client/unregisterCapability"
            | "window/workDoneProgress/create"
            | "window/showMessageRequest" => Value::Null,
            _ => {
                let error = json!({"code": -32601, "message": format!("{method} unsupported")});
                self.send(json!({"jsonrpc": "2.0", "id": id, "error": error}));
                return;
            }
        };
        self.send(json!({"jsonrpc": "2.0", "id": id, "result": result}));
    }

    fn send_open(&mut self) {
        let Some(document) = &self.document else {
            return;
        };
        let params = json!({"textDocument": {
            "uri": document.uri,
            "languageId": document.language_id,
            "version": document.version,
            "text": document.text,
        }});
        self.notify("textDocument/didOpen", params);
    }

    fn request(&mut self, method: &str, params: Value) -> i64 {
        self.next_id += 1;
        let id = self.next_id;
        self.send(message(Some(id), method, params));
        id
    }

    fn notify(&mut self, method: &str, params: Value) {
        self.send(message(None, method, params));
    }

    fn send(&mut self, message: Value) {
        if self.alive && transport::write_message(&mut self.stdin, &message).is_err() {
            self.alive = false;
        }
    }
}

// Build a request or notification; `params` is left out entirely when null.
fn message(id: Option<i64>, method: &str, params: Value) -> Value {
    let mut message = Map::new();
    message.insert("jsonrpc".into(), "2.0".into());
    if let Some(id) = id {
        message.insert("id".into(), id.into());
    }
    message.insert("method".into(), method.into());
    if !params.is_null() {
        message.insert("params".into(), params);
    }
    Value::Object(message)
}

// Ask the server to exit, then make sure it does.
impl Drop for Client {
    fn drop(&mut self) {
        self.request("shutdown", Value::Null);
        self.notify("exit", Value::Null);
        for _ in 0..20 {
            if !matches!(self.child.try_wait(), Ok(None)) {
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(test)]
mod tests {
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
        let plugin = parse("c", include_str!("../../plugins/lsp/c.conf")).unwrap();
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
        drop(client);
        fs::remove_dir_all(&dir).unwrap();
    }
}
