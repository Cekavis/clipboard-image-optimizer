<p align="center">
  <img src="src-tauri/icons/icon.svg" alt="Clipboard Image Optimizer Icon" width="120" />
</p>

<h3 align="center">
<a href="https://lowtechguys.com/clop/">Clop</a> alternative for Windows, based on <a href="https://tauri.app/">Tauri</a>.
</h3>

## Features

- 🖼️ **Clipboard Monitoring:** Automatically detects images copied to your clipboard.
- ⚡ **JPEG Optimization:** Uses [mozjpeg](https://github.com/mozilla/mozjpeg) to encode JPEGs at quality 60 for a great balance of size and quality.
- 🦀 **Powered by Rust & Tauri:** Leverages the safety and performance of Rust, and the modern desktop capabilities of Tauri.

## Installation

Download the latest release from the [Releases](https://github.com/Cekavis/clipboard-image-optimizer/releases) page.

## Usage

Just copy an image to your clipboard as usual. Clipboard Image Optimizer will automatically optimize it in the background. Paste the image anywhere (e.g., chat, email, documents) and enjoy smaller file sizes!

## Credits

- Inspired by [Clop](https://github.com/pit-ray/clop)
- Built with [Tauri](https://tauri.app/)
- Uses [mozjpeg](https://github.com/mozilla/mozjpeg) for JPEG optimization
- Uses [mozjpeg-rust](https://github.com/ImageOptim/mozjpeg-rust) wrapper for Rust bindings to mozjpeg

## License

This project is licensed under the GNU General Public License v3.0.
