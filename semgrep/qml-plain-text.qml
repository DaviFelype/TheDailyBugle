import QtQuick

Column {
  Text {
    // ruleid: qml-no-rich-text
    textFormat: Text.RichText
  }
  Text {
    // ruleid: qml-no-rich-text
    textFormat: Text.StyledText
  }
  Text {
    // ruleid: qml-no-rich-text
    textFormat: Text.MarkdownText
  }
  Text {
    // ruleid: qml-no-rich-text
    textFormat: Text.AutoText
  }
  Text {
    // ok: qml-no-rich-text
    textFormat: Text.PlainText
  }
}
