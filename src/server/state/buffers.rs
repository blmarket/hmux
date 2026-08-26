//! The server's paste buffers — tmux's global `paste_buffer` list.
//!
//! Buffers are named (`buffer0`, `buffer1`, ... unless `set-buffer -b` names
//! one) and capped by `buffer-limit`, oldest automatic buffer evicted first.

use bytes::{BufMut, Bytes, BytesMut};

use super::ServerState;

impl ServerState {
    /// `set-buffer [-b name] data`: store a paste buffer. Without a name, a fresh
    /// auto-named buffer (`buffer0`, `buffer1`, …) is pushed to the front; with a
    /// name, that buffer's data is replaced (or created).
    pub fn set_buffer(&mut self, name: Option<&str>, data: &[u8]) {
        let created = unsafe { libc::time(std::ptr::null_mut()) as i64 };
        match name {
            Some(n) => {
                self.automatic_buffers.remove(n);
                self.buffer_created.insert(n.to_string(), created);
                if let Some(entry) = self.buffers.iter_mut().find(|(bn, _)| bn == n) {
                    entry.1 = Bytes::copy_from_slice(data);
                    // tmux's `paste_set` frees the buffer it replaces before it
                    // inserts the new one, so a replacement is reported as a
                    // deletion and a change rather than as one change.
                    self.notify_paste_buffer(n, true);
                } else {
                    self.buffers
                        .insert(0, (n.to_string(), Bytes::copy_from_slice(data)));
                }
                self.notify_paste_buffer(n, false);
            }
            None => {
                let n = format!("buffer{}", self.next_buffer_id);
                self.next_buffer_id += 1;
                self.automatic_buffers.insert(n.clone());
                self.buffer_created.insert(n.clone(), created);
                self.buffers
                    .insert(0, (n.clone(), Bytes::copy_from_slice(data)));
                self.notify_paste_buffer(&n, false);
                self.enforce_buffer_limit();
            }
        }
    }

    /// `buffer-limit` bounds only automatically named buffers. Explicitly
    /// named buffers are retained, matching tmux's paste-buffer policy.
    fn enforce_buffer_limit(&mut self) {
        let limit = self
            .server_options()
            .get("buffer-limit")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(50);
        while self.automatic_buffers.len() > limit {
            let Some(position) = self
                .buffers
                .iter()
                .rposition(|(name, _)| self.automatic_buffers.contains(name))
            else {
                break;
            };
            let (name, _) = self.buffers.remove(position);
            self.automatic_buffers.remove(&name);
            self.buffer_created.remove(&name);
            self.notify_paste_buffer(&name, true);
        }
    }

    pub(crate) fn add_buffer_with_prefix(&mut self, prefix: Option<&str>, data: &[u8]) {
        match prefix {
            None => self.set_buffer(None, data),
            Some(prefix) => {
                let mut index = self.next_buffer_id;
                loop {
                    let name = format!("{prefix}{index}");
                    if self.buffers.iter().all(|(existing, _)| existing != &name) {
                        self.next_buffer_id = index + 1;
                        self.set_buffer(Some(&name), data);
                        break;
                    }
                    index += 1;
                }
            }
        }
    }

    /// Append data to a paste buffer. A missing named buffer is created; with
    /// no name, append to the newest buffer or create an auto-named buffer when
    /// none exists.
    pub fn append_buffer(&mut self, name: Option<&str>, data: &[u8]) {
        let entry = match name {
            Some(n) => self.buffers.iter_mut().find(|(bn, _)| bn == n),
            None => self.buffers.first_mut(),
        };
        if let Some((existing_name, existing)) = entry {
            let mut appended = BytesMut::with_capacity(existing.len() + data.len());
            appended.put_slice(existing);
            appended.put_slice(data);
            *existing = appended.freeze();
            // tmux appends by building the joined data and handing it back to
            // `paste_set`, so an append reports the same pair a replacement
            // does.
            let existing_name = existing_name.clone();
            self.notify_paste_buffer(&existing_name, true);
            self.notify_paste_buffer(&existing_name, false);
        } else {
            self.set_buffer(name, data);
        }
    }

    /// Rename a paste buffer. If `new_name` already exists, tmux replaces it
    /// with the renamed source buffer.
    pub fn rename_buffer(&mut self, name: &str, new_name: &str) -> bool {
        if !self
            .buffers
            .iter()
            .any(|(buffer_name, _)| buffer_name == name)
        {
            return false;
        }
        if name != new_name {
            if self
                .buffers
                .iter()
                .any(|(buffer_name, _)| buffer_name == new_name)
            {
                // The buffer the new name belonged to is freed first, which is
                // a deletion of its own before the rename is reported.
                self.notify_paste_buffer(new_name, true);
            }
            self.buffers
                .retain(|(buffer_name, _)| buffer_name != new_name);
            if let Some((buffer_name, _)) = self
                .buffers
                .iter_mut()
                .find(|(buffer_name, _)| buffer_name == name)
            {
                *buffer_name = new_name.to_string();
            }
            if let Some(created) = self.buffer_created.remove(name) {
                self.buffer_created.insert(new_name.to_string(), created);
            }
            self.notify_paste_buffer(name, true);
            self.notify_paste_buffer(new_name, false);
        }
        self.automatic_buffers.remove(name);
        self.automatic_buffers.remove(new_name);
        true
    }

    /// The named buffer's data, or the most recent automatically named buffer's
    /// data if `name` is `None`. Explicitly named buffers do not participate in
    /// tmux's unnamed show/save lookup.
    pub fn buffer(&self, name: Option<&str>) -> Option<&[u8]> {
        match name {
            Some(n) => self
                .buffers
                .iter()
                .find(|(bn, _)| bn == n)
                .map(|(_, d)| &d[..]),
            None => self
                .buffers
                .iter()
                .find(|(bn, _)| self.automatic_buffers.contains(bn))
                .map(|(_, d)| &d[..]),
        }
    }

    /// Iterate paste buffers, newest first (tmux's `list-buffers` order).
    pub fn buffers(&self) -> &[(String, Bytes)] {
        &self.buffers
    }

    pub(crate) fn automatic_buffer_name(&self) -> Option<&str> {
        self.buffers
            .iter()
            .find(|(name, _)| self.automatic_buffers.contains(name))
            .map(|(name, _)| name.as_str())
    }

    pub(crate) fn buffer_created(&self, name: &str) -> Option<i64> {
        self.buffer_created.get(name).copied()
    }

    /// Delete a paste buffer by name. Returns whether it existed.
    pub fn delete_buffer(&mut self, name: &str) -> bool {
        let before = self.buffers.len();
        self.buffers.retain(|(n, _)| n != name);
        self.automatic_buffers.remove(name);
        self.buffer_created.remove(name);
        let removed = self.buffers.len() != before;
        if removed {
            self.notify_paste_buffer(name, true);
        }
        removed
    }
}
