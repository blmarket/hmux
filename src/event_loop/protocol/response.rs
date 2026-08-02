use super::*;

pub(super) struct CommandResponse {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    exit: i32,
    phase: ResponsePhase,
}

enum ResponsePhase {
    OpenStdout,
    WaitStdout,
    WriteStdout(usize),
    CloseStdout,
    OpenStderr,
    WaitStderr,
    WriteStderr(usize),
    CloseStderr,
    Exit,
    Complete,
}

impl CommandResponse {
    pub(super) fn new(result: CommandResult) -> Self {
        Self {
            stdout: result.stdout_data().to_vec(),
            stderr: result.stderr.into_bytes(),
            exit: result.exit,
            phase: ResponsePhase::OpenStdout,
        }
    }

    pub(super) fn acknowledge(&mut self, stream: i32, error: i32) {
        self.phase = match (&self.phase, stream) {
            (ResponsePhase::WaitStdout, 1) if error == 0 => ResponsePhase::WriteStdout(0),
            (ResponsePhase::WaitStdout, 1) => ResponsePhase::CloseStdout,
            (ResponsePhase::WaitStderr, 2) if error == 0 => ResponsePhase::WriteStderr(0),
            (ResponsePhase::WaitStderr, 2) => ResponsePhase::CloseStderr,
            _ => return,
        };
    }

    pub(super) fn next_frame(&mut self) -> Option<Frame> {
        loop {
            match &mut self.phase {
                ResponsePhase::OpenStdout if self.stdout.is_empty() => {
                    self.phase = ResponsePhase::OpenStderr;
                }
                ResponsePhase::OpenStdout => {
                    self.phase = ResponsePhase::WaitStdout;
                    return Some(Frame::new(Message::WriteOpen {
                        stream: 1,
                        fd: 1,
                        flags: 0,
                        path: Vec::new(),
                    }));
                }
                ResponsePhase::WaitStdout | ResponsePhase::WaitStderr => return None,
                ResponsePhase::WriteStdout(offset) if *offset < self.stdout.len() => {
                    let end = (*offset + OUTPUT_CHUNK).min(self.stdout.len());
                    let frame = Frame::new(Message::Write {
                        stream: 1,
                        data: self.stdout[*offset..end].to_vec(),
                    });
                    *offset = end;
                    return Some(frame);
                }
                ResponsePhase::WriteStdout(_) => {
                    self.phase = ResponsePhase::CloseStdout;
                }
                ResponsePhase::CloseStdout => {
                    self.phase = ResponsePhase::OpenStderr;
                    return Some(Frame::new(Message::WriteClose { stream: 1 }));
                }
                ResponsePhase::OpenStderr if self.stderr.is_empty() => {
                    self.phase = ResponsePhase::Exit;
                }
                ResponsePhase::OpenStderr => {
                    self.phase = ResponsePhase::WaitStderr;
                    return Some(Frame::new(Message::WriteOpen {
                        stream: 2,
                        fd: 2,
                        flags: 0,
                        path: Vec::new(),
                    }));
                }
                ResponsePhase::WriteStderr(offset) if *offset < self.stderr.len() => {
                    let end = (*offset + OUTPUT_CHUNK).min(self.stderr.len());
                    let frame = Frame::new(Message::Write {
                        stream: 2,
                        data: self.stderr[*offset..end].to_vec(),
                    });
                    *offset = end;
                    return Some(frame);
                }
                ResponsePhase::WriteStderr(_) => {
                    self.phase = ResponsePhase::CloseStderr;
                }
                ResponsePhase::CloseStderr => {
                    self.phase = ResponsePhase::Exit;
                    return Some(Frame::new(Message::WriteClose { stream: 2 }));
                }
                ResponsePhase::Exit => {
                    self.phase = ResponsePhase::Complete;
                    return Some(Frame::new(Message::Exit(Some(self.exit))));
                }
                ResponsePhase::Complete => return None,
            }
        }
    }

    pub(super) fn is_complete(&self) -> bool {
        matches!(self.phase, ResponsePhase::Complete)
    }
}
