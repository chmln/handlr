<h1 align=center> <img src="https://user-images.githubusercontent.com/11352152/82113733-3f9c9800-9726-11ea-977d-a2f43e5d392e.png" width=64 align=top /><br/>handlr</h1>

Manage your default applications with ease using `handlr`!

## Features

- Set default handler by extension or mime-type
- Intelligent mime type detection from files based on extension and content
- Open multiple files at once
- Set multiple handlers for mime/extension and use `rofi`/`dmenu` to pick one
- Wildcard support like `text/*`
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

# Set wildcard handler for all text files
handlr set 'text/*' nvim.desktop

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
- Superb autocomplete (currently fish, zsh and bash), including mimes, extensions, and `.desktop` files
- Optional json output for scripting
- Properly supports `Terminal=true` entries

## Setting default terminal 

Unfortunately, there isn't an XDG spec and thus a standardized way for `handlr` to get your default terminal emulator to run `Terminal=true` desktop entries. There was a proposal floating around a few years ago to use `x-scheme-handler/terminal` for this purpose. It seems to me the least worst option, compared to handling quirks of N+1 distros or using a handlr-specific config option. 

Now if `x-scheme-handler/terminal` is present, `handlr` will use it. 

Otherwise, `handlr` will:
1. Find an app with `TerminalEmulator` category
2. Set it as the default for `x-scheme-handler/terminal`
3. Send you a notification to let you know it guessed your terminal and provide instructions to change it if necessary

On the upside, `Terminal=true` entries will now work outside of interactive terminals, unlike `xdg-utils`.

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
2. Download completions for fish:
```sh
curl https://raw.githubusercontent.com/chmln/handlr/master/completions/handlr.fish --create-dirs -o ~/.config/fish/completions/handlr.fish
```

## Attribution
Icons made by <a href="https://www.flaticon.com/authors/eucalyp" title="Eucalyp">Eucalyp</a> from <a href="https://www.flaticon.com/" title="Flaticon"> www.flaticon.com</a>

Cover photo by [creativebloq.com](https://creativebloq.com)
