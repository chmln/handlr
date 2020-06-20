<h1 align=center> <img src="https://user-images.githubusercontent.com/11352152/82113733-3f9c9800-9726-11ea-977d-a2f43e5d392e.png" width=64 align=top /><br/>handlr</h1>

Manage your default applications with ease using `handlr`!

## Features

- Set default handler by extension or mime-type
- Intelligent mime type detection from files based on extension and content
- Open multiple files at once
- Set multiple handlers for mime/extension and use rofi/dmenu to pick one
- List default associations
- Automatically removes invalid/wrong `.desktop` entries from `mimeapps.list`
- Helper commands like `launch`, `get --json` for your scripting needs
- Unnecessarily fast (written in Rust)
- Single compiled binary with no dependencies

## Usage

```sh
# Open a file/URL
handlr open ~/.dotfiles/pacman/packages.txt
handlr open https://google.ca

# Set default handler for png files
handlr set .png feh.desktop

# Set default handler based on mime
handlr set application/pdf evince.desktop

# List default apps
handlr list

# Get the handler for a mime/extension
$ handlr get .png
feh.desktop

# Launch a handler with given path/URL
handlr launch x-scheme-handler/https -- https://google.ca
```

## Compared to `xdg-utils`
- Can open multiple files/URLs at once
- Can have multiple handlers and use rofi/dmenu to pick one at runtime
- Far easier to use with simple commands like `get`, `set`, `list`
- Can operate on extensions, **no need to look up or remember mime types**
  - useful for common tasks like setting a handler for png/docx/etc files
- Superb autocomplete (currently just fish), including mimes, extensions, and `.desktop` files
- Optional json output for commands like `get`

## Setting multiple handlers

1) Open `~/.config/handlr/handlr.toml` and set `enable_selector = true`. Optionally, you can also tweak the `selector` to your selector command (using e.g. rofi or dmenu).

2) Add a second/third/whatever handler using `handlr add`, for example
```
handlr add x-scheme-handler/https firefox-developer-edition.desktop
```

3) Now in this example when you open a URL, you will be prompted to select the desired application.

![](https://user-images.githubusercontent.com/11352152/85187445-c4bb2580-b26d-11ea-80a6-679e494ab062.png)

## Screenshots

<table><tr><td>
<img src=https://user-images.githubusercontent.com/11352152/82159698-2434a880-985e-11ea-95c7-a07694ea9691.png width=500>
</td><td>
<img width=450 src=https://user-images.githubusercontent.com/11352152/82159699-2434a880-985e-11ea-9493-c21773093c38.png>
</td></tr></table>

## Installation

### Arch Linux

```sh
yay -S handlr-bin
```

Optionally you can also install `xdg-utils-handlr` to replace `xdg-open`:

```sh
yay -S xdg-utils-handlr
```

### Rust/Cargo

```sh
cargo install handlr
```

### Binaries

1. Download the latest [release binary](https://github.com/chmln/handlr/releases) and put it somewhere in `$PATH`
2. Download completions:
```sh
curl https://raw.githubusercontent.com/chmln/handlr/master/completions/handlr.fish --create-dirs -o ~/.config/fish/completions/handlr.fish
```

## Attribution
Icons made by <a href="https://www.flaticon.com/authors/eucalyp" title="Eucalyp">Eucalyp</a> from <a href="https://www.flaticon.com/" title="Flaticon"> www.flaticon.com</a>

Cover photo by [creativebloq.com](https://creativebloq.com)
