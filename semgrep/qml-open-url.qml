import QtQuick

MouseArea {
  // ruleid: qml-no-open-url-externally
  onClicked: Qt.openUrlExternally(model.link)
  // ok: qml-no-open-url-externally
  onPressed: root.openLink(model.link)
}
