# Uroborus manager

## Description

A mod manager for Resident Evil 5 (2009)

## Getting started

### Prerequisites

- [Node.js](https://nodejs.org/en)
- [Rust](https://www.rust-lang.org/tools/install)

### Installation

1. Clone the repository:

    ```bash
     git clone https://github.com/TiagoRibeiro25/Uroborus-Manager.git
     cd Uroborus-Manager
    ```

2. Install dependencies:

    ```bash
    npm install
    ```

3. Run the app in development mode (export export WEBKIT_DISABLE_DMABUF_RENDERER=1 on linux with wayland):

    ```bash
    npm run tauri dev
    ```

4. Build the app for production:

    ```bash
    npm run tauri build
    ```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
