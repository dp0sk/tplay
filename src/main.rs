//! Main module for the application.
//!
//! This module contains the main function and handles command line arguments,
//! media processing, and terminal display.
//! The main function launches two threads, one for processing the media and
//! one for displaying the terminal.
//! The media processing thread is responsible for reading the media file,
//! processing it, and sending the processed frames to the terminal thread.
//! The terminal thread is responsible for displaying the terminal and
//! receiving the processed frames from the media processing thread.
//! The media processing thread and the terminal thread communicate via a
//! shared buffer.
//!
mod common;
mod downloader;
mod pipeline;
mod terminal;
use crate::common::errors::MyError;
use crate::pipeline::char_maps::CHARS1;
use crate::pipeline::image_pipeline::ImagePipeline;
use clap::Parser;
use std::sync::mpsc::{channel, sync_channel};

use pipeline::frames::{open_media, FrameIterator};

use pipeline::runner;

use pipeline::runner::Control;
use std::thread;
use terminal::Terminal;
pub type StringInfo = (String, Vec<u8>);
/// Command line arguments structure.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the file/stream to process
    #[arg(required = true, index = 1)]
    input: String,
    /// Maximum fps
    #[arg(short, long, default_value = "60")]
    fps: u64,
    /// Custom lookup char table
    #[arg(short, long, default_value = CHARS1)]
    char_map: String,
    /// Experimental width modifier (emojis have 2x width)
    #[arg(short, long, default_value = "false")]
    gray: bool,
    /// Experimental width modifier (emojis have 2x width)
    #[arg(long, default_value = "1")]
    w_mod: u32,
}

/// Main function for the application.
///
/// This function parses command line arguments, opens the media, initializes the
/// pipeline and terminal threads, and then waits for them to finish.
fn main() -> Result<(), MyError> {
    let args = Args::parse();

    let media: FrameIterator = open_media(args.input.clone())?;

    // Set up a channel for passing frames and controls
    let (tx_frames, rx_frames) = sync_channel::<Option<StringInfo>>(1);
    let (tx_controls, rx_controls) = channel::<Control>();

    // Launch Terminal Thread
    let handle_thread_terminal = thread::spawn(move || {
        let mut term = Terminal::new(args.input, args.gray, rx_frames, tx_controls);
        term.run().expect("Error running terminal thread");
    });

    // Launch Pipeline Thread
    let handle_thread_pipeline = thread::spawn(move || {
        let mut runner = runner::Runner::init(
            ImagePipeline::new((80, 24), args.char_map.chars().collect()),
            media,
            args.fps,
            tx_frames,
            rx_controls,
            args.w_mod,
        );
        runner.run().expect("Error running pipeline thread");
    });

    // Wait for threads to finish
    let _ = handle_thread_pipeline.join();
    let _ = handle_thread_terminal.join();

    Ok(())
}
