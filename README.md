# gview - A TUI Viewer for Specific Git Commit IDs

![gview image](resources/gview.png)

gview is a TUI application that lets you inspect files or search for files by traversing Git commit IDs.

Currently, gview displays the repository located in the user's current working directory.

Please note that this application is a personal hobby project and is not in a finished state. It works reliably for medium-sized repositories with a few hundred commits. However, performance may degrade when working with large repositories containing tens of thousands of commits.

# install

```bash
cargo install gview
```

# default keymap

| Key | Description |
| --- | ----------- |
| <kbd>Enter</kbd> in Match | Jump to the file list |
| <kbd>Enter</kbd> in File List | Open the selected file |
| <kbd>↑</kbd>, <kbd>↓</kbd> in Match | Navigate filter options |
| <kbd>↑</kbd>, <kbd>↓</kbd> in File List | Navigate through files |
| <kbd>↑</kbd>, <kbd>↓</kbd> in Commit ID | Switch between commits |
| <kbd>↑</kbd>, <kbd>↓</kbd> in File Contents | Scroll through file content |
| <kbd>Tab</kbd> | Move to the next section |
| <kbd>←</kbd>, <kbd>→</kbd> in File List | Scroll file names horizontally |
| <kbd>o</kbd> in File Contents | Toggle display mode (line numbers → blame → no lines) |
| <kbd>g</kbd> in File Contents | Open current file in browser at current commit and line |

## Browser Integration

When viewing a file, press <kbd>g</kbd> to open the current file in your web browser. This feature:

- Opens the file at the exact commit ID you're viewing in gview
- Highlights the line that's currently at the top of your view
- Works with GitHub, GitHub Enterprise, and other Git hosting services
- Supports both SSH and HTTPS remote URLs
- Works cross-platform (macOS, Linux, Windows)

**Example**: If you're viewing `src/main.rs` at commit `abc123f` with line 42 at the top of the screen, pressing <kbd>g</kbd> will open:
```
https://github.com/owner/repo/blob/abc123f/src/main.rs#L42
```

# contribution

Contributions are always welcome! Please note that responses may not be immediate, as this is maintained on a best-effort basis.
There are still several implementation tasks that would improve usability and are relatively easy to tackle. Check the Issues page for more details.


# LICENSE
This project is licensed under the MIT License.