use clap::Parser;
use clap::{arg, command};
use std::fs::{self, DirEntry};
use std::path::PathBuf;
use std::process::Command;

/// Simple program add subtitles to video files. Create two directories one with the subtitles and one containing the video files.
/// Make sure that both directories only contain the specified files and no other.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Exit after the nth file was processed
    #[arg(short, long)]
    exit: Option<usize>,

    /// Language of the subtitles probably 3 char ISO code https://en.wikipedia.org/wiki/List_of_ISO_639_language_codes
    #[arg(short, long)]
    language: String,

    /// Offset for the subtitles (default seconds) +Number is -Offset and -Number is +Offset idk who thought this is a good idea
    /// https://ffmpeg.org/ffmpeg-utils.html#time-duration-syntax
    #[arg(short, long, allow_negative_numbers = true)]
    offset: Option<isize>,

    /// Source directory path for subtitle files
    #[arg(short, long)]
    source_dir: PathBuf,

    /// Target directory path for video files
    #[arg(short, long)]
    target_dir: PathBuf,

    /// Verbose output including all of the ffmpeg output
    #[arg(short, long, action)]
    verbose: bool,

    /// Yes flag for ffmpeg e.g. overwrite output file if it exists (adds -y to ffmpeg)
    #[arg(short, long, action)]
    yes: bool,
}

fn main() {
    let args = Args::parse();

    let mut source_files: Vec<_> = fs::read_dir(args.source_dir)
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    let mut target_files: Vec<_> = fs::read_dir(args.target_dir)
        .unwrap()
        .map(|r| r.unwrap())
        .collect();

    source_files.sort_by_key(|dir| dir.path());
    target_files.sort_by_key(|dir| dir.path());

    let mut filtered_target_files: Vec<DirEntry> = vec![];
    for file in target_files {
        if file.path().is_dir() {
            continue;
        }
        if file
            .path()
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .ends_with("_merged")
        {
            continue;
        }
        filtered_target_files.push(file);
    }

    target_files = filtered_target_files;

    let stop_after = args.exit.unwrap_or_default();
    for (idx, (path, target_path)) in source_files.iter().zip(target_files).enumerate() {
        if stop_after > 0 && stop_after == idx {
            break;
        }
        let path = path.path();
        println!("Subtitle path: {}", path.display());
        let target_path = target_path.path();
        println!("Video path: {}", target_path.display());

        let output_path = target_path.file_stem().unwrap().to_str().unwrap();
        let dir = target_path.parent().unwrap();
        let output_path = dir.join(output_path).to_str().unwrap().to_owned()
            + "_merged"
            + "."
            + &target_path.extension().unwrap().to_string_lossy();
        println!("Output path: {}", output_path);

        let mut command = Command::new("ffmpeg");
        if args.yes {
            command.arg("-y");
        }

        if let Some(offset) = args.offset {
            command.arg("-itsoffset").arg(offset.to_string());
        }

        command
            .arg("-i")
            .arg(&target_path)
            .arg("-i")
            .arg(&path)
            .arg("-map")
            .arg("0")
            .arg("-map")
            .arg("1")
            .arg("-c")
            .arg("copy");

        if !args.language.is_empty() {
            command
                .arg("-metadata:s:s:1")
                .arg(format!("language={}", args.language.trim()));
        }

        if args.verbose {
            command.arg(output_path).status().expect("ffmpeg failed");
        } else {
            command.arg(output_path).output().expect("ffmpeg failed");
        }
    }
}
