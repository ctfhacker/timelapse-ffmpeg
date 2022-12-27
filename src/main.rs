use clap::Parser;
use tempfile::NamedTempFile;

use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Parser, Debug)]
struct Args {
    /// Length of the final timelapse (in seconds)
    #[arg(short, long)]
    length: f32,

    /// Path to the input video
    #[arg(short, long)]
    input: PathBuf,

    /// Path to write the output video
    #[arg(short, long)]
    output: PathBuf,
}

/// Ensures `ffmpeg` is available
///
/// # Panics
///
/// * `ffmpeg` not found
fn check_for_ffmpeg() {
    assert!(
        Command::new("ffmpeg")
            .arg("-h")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .is_ok(),
        "'ffmpeg' binary not found. Please install."
    );
    assert!(
        Command::new("ffprobe")
            .arg("-h")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .is_ok(),
        "'ffprobe' binary not found. Please install."
    );
}

/// Get the length of `input` via `ffprobe`
fn get_duration(input: &PathBuf) -> f32 {
    let stdout = Command::new("ffprobe")
        .args([
            "-i",
            input.as_path().to_str().unwrap(),
            "-show_entries",
            "format=duration",
            "-v",
            "quiet",
            "-print_format",
            "csv=p=0",
        ])
        .output();

    String::from_utf8_lossy(
        &stdout
            .unwrap()
            .stdout
            .split(|x| *x == b'\n')
            .next()
            .unwrap(),
    )
    .parse::<f32>()
    .unwrap()
}

fn main() -> Result<(), std::io::Error> {
    // Sanity check `ffmpeg` is installed
    check_for_ffmpeg();

    let args = Args::parse();

    // Prepare the temporary output file
    let tmp_out = NamedTempFile::new()?;
    let tmp_out_path = tmp_out.path();

    // Rename the tmpfile to use `.mp4` to allow `ffmpeg` to output the right format
    let tmp_out_name = tmp_out_path.with_extension("mp4");

    // Get the duration of the input video
    let duration = get_duration(&args.input);

    let final_length = args.length;

    // Calculate the PTS factor based on the input duration and wanted output length
    let factor = duration / final_length;
    let factor = 1.0 / factor;

    let setpts = format!("setpts={factor}*PTS");

    println!("Creating the timelapse");
    let _ = Command::new("ffmpeg")
        .args([
            "-i",
            args.input.as_path().to_str().unwrap(),
            "-filter:v",
            &setpts,
            tmp_out_name.to_str().unwrap(),
        ])
        .output();

    // Trim the produced timelapse to the correct length
    println!("Trimming the timelapse");
    let _ = Command::new("ffmpeg")
        .args([
            "-i",
            tmp_out_name.to_str().unwrap(),
            "-ss",
            "0.0",
            "-t",
            &(final_length + 1.0).to_string(),
            "-y", // Overwrite output files
            args.output.to_str().unwrap(),
        ])
        .output()?;

    // Success
    Ok(())
}
