import QtQuick
import Quickshell
import Quickshell.Io

Item {
  Process {
    // ruleid: qml-no-shell-command
    command: ["sh", "-c", "xdg-open " + root.url]
  }
  Process {
    // ruleid: qml-no-shell-command
    command: [ "/usr/bin/bash", "-lc", 'exec "$@"', "bash", "xdg-open", root.url ]
  }
  function open(url) {
    // ruleid: qml-no-shell-command
    Quickshell.execDetached(["zsh", "-c", url])
    // ok: qml-no-shell-command
    Quickshell.execDetached(["xdg-open", url])
  }
  Process {
    // ok: qml-no-shell-command
    command: ["thedailybugle", "feed", "--json"]
  }
  Process {
    // ok: qml-no-shell-command
    command: ["flash", "--json"]
  }
}
