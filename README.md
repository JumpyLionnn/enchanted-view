# Enchanted View
The Next gen of image viewing built with Rust ;)

## Installation
### Download prebuilt binaries
You can download prebuilt binaries in the [Releases](https://github.com/JumpyLionnn/enchanted-view/releases/latest) page.
Releases are available for the following platforms:
- Linux
- MacOS
- Windows
### Building from source
1. To build from source you will need rustc and cargo installed. You can Install them [here](https://www.rust-lang.org/tools/install)
2. Clone the repository
   ```
    git clone https://github.com/JumpyLionnn/enchanted-view.git
   ```
3. Run the project in debug mode
   ```
    cargo run -- path/to/image.png
   ```

## Features
- pan and zoom (on pixel level)
- zoom ui
- Checkers background for images with transparency
- rotate/flip

## Planned Features
- picking a color
- pick average color or darkest/lightest color from a region
- convert between formats
- gif - step between frames export frames as other formats or specific frames
- svg - see the svg tree and inspect different elements
- pixel grid
- rulers
- see details about the image like file size, dimensions and more

and a lot more...
### Maybe Features
- printing - depends on platform/library support
- Browser support

## Contributing
Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are greatly appreciated.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement". Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (git checkout -b feature/AmazingFeature)
3. Commit your Changes (git commit -m 'Add some AmazingFeature')
4. Push to the Branch (git push origin feature/AmazingFeature)
5. Open a Pull Request

## License
Distributed under the MIT License. See LICENSE for more information.