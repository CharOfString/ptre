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

use serde_json::Value;
use std::io::{self, BufRead, Write};

// Write one JSON-RPC message with the LSP `Content-Length` framing.
pub(crate) fn write_message(writer: &mut impl Write, message: &Value) -> io::Result<()> {
    let body = serde_json::to_vec(message)?;
    write!(writer, "Content-Length: {}\r\n\r\n", body.len())?;
    writer.write_all(&body)?;
    writer.flush()
}

// Read one framed message. `None` means the server closed its output between messages.
pub(crate) fn read_message(reader: &mut impl BufRead) -> io::Result<Option<Value>> {
    let mut length = None;
    let mut line = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            return match length {
                None => Ok(None),
                Some(_) => Err(io::ErrorKind::UnexpectedEof.into()),
            };
        }
        let header = line.trim_end();
        if header.is_empty() {
            break;
        }
        // Other headers (such as `Content-Type`) carry nothing we need.
        if let Some((name, value)) = header.split_once(':')
            && name.eq_ignore_ascii_case("content-length")
        {
            let value = value
                .trim()
                .parse()
                .map_err(|_| invalid("bad Content-Length"))?;
            length = Some(value);
        }
    }

    let length = length.ok_or_else(|| invalid("missing Content-Length"))?;
    let mut body = vec![0; length];
    reader.read_exact(&mut body)?;
    Ok(Some(serde_json::from_slice(&body)?))
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn messages_round_trip_through_framing() {
        let mut stream = Vec::new();
        write_message(&mut stream, &json!({"id": 1, "text": "ünïcode"})).unwrap();
        write_message(&mut stream, &json!({"id": 2})).unwrap();
        let mut reader = stream.as_slice();
        assert_eq!(
            read_message(&mut reader).unwrap(),
            Some(json!({"id": 1, "text": "ünïcode"}))
        );
        assert_eq!(read_message(&mut reader).unwrap(), Some(json!({"id": 2})));
        assert_eq!(read_message(&mut reader).unwrap(), None);
    }

    #[test]
    fn extra_headers_are_ignored_and_bad_frames_are_errors() {
        let frame = b"Content-Type: application/vscode-jsonrpc\r\ncontent-length: 2\r\n\r\n{}";
        assert_eq!(read_message(&mut &frame[..]).unwrap(), Some(json!({})));
        assert!(read_message(&mut &b"Content-Type: x\r\n\r\n{}"[..]).is_err());
        assert!(read_message(&mut &b"Content-Length: 10\r\n\r\n{}"[..]).is_err());
        assert!(read_message(&mut &b"Content-Length: 5\r\n"[..]).is_err());
    }
}
