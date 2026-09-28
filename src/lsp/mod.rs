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

// A minimal Language Server Protocol client: enough to open one document, keep it in sync and
// ask for completions. Which server runs for which language comes from plugin files (see
// `plugin`); server differences are absorbed in `normalize`.

mod client;
mod diagnostics;
mod normalize;
mod plugin;
mod position;
mod transport;
mod workspace;

pub(crate) use client::Client;
pub(crate) use diagnostics::{Diagnostic, Severity};
pub(crate) use normalize::{Candidate, Completions};
pub(crate) use plugin::{Plugin, language_id, load as load_plugins};
