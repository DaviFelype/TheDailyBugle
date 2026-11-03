pragma ComponentBehavior: Bound

import QtQuick
import qs.Commons
import qs.Commons as Commons
import qs.Ui
import "Model.js" as Model

// The popup: the error message, if any, and the list of items. Every text
// from the feed is shown with Text.PlainText. Font sizes call Style.fontToken
// directly because qmllint cannot see the members of Style.font.
Panel {
  id: root
  moduleName: "davifelype.thedailybugle"
  ipcTarget: "davifelype.thedailybugle"
  manageIpc: false

  property var anchorItem: null
  // The BarWidget that owns the feed; the bar identifies the panel by it.
  property var hostWidget: null
  readonly property var barIdentity: hostWidget || root
  readonly property var feedState: hostWidget ? hostWidget.feedState : Model.initialState()

  function openLink(url) {
    if (hostWidget) hostWidget.openLink(url)
  }

  KeyboardPanel {
    id: panel
    anchorItem: root.anchorItem
    owner: root.barIdentity
    bar: root.bar
    open: root.opened
    centerOnBar: true
    focusTarget: keyCatcher
    contentWidth: panel.fittedContentWidth(Style.space(480))
    contentHeight: panel.fittedContentHeight(feedColumn.implicitHeight)

    PanelKeyCatcher {
      id: keyCatcher
      anchors.fill: parent
      onCloseRequested: root.close()
      onTabRequested: function(direction) { root.switchPanel(direction) }

      Flickable {
        id: feedScroll
        anchors.fill: parent
        contentWidth: width
        contentHeight: feedColumn.implicitHeight
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        interactive: contentHeight > height

        Column {
          id: feedColumn
          width: feedScroll.width
          spacing: Style.space(10)

          Text {
            width: parent.width
            visible: text !== ""
            text: Model.errorMessage(root.feedState.status)
            textFormat: Text.PlainText
            wrapMode: Text.Wrap
            color: Commons.Color.urgent
            font.family: Style.fontFamily
            font.pixelSize: Style.fontToken("body", Style.fontPx(1.0))
          }

          Repeater {
            model: root.feedState.items

            delegate: MouseArea {
              id: itemArea
              required property var modelData

              width: feedColumn.width
              height: itemColumn.implicitHeight
              cursorShape: Qt.PointingHandCursor
              hoverEnabled: true
              onClicked: root.openLink(itemArea.modelData.url)

              Column {
                id: itemColumn
                width: parent.width
                spacing: Style.space(2)

                Text {
                  width: parent.width
                  text: itemArea.modelData.title
                  textFormat: Text.PlainText
                  wrapMode: Text.Wrap
                  color: itemArea.containsMouse ? Commons.Color.accent : Commons.Color.foreground
                  font.family: Style.fontFamily
                  font.pixelSize: Style.fontToken("body", Style.fontPx(1.0))
                }

                Text {
                  width: parent.width
                  text: itemArea.modelData.source
                  textFormat: Text.PlainText
                  elide: Text.ElideRight
                  color: Commons.Color.muted
                  font.family: Style.fontFamily
                  font.pixelSize: Style.fontToken("body-small", Style.fontPx(0.917))
                }
              }
            }
          }
        }
      }
    }
  }
}
