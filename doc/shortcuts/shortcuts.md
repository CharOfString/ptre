# Shortcuts
> **Note**: If we provide shortcuts like `Ctrl + X` `Ctrl + S`, it means that you'll need to press `Ctrl + X` first and 
> hit `Ctrl + S` immediately after then.

* **Save**: `Ctrl + X` `Ctrl + S`.
* **Quit**: `Ctrl + X` `Ctrl + C`.
* **Cut**: `Ctrl + W`.
* **Copy**: `Alt + W`.
* **Paste**: `Ctrl + Y`.
* **Complete** (open/close the completion popup): `Alt + /`.
* **Toggle auto completion**: `Ctrl + C` `Alt + L`.
* **Apply autofix**: `Ctrl + C` `A`.
* **Search forward / backward**: `Ctrl + S` / `Ctrl + R`. (`Enter` to stop, `Ctrl + G` to go back.)
* **Query replace**: `Alt + %`. Then answer at each match: `Y`/`Space` replace, `N` skip, `!` replace all, `.` replace and stop, `Q` stop.
* **Go to line**: `Alt + G` `G` (or `Alt + G` `Alt + G`).

## Terminal

* `Ctrl + C` `T` `T`: open or hide the terminal. Hiding it keeps the shell running.
* `Ctrl + X` `O`: switch focus between the visible panes: the file tree, the editor and the terminal.
* `Ctrl + C` `T` `C`: run Clang-tidy.
* `Alt + V` / `Ctrl + V`: scroll terminal history up / down one page.
  `Shift + PageUp` / `Shift + PageDown` also work. Up to 1000 history rows are kept.
  These shortcuts apply while the terminal has focus; typing returns to the latest output.
  In alternate-screen programs, these keys go to the program instead.
* Shell keys, including Ctrl-C and Ctrl-D, go to the terminal while it has focus.
  Switch back with `Ctrl + X` `O` before using editor commands.

The terminal opens below the editor when the workspace has at least 28 rows;
otherwise it opens on the right. It starts in the current file's directory,
or the editor's working directory for an unnamed file. After the shell exits,
hide and reopen the terminal to start a new session.

## File tree

* `Ctrl + C` `C` `T`: show or hide the file tree on the left, like Emacs Treemacs.
  It is also in the WINDOW menu. Showing the tree gives it focus.
* `N` / `P`: next / previous row. `Down` / `Up` and `Ctrl + N` / `Ctrl + P` also work.
* `Tab`: open or close the selected folder.
* `Enter`: open the selected file in a new tab (or go to its tab when it is open already),
  or open or close the selected folder.
* `U`: go to the parent folder.
* `G`: read the folders again.
* `Q`: hide the tree.

The tree starts at the project of the current file: the nearest folder above it that holds
`.git`, `Cargo.toml`, `CMakeLists.txt`, `compile_commands.json` or `compile_flags.txt`
(otherwise the file's own folder). For an unnamed file, it starts at the editor's working directory.
It opens the folders above the current file and selects it. The `.git` folder is not shown.

## Tabs

* `Ctrl + X` `Right` / `Ctrl + X` `Left`: go to the next / previous tab. At the last tab, the next one is the first.
* `Ctrl + X` `K`: close the tab. If it has unsaved changes, a dialog asks first:
  `Y` / `Enter` closes it without saving, `N` / `Esc` keeps it open.

The tab row is the first line of the editor. The active tab is highlighted, and `●` marks a tab
with unsaved changes. A file from the file tree opens in a new tab after the active one; an empty
unnamed tab is reused for it. Closing the last tab leaves an empty unnamed tab.
