# File Scout

File Scout is a command-line tool for exploring and managing files in your system. It provides a user-friendly interface to navigate directories, view file details, and perform file operations.

## Features 

- Navigate through directories
- View text files
- Keyboard shortcuts for quick actions

## Installation

To install File Scout, you need to have Rust and Cargo installed on your system. Then, you can build the project using Cargo:

```sh
cargo build --release
```
## Usage

After building the project, you can run the `file_scout` executable from the `target/release` directory:

## Usage

After building the project, you can run the `fs` executable from the release directory:

```sh
./target/release/fs
```

Use the arrow keys to navigate through directories and files. Press `Enter` to view file details or perform operations on the selected file.

### Keyboard Shortcuts

- `Left Arrow`: Go to the parent directory or scroll left in the content view
- `Right Arrow`: Enter the selected directory or scroll right in the content view
- `Tab`: Toggle between list view and content view (Currently supported UTF-8 only)
- `Up Arrow`: Move up in the list view or scroll up in the content view
- `Down Arrow`: Move down in the list view or scroll down in the content view
- `C`: Change the color scheme
- `E`: File Encryption
- `D`: File Decryption
- `O`: Open File
- `Delete`: Delete the selected file
- `Q`: Quit the application

## Contributing
Contributions are welcome! Please open an issue or submit a pull request on GitHub.

## License

This project is licensed under the MIT License. See the `LICENSE` file for details.
