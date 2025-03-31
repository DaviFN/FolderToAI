use lazy_static::lazy_static;
use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;

lazy_static! {
    static ref BINARY_MAGIC: Vec<&'static [u8]> = vec![
        // Image Formats
        &[0x89, 0x50, 0x4E, 0x47],  // PNG
        &[0x47, 0x49, 0x46, 0x38],  // GIF
        &[0xFF, 0xD8, 0xFF],        // JPEG
        &[0x42, 0x4D],              // BMP
        &[0x49, 0x49, 0x2A, 0x00],  // TIFF (Intel)
        &[0x4D, 0x4D, 0x00, 0x2A],  // TIFF (Motorola)
        &[0x57, 0x45, 0x42, 0x50],  // WebP
        
        // Compressed Archives
        &[0x50, 0x4B, 0x03, 0x04],  // ZIP
        &[0x1F, 0x8B, 0x08],        // GZIP
        &[0x42, 0x5A, 0x68],       // BZip2
        &[0xFD, 0x37, 0x7A, 0x58], // XZ
        &[0x52, 0x61, 0x72, 0x21], // RAR
        &[0x37, 0x7A, 0xBC, 0xAF], // 7-Zip
        
        // Executables and Binaries
        &[0x7F, 0x45, 0x4C, 0x46],  // ELF
        &[0x4D, 0x5A],             // DOS/Windows
        &[0xCA, 0xFE, 0xBA, 0xBE],  // Java Class
        &[0x00, 0x61, 0x73, 0x6D],  // WebAssembly
        
        // Media Formats
        &[0x52, 0x49, 0x46, 0x46],  // WAV/AVI
        &[0x66, 0x74, 0x79, 0x70],  // MP4
        &[0x49, 0x44, 0x33],        // MP3 (ID3v2 header)
        &[0x4F, 0x67, 0x67, 0x53],  // OGG
        &[0x1A, 0x45, 0xDF, 0xA3],  // WebM
        &[0x00, 0x00, 0x01, 0xB3],  // MPEG-2
        &[0x00, 0x00, 0x01, 0xB6],  // MPEG-4
        &[0x66, 0x4C, 0x61, 0x43],  // FLAC
        &[0x4D, 0x54, 0x68, 0x64],  // MIDI
        
        // Documents and Data
        &[0x25, 0x50, 0x44, 0x46],  // PDF
        &[0xD0, 0xCF, 0x11, 0xE0],  // Microsoft OLE
        &[0x53, 0x51, 0x4C, 0x69, 0x74, 0x65], // SQLite
        
        // Installers
        &[0x30, 0x26, 0xB2, 0x75],  // MSI
        &[0x00, 0x00, 0x00, 0x18, 0x66, 0x74, 0x79, 0x70], // HEIF/HEIC
    ];

    static ref BINARY_EXTENSIONS: Vec<&'static str> = vec![
        // Executables and Installers
        "exe", "bin", "msi", "apk", "ipa", "deb", "rpm", "pkg", "dmg", "appx", "appxbundle",
        
        // Compressed Archives
        "zip", "rar", "7z", "gz", "bz2", "xz", "tar", "tgz", "tbz2", "txz", "lzma", "lz4", "zst",
        
        // Image Formats
        "png", "jpg", "jpeg", "gif", "bmp", "tiff", "webp", "heic", "heif", "ico", "cur", "psd", "xcf", "svgz",
        
        // Audio and Video Formats
        "mp3", "mp4", "wav", "ogg", "flac", "midi", "aac", "m4a", "mov", "avi", "mkv", "webm", "mpg", "mpeg", "rmvb",
        "rm", "ra", "ram", "3gp", "3g2", "asf", "wmv", "wma", "flv", "swf",
        
        // System and Database Files
        "db", "sqlite", "sqlite3", "mdb", "accdb", "dll", "so", "dylib", "o", "a", "lib", "sys",
        
        // Virtual Machine and Disk Images
        "vmdk", "vdi", "vhd", "vhdx", "iso", "img",
        
        // Binary Documents
        "pdf",

        // Others
        "pdb",
    ];

    static ref TEXT_EXTENSIONS: Vec<&'static str> = vec![
        // Documentation and Text Files
        "txt", "md", "rst", "tex", "bib", "sty", "cls", "log", "csv", "tsv", "toml",
        
        // Web Development
        "html", "css", "js", "json", "yaml", "yml", "xml",
        
        // Programming Languages
        "py", "java", "c", "cpp", "h", "hpp", "cc", "cxx", "swift", "go", "rb", "php", "perl", "pl",
        "rs", "rlib", "lua", "tcl", "awk", "sed", "m", "m4", "sh", "bash", "zsh", "fish",
        "kt", "kts", "groovy", "scala", "sbt", "scm", "lisp", "el", "emacs",
        
        // Scripting Languages
        "bat", "cmd", "ps1", "psm1", "vbs", "vbscript",
        
        // Configuration and Data Files
        "ini", "conf", "cfg", "properties", "sql", "env", "dotenv",
        
        // Build and Project Files
        "pom", "gradle", "build.gradle", "build.xml", "Makefile", "CMakeLists.txt",
        
        // Version Control
        "gitignore", "gitattributes",
        
        // IDE and Editor Configurations
        "iml", "project", "settings.json", "vscode", "idea",
        
        // Other Programming Files
        "f90", "f", "f03", "f08", "f77", "f95", "for", "fpp", "creole", "feature", "cu", "cuh",
        "pyx", "pxd", "pxi", "erl", "es", "escript", "hrl", "xrl", "yrl", "fs", "fsi", "fsx",
        "fx", "flux", "g", "gap", "gd", "gi", "tst", "glsl", "fp", "frag", "frg", "fsh", "rno",
        "roff", "gvy", "gsp", "hcl", "tf", "hlsl", "fxh", "hlsli", "rdoc", "rbbas", "rbfrm",
        "rbmnu", "rbres", "rbtbar", "rbuistate", "rhtml", "raml", "qml", "qbs", "pro", "pri",
        "r", "rd", "rsx", "gcode", "gco", "gams", "gms", "mtml", "muf", "maxscript", "ms", "mcr",
    ];
}

