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
* `Ctrl + X` `O`: switch focus between the editor and the visible terminal.
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
