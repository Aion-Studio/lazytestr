
# lazytestr

`lazytestr` is a command-line utility  designed to help manage and run tests with an intuitive terminal UI. This tool scans for test files, lists available tests, and allows you to run tests directly from the terminal with colorized output and live updates.
![Screenshot](https://github.com/Aion-Studio/lazytestr/assets/15694731/9fa07094-60db-4a02-b327-7243ea144d9b)


## Features

- **Test Discovery**: Automatically scans and lists all available test files and test functions.
- **Intuitive UI**: Navigate through test files and test functions using a terminal-based user interface.
- **Live Test Output**: Run tests and view live output with color-coded results.
- **Watch Mode**: Automatically re-run tests when source files change.

## Installation

To install My Rust Tool, follow these steps:

1. **Clone the Repository**:
    ```sh
    git clone git@github.com:Aion-Studio/lazytestr.git
    cd lazytestr
    ```


2. **Build and Move Binary to `/usr/local/bin`**:

    Ensure that `/usr/local/bin` is in your `PATH`:
    ```sh
    echo $PATH
    ```

    If `/usr/local/bin` is not in your `PATH`, add it by editing your shell profile (`~/.bashrc`, `~/.zshrc`, etc.):
    ```sh
    export PATH=$PATH:/usr/local/bin
    ```
    ```sh
    ./build
    ```

OR

Clone and build the project yourself and move it to wherever you have local binaries in your path.

    

## Usage

Run the tool by executing:

```sh
lazytestr
```

Key Bindings


Navigation:


`h / l`: Switch between panes.


`j / k`: Move up/down in the current pane.


Actions:


Enter: Run the selected test.


`w`: Toggle watch mode.


`q`: Quit the application.


`y`: Copy debug contents or test output to clipboard.



