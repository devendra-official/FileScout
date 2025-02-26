# File Scout

File Scout is a command-line tool for exploring and managing files in your system. It provides a user-friendly interface to navigate directories, view file details, and perform file operations.

## Features 

- Navigate through directories
- View file details
- Perform file operations (copy, move, delete)
- Keyboard shortcuts for quick actions

## Installation

To install File Scout, you need to have Rust and Cargo installed on your system. Then, you can build the project using Cargo:

```sh
cargo build --release
```
## Usage

After building the project, you can run the `file_scout` executable from the `target/release` directory:

```sh
./target/release/file_scout
```

Use the arrow keys to navigate through directories and files. Press `Enter` to view file details or perform operations on the selected file.

### Keyboard Shortcuts

- `Up/Down Arrow`: Navigate through the list
- `Enter`: View details or perform operations
- `C`: Copy the selected file
- `M`: Move the selected file
- `D`: Delete the selected file
- `Q`: Quit the application

## Contributing

Contributions are welcome! Please open an issue or submit a pull request on GitHub.

## License

This project is licensed under the MIT License. See the `LICENSE` file for details.