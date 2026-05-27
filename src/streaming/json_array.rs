//! Shared streaming support for JSON exports with a top-level messages array.

use std::collections::VecDeque;
use std::io::BufRead;

use super::{StreamingError, StreamingResult};

const HEADER_SEARCH_LIMIT: u64 = 10 * 1024 * 1024;

pub(crate) struct JsonArrayObjectReader<R: BufRead> {
    reader: R,
    bytes_read: u64,
    pending: VecDeque<char>,
    buffer: String,
    max_object_size: usize,
    finished: bool,
}

impl<R: BufRead> JsonArrayObjectReader<R> {
    pub(crate) fn new(
        reader: R,
        array_name: &str,
        buffer_size: usize,
        max_object_size: usize,
    ) -> StreamingResult<Self> {
        let mut object_reader = Self {
            reader,
            bytes_read: 0,
            pending: VecDeque::new(),
            buffer: String::with_capacity(max_object_size.min(buffer_size)),
            max_object_size,
            finished: false,
        };

        object_reader.seek_to_array(array_name, buffer_size)?;
        Ok(object_reader)
    }

    pub(crate) fn next_object(&mut self) -> StreamingResult<Option<String>> {
        if self.finished {
            return Ok(None);
        }

        self.buffer.clear();
        let mut found_start = false;
        let mut brace_depth = 0i32;
        let mut in_string = false;
        let mut escaped = false;

        loop {
            let Some(ch) = self.next_char()? else {
                self.finished = true;
                return if found_start {
                    Err(StreamingError::UnexpectedEof)
                } else {
                    Ok(None)
                };
            };

            if found_start {
                self.buffer.push(ch);
                update_json_object_state(ch, &mut in_string, &mut escaped, &mut brace_depth);

                if self.buffer.len() > self.max_object_size {
                    return Err(StreamingError::BufferOverflow {
                        max_size: self.max_object_size,
                        actual_size: self.buffer.len(),
                    });
                }

                if brace_depth == 0 {
                    return Ok(Some(self.buffer.clone()));
                }
            } else {
                match ch {
                    c if c.is_whitespace() || c == ',' => {}
                    ']' => {
                        self.finished = true;
                        return Ok(None);
                    }
                    '{' => {
                        found_start = true;
                        brace_depth = 1;
                        self.buffer.push(ch);
                    }
                    _ => {
                        return Err(StreamingError::InvalidFormat(format!(
                            "Unexpected character in messages array: {ch:?}"
                        )));
                    }
                }
            }
        }
    }

    pub(crate) fn bytes_read(&self) -> u64 {
        self.bytes_read
    }

    fn seek_to_array(&mut self, array_name: &str, buffer_size: usize) -> StreamingResult<()> {
        let key = format!("\"{array_name}\"");
        let mut found_key = false;
        let mut line = String::with_capacity(buffer_size);

        loop {
            line.clear();
            let bytes = self.reader.read_line(&mut line)?;
            if bytes == 0 {
                return Err(StreamingError::InvalidFormat(format!(
                    "Could not find '{array_name}' array in file"
                )));
            }

            self.bytes_read += bytes as u64;
            let search_start = if found_key {
                0
            } else if let Some(key_pos) = line.find(&key) {
                found_key = true;
                key_pos + key.len()
            } else {
                self.check_header_limit(array_name)?;
                continue;
            };

            if let Some(array_pos) = line[search_start..].find('[') {
                let array_start = search_start + array_pos + 1;
                self.pending.extend(line[array_start..].chars());
                return Ok(());
            }

            self.check_header_limit(array_name)?;
        }
    }

    fn check_header_limit(&self, array_name: &str) -> StreamingResult<()> {
        if self.bytes_read > HEADER_SEARCH_LIMIT {
            return Err(StreamingError::InvalidFormat(format!(
                "File header too large or '{array_name}' array not found"
            )));
        }

        Ok(())
    }

    fn next_char(&mut self) -> StreamingResult<Option<char>> {
        if let Some(ch) = self.pending.pop_front() {
            return Ok(Some(ch));
        }

        let mut line = String::new();
        let bytes = self.reader.read_line(&mut line)?;
        if bytes == 0 {
            return Ok(None);
        }

        self.bytes_read += bytes as u64;
        self.pending.extend(line.chars());
        Ok(self.pending.pop_front())
    }
}

fn update_json_object_state(
    ch: char,
    in_string: &mut bool,
    escaped: &mut bool,
    brace_depth: &mut i32,
) {
    if *in_string {
        if *escaped {
            *escaped = false;
        } else if ch == '\\' {
            *escaped = true;
        } else if ch == '"' {
            *in_string = false;
        }
    } else {
        match ch {
            '"' => *in_string = true,
            '{' => *brace_depth += 1,
            '}' => *brace_depth -= 1,
            _ => {}
        }
    }
}
