<h1 align=center> <img src="https://user-images.githubusercontent.com/11352152/82113733-3f9c9800-9726-11ea-977d-a2f43e5d392e.png" width=64 align=top /><br/>handlr</h1>

Manage your default applications with ease using `handlr`!

## Features

- Set default handler by extension or mime-type
- Open path/url with default handler (like xdg-open)
- List default associations
- Automatically removes invalid/wrong `.desktop` entries from `mimeapps.list`
- Simply a great command-line experience

## Compared to `xdg-open` and `xdg-mime`
- Can operate on extensions, **no need to look up or remember mime types**
  - useful for common tasks like setting a handler for png/docx/etc files
- Superb autocomplete (currently just fish), including mimes, extensions, and `.desktop` files
- Optional json output for commands like `get`


## Screenshots

<table><tr><td>
<img src=https://user-images.githubusercontent.com/11352152/82159698-2434a880-985e-11ea-95c7-a07694ea9691.png width=500>
</td><td>
<img width=450 src=https://user-images.githubusercontent.com/11352152/82159699-2434a880-985e-11ea-9493-c21773093c38.png>
</td></tr></table>

## Installation

Just download the latest [release binary](https://github.com/chmln/handlr/releases).

---

Alternatively, you can install with `cargo`:

```sh
cargo install handlr
```

#### Icon Attribution
Icons made by <a href="https://www.flaticon.com/authors/eucalyp" title="Eucalyp">Eucalyp</a> from <a href="https://www.flaticon.com/" title="Flaticon"> www.flaticon.com</a>
