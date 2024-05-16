# Atari ST(E) MSA-to-ST disk image Converter

This Rust project is part of the broader Atari ST enthusiasts' community and is aimed at individuals who are interested in Atari ST emulators, specifically, the MiSTer FPGA project.

The Atari ST is a line of personal computers that was popular in the late 1980s and 1990s+. This tool aids those interested in exploring this vintage technology in the present day by utilizing modern hardware like FPGA.

The program is a simple and efficient tool for converting MSA (Magic Shadow Archiver) files to ST format. MSA files are a common format for Atari ST disk images, while ST is a disk image format used by the MiSTer FPGA. This tool traverses the specified directory, identifies each MSA file, decodes it, and converts it over to the corresponding ST format.

## Understanding the Formats

### MSA (Magic Shadow Archiver)

This is a disk image format originally used by the Magic Shadow Archiver program on the Atari ST. The MSA format is a file-by-file, track-by-track representation of the original diskette, which makes it ideal for archiving and reproduction.

### ST (MiSTer FPGA and others)

The ST disk image format is used by the MiSTer FPGA project and other Atari ST emulators. It's a binary image of the entire floppy disk, sector by sector, which makes it ideal for fast and precise reading by emulators.

This conversion tool in Rust aids in the process of transforming MSA files into readable ST format, enabling them to be used in Atari ST emulators like MiSTer FPGA.

## How To Use It

To use this program, ensure you have Rust installed in your environment. Then compile your Rust program with `rustc`.

To run the program, you can enter the paths to your source and destination directories as arguments.