pub fn file_is_binary(path: &str) -> bool {
    // check file extension first; if a file matches a text/binary extension, it is classified as such
    if let Some(ext) = path.rsplit_once('.') {
        let extension = ext.1.to_lowercase();
        if BINARY_EXTENSIONS.contains(&extension.as_str()) {
            return true;
        } else if TEXT_EXTENSIONS.contains(&extension.as_str()) {
            return false;
        }
    }

    // open the file
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(_) => return false, // default to not binary if file cannot be opened
    };

    // magic number detection (although this will only work in the rare case the file's extension has been changed)
    let mut magic_buffer = [0; 8];
    let bytes_read = match file.read(&mut magic_buffer) {
        Ok(bytes_read) => bytes_read,
        Err(_) => return false, // Default to not binary if read fails
    };

    // check if magic number matches any known binary format
    if BINARY_MAGIC.iter().any(|sig| 
        bytes_read >= sig.len() && &magic_buffer[..sig.len()] == *sig
    ) {
        return true; // if magic number matches, return true immediately
    }

    // reset the file position to the beginning for content analysis
    match file.seek(SeekFrom::Start(0)) {
        Ok(_) => (),
        Err(_) => return true, // return binary if seek fails
    };

    // second check: content analysis (remaining of the first 10 KiB)
    const CONTENT_BUFFER_SIZE: usize = 10 * 1024;
    let mut content_buffer = vec![0; CONTENT_BUFFER_SIZE];
    let content_bytes_read = match file.read(&mut content_buffer) {
        Ok(bytes_read) => bytes_read,
        Err(_) => return false, // default to not binary if read fails
    };

    // check if it's valid UTF8
    let utf8_valid = std::str::from_utf8(&content_buffer[..content_bytes_read]).map_or(false, |s| {
        // check if the string contains any invalid UTF-8 sequences
        s.chars().all(|c| c.is_ascii() || c.is_ascii_graphic() || c.is_ascii_control() || c.is_ascii_punctuation())
    });

    if utf8_valid {
        return false;
    }

    const PERCENTAGE_OF_NON_ASCII_CHARACTERS_ALLOWED: f64 = 0.20;
    // check for null bytes or non-UTF8 sequences, allowing some non-ASCII characters as defined in 'PERCENTAGE_OF_NON_ASCII_CHARACTERS_ALLOWED'
    let non_ascii_count = content_buffer.iter().take(content_bytes_read).filter(|&&b| b == 0 || b < 0x20 || b > 0x7E).count();
    let threshold = (content_bytes_read as f64 * PERCENTAGE_OF_NON_ASCII_CHARACTERS_ALLOWED).ceil() as usize; // allow up to a certain percentage of non-ASCII characters

    return non_ascii_count > threshold;
}

pub fn get_file_size_in_bytes(path: &str) -> Result<usize, ()> {
    if let Ok(metadata) = std::fs::metadata(path) {
        return Ok(metadata.len() as usize);
    }
    Err(())
}