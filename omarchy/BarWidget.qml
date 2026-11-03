import QtQuick
import Quickshell
import Quickshell.Io
import qs.Ui
import "Model.js" as Model

// The bar button. It owns the feed: runs `thedailybugle feed --json`, keeps
// the state, and opens links. FeedPanel.qml only displays what it is given.
// WidgetButton renders its text with Text.PlainText.
BarWidget {
  id: root
  moduleName: "davifelype.thedailybugle"

  readonly property int refreshIntervalMs: 15 * 60 * 1000
  // A run still going after this counts as failed, so a hung binary cannot
  // leave the widget waiting forever.
  readonly property int runTimeoutMs: 30 * 1000

  property var feedState: Model.initialState()

  // Exit and stream-finished arrive in no guaranteed order (see Omarchy's
  // speedtest panel), so a run is applied once both have arrived.
  property var pendingStdout: null
  property var pendingExitCode: null
  property bool runStarted: false

  function refresh() {
    if (feedProcess.running) return
    pendingStdout = null
    pendingExitCode = null
    runStarted = false
    feedProcess.running = true
    runTimeout.restart()
  }

  function finishRun(stdout, exitCode) {
    runTimeout.stop()
    feedState = Model.nextState(feedState, stdout, exitCode)
  }

  function finishRunWhenComplete() {
    if (pendingStdout === null || pendingExitCode === null) return
    finishRun(pendingStdout, pendingExitCode)
  }

  function openLink(url) {
    if (!Model.isOpenableUrl(url)) return
    // An argument list, not a command line: no shell ever reads the URL.
    Quickshell.execDetached(["xdg-open", url])
  }

  function togglePanel() {
    feedPanel.toggle()
  }

  // Shape contract for shell.summon/hide/toggle routing: the bar looks for
  // open/close/opened on the bar-widget root (same as Omarchy's weather).
  readonly property bool opened: feedPanel.opened
  readonly property bool popoutSwitchClosing: feedPanel.popoutSwitchClosing

  function open() {
    feedPanel.open()
  }

  function close() {
    feedPanel.close()
  }

  function closeForPopoutSwitch() {
    feedPanel.closeForPopoutSwitch()
  }

  implicitWidth: button.implicitWidth
  implicitHeight: button.implicitHeight


  Process {
    id: feedProcess
    command: ["thedailybugle", "feed", "--json"]
    stdout: StdioCollector {
      waitForEnd: true
      onStreamFinished: {
        root.pendingStdout = String(text || "")
        root.finishRunWhenComplete()
      }
    }
    onStarted: root.runStarted = true
    onExited: function(exitCode, exitStatus) {
      // A crash has no meaningful exit code; treat it as a failure.
      root.pendingExitCode = exitStatus === 0 ? exitCode : -1
      root.finishRunWhenComplete()
    }
    // A missing binary never starts, and then neither signal above may fire.
    onRunningChanged: {
      if (!running && !root.runStarted) root.finishRun("", null)
    }
  }

  Timer {
    id: runTimeout
    interval: root.runTimeoutMs
    onTriggered: {
      feedProcess.running = false
      root.finishRun("", null)
    }
  }

  Timer {
    interval: root.refreshIntervalMs
    running: true
    repeat: true
    triggeredOnStart: true
    onTriggered: root.refresh()
  }

  // Its popup is a window of its own, so this item stays hidden.
  FeedPanel {
    id: feedPanel
    visible: false
    bar: root.bar
    anchorItem: button
    hostWidget: root
  }

  WidgetButton {
    id: button
    anchors.fill: parent
    bar: root.bar
    text: Model.barText(root.feedState)
    horizontalMargin: 7.5
    // The panel is the detail view.
    tooltipText: ""

    onPressed: function(pressedButton) {
      if (pressedButton === Qt.MiddleButton) root.refresh()
      else root.togglePanel()
    }
  }
}
