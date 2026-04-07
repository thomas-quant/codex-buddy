use std::{
    collections::BTreeMap,
    io::{ErrorKind, Read, Write},
    sync::mpsc::{self, Receiver},
    thread,
};

use anyhow::Result;
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use vt100::Parser;

pub struct PtyHost {
    parser: Parser,
    writer: Box<dyn Write + Send>,
    child: Box<dyn portable_pty::Child + Send + Sync>,
    master: Box<dyn portable_pty::MasterPty + Send>,
    output_rx: Receiver<Vec<u8>>,
    _reader_thread: thread::JoinHandle<()>,
}

impl PtyHost {
    pub fn spawn(
        command: &str,
        args: &[String],
        env: &BTreeMap<String, String>,
        cols: u16,
        rows: u16,
    ) -> Result<Self> {
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut builder = CommandBuilder::new(command);
        builder.args(args.iter().map(String::as_str));
        for (key, value) in env {
            builder.env(key, value);
        }

        Self::spawn_from_builder(pair, builder, cols, rows)
    }

    pub fn spawn_for_test(command: &str, args: &[&str], cols: u16, rows: u16) -> Result<Self> {
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut builder = CommandBuilder::new(command);
        builder.args(args.iter().copied());

        Self::spawn_from_builder(pair, builder, cols, rows)
    }

    fn spawn_from_builder(
        pair: portable_pty::PtyPair,
        builder: CommandBuilder,
        cols: u16,
        rows: u16,
    ) -> Result<Self> {
        let portable_pty::PtyPair { slave, master } = pair;

        let child = slave.spawn_command(builder)?;
        let reader = master.try_clone_reader()?;
        let writer = master.take_writer()?;
        let (output_tx, output_rx) = mpsc::channel();

        let reader_thread = thread::spawn(move || {
            let mut reader = reader;
            let mut buffer = [0_u8; 4096];

            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(read) => {
                        if output_tx.send(buffer[..read].to_vec()).is_err() {
                            break;
                        }
                    }
                    Err(err) if err.kind() == ErrorKind::Interrupted => continue,
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            parser: Parser::new(rows, cols, 0),
            writer,
            child,
            master,
            output_rx,
            _reader_thread: reader_thread,
        })
    }

    pub fn pump_output(&mut self) -> Result<()> {
        while let Ok(chunk) = self.output_rx.try_recv() {
            self.parser.process(&chunk);
        }

        Ok(())
    }

    pub fn write_all(&mut self, bytes: &[u8]) -> Result<()> {
        self.writer.write_all(bytes)?;
        self.writer.flush()?;
        Ok(())
    }

    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()> {
        self.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        self.parser.set_size(rows, cols);
        Ok(())
    }

    pub fn try_wait(&mut self) -> Result<Option<portable_pty::ExitStatus>> {
        Ok(self.child.try_wait()?)
    }

    pub fn screen_text(&self) -> String {
        self.parser.screen().contents()
    }
}
