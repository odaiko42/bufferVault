"""
GUI for BufferVault
Provides interface for browsing and selecting clipboard history
"""
import sys
from PyQt5.QtWidgets import (
    QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout,
    QListWidget, QListWidgetItem, QPushButton, QLabel, QLineEdit,
    QSystemTrayIcon, QMenu, QMessageBox, QDialog, QTextEdit
)
from PyQt5.QtCore import Qt, QTimer, pyqtSignal
from PyQt5.QtGui import QIcon, QFont
from clipboard_monitor import ClipboardMonitor
from config import Config


class ClipboardHistoryDialog(QDialog):
    """Dialog to show full clipboard entry content"""
    
    def __init__(self, entry, parent=None):
        super().__init__(parent)
        self.entry = entry
        self.setWindowTitle("Clipboard Entry Details")
        self.setMinimumSize(600, 400)
        self.init_ui()
    
    def init_ui(self):
        layout = QVBoxLayout()
        
        # Time label
        time_label = QLabel(f"Copied at: {self.entry.get_display_time()}")
        time_label.setFont(QFont("Arial", 10, QFont.Bold))
        layout.addWidget(time_label)
        
        # Content
        content_label = QLabel("Content:")
        layout.addWidget(content_label)
        
        content_text = QTextEdit()
        content_text.setPlainText(self.entry.content)
        content_text.setReadOnly(True)
        layout.addWidget(content_text)
        
        # Buttons
        button_layout = QHBoxLayout()
        
        copy_btn = QPushButton("Copy to Clipboard")
        copy_btn.clicked.connect(self.accept)
        button_layout.addWidget(copy_btn)
        
        close_btn = QPushButton("Close")
        close_btn.clicked.connect(self.reject)
        button_layout.addWidget(close_btn)
        
        layout.addLayout(button_layout)
        self.setLayout(layout)


class BufferVaultGUI(QMainWindow):
    """Main GUI window for BufferVault"""
    
    def __init__(self):
        super().__init__()
        self.config = Config()
        self.monitor = ClipboardMonitor(config=self.config)
        self.init_ui()
        self.setup_system_tray()
        
        # Start clipboard monitoring
        self.monitor.start()
        
        # Update history periodically
        self.update_timer = QTimer()
        self.update_timer.timeout.connect(self.refresh_history)
        self.update_timer.start(1000)  # Update every second
    
    def init_ui(self):
        """Initialize the user interface"""
        self.setWindowTitle("BufferVault - Clipboard History Manager")
        self.setMinimumSize(800, 600)
        
        # Central widget
        central_widget = QWidget()
        self.setCentralWidget(central_widget)
        
        # Main layout
        layout = QVBoxLayout()
        central_widget.setLayout(layout)
        
        # Title
        title = QLabel("BufferVault - Your Encrypted Clipboard History")
        title.setFont(QFont("Arial", 16, QFont.Bold))
        title.setAlignment(Qt.AlignCenter)
        layout.addWidget(title)
        
        # Search bar
        search_layout = QHBoxLayout()
        search_label = QLabel("Search:")
        self.search_input = QLineEdit()
        self.search_input.setPlaceholderText("Type to search clipboard history...")
        self.search_input.textChanged.connect(self.on_search)
        
        search_layout.addWidget(search_label)
        search_layout.addWidget(self.search_input)
        layout.addLayout(search_layout)
        
        # History list
        self.history_list = QListWidget()
        self.history_list.itemDoubleClicked.connect(self.on_item_double_clicked)
        self.history_list.setAlternatingRowColors(True)
        layout.addWidget(self.history_list)
        
        # Buttons
        button_layout = QHBoxLayout()
        
        self.paste_btn = QPushButton("Paste Selected")
        self.paste_btn.clicked.connect(self.paste_selected)
        self.paste_btn.setEnabled(False)
        
        self.view_btn = QPushButton("View Full Content")
        self.view_btn.clicked.connect(self.view_selected)
        self.view_btn.setEnabled(False)
        
        self.clear_btn = QPushButton("Clear History")
        self.clear_btn.clicked.connect(self.clear_history)
        
        self.refresh_btn = QPushButton("Refresh")
        self.refresh_btn.clicked.connect(self.refresh_history)
        
        button_layout.addWidget(self.paste_btn)
        button_layout.addWidget(self.view_btn)
        button_layout.addWidget(self.clear_btn)
        button_layout.addWidget(self.refresh_btn)
        
        layout.addLayout(button_layout)
        
        # Status bar
        self.status_label = QLabel("Ready")
        layout.addWidget(self.status_label)
        
        # Connect selection change
        self.history_list.itemSelectionChanged.connect(self.on_selection_changed)
        
        # Load initial history
        self.refresh_history()
    
    def setup_system_tray(self):
        """Setup system tray icon"""
        self.tray_icon = QSystemTrayIcon(self)
        
        # Create tray menu
        tray_menu = QMenu()
        
        show_action = tray_menu.addAction("Show BufferVault")
        show_action.triggered.connect(self.show)
        
        tray_menu.addSeparator()
        
        quit_action = tray_menu.addAction("Exit")
        quit_action.triggered.connect(self.quit_application)
        
        self.tray_icon.setContextMenu(tray_menu)
        self.tray_icon.activated.connect(self.on_tray_activated)
        
        # Set icon (using default for now)
        self.tray_icon.setToolTip("BufferVault - Clipboard Manager")
        self.tray_icon.show()
    
    def on_tray_activated(self, reason):
        """Handle tray icon activation"""
        if reason == QSystemTrayIcon.DoubleClick:
            self.show()
            self.activateWindow()
    
    def refresh_history(self):
        """Refresh the history list"""
        current_selection = self.history_list.currentRow()
        
        self.history_list.clear()
        history = self.monitor.get_history(limit=self.config.get('max_history_items'))
        
        for i, entry in enumerate(history):
            item_text = f"[{entry.get_display_time()}] {entry.get_preview(80)}"
            item = QListWidgetItem(item_text)
            item.setData(Qt.UserRole, i)  # Store index
            self.history_list.addItem(item)
        
        # Restore selection if possible
        if current_selection >= 0 and current_selection < self.history_list.count():
            self.history_list.setCurrentRow(current_selection)
        
        # Update status
        count = len(history)
        self.status_label.setText(f"Total items: {count}")
    
    def on_search(self, query):
        """Handle search input"""
        if not query:
            self.refresh_history()
            return
        
        self.history_list.clear()
        results = self.monitor.search(query)
        
        for index, entry in results:
            item_text = f"[{entry.get_display_time()}] {entry.get_preview(80)}"
            item = QListWidgetItem(item_text)
            item.setData(Qt.UserRole, index)
            self.history_list.addItem(item)
        
        self.status_label.setText(f"Found {len(results)} matching items")
    
    def on_selection_changed(self):
        """Handle selection change"""
        has_selection = self.history_list.currentItem() is not None
        self.paste_btn.setEnabled(has_selection)
        self.view_btn.setEnabled(has_selection)
    
    def paste_selected(self):
        """Paste selected item to clipboard"""
        current_item = self.history_list.currentItem()
        if not current_item:
            return
        
        index = current_item.data(Qt.UserRole)
        if self.monitor.restore_to_clipboard(index):
            self.status_label.setText("Item copied to clipboard! You can now paste it anywhere.")
            QMessageBox.information(
                self,
                "Success",
                "The selected item has been copied to your clipboard.\nYou can now paste it anywhere using Ctrl+V."
            )
        else:
            QMessageBox.warning(self, "Error", "Failed to copy item to clipboard")
    
    def view_selected(self):
        """View full content of selected item"""
        current_item = self.history_list.currentItem()
        if not current_item:
            return
        
        index = current_item.data(Qt.UserRole)
        entry = self.monitor.storage.get_entry(index)
        
        if entry:
            dialog = ClipboardHistoryDialog(entry, self)
            if dialog.exec_() == QDialog.Accepted:
                # User clicked "Copy to Clipboard"
                self.monitor.restore_to_clipboard(index)
                self.status_label.setText("Item copied to clipboard!")
    
    def on_item_double_clicked(self, item):
        """Handle double click on item"""
        self.paste_selected()
    
    def clear_history(self):
        """Clear all clipboard history"""
        reply = QMessageBox.question(
            self,
            "Clear History",
            "Are you sure you want to clear all clipboard history?\nThis action cannot be undone.",
            QMessageBox.Yes | QMessageBox.No,
            QMessageBox.No
        )
        
        if reply == QMessageBox.Yes:
            self.monitor.clear_history()
            self.refresh_history()
            self.status_label.setText("History cleared")
    
    def closeEvent(self, event):
        """Handle window close event"""
        event.ignore()
        self.hide()
        self.tray_icon.showMessage(
            "BufferVault",
            "Application minimized to tray. Clipboard monitoring continues.",
            QSystemTrayIcon.Information,
            2000
        )
    
    def quit_application(self):
        """Quit the application"""
        reply = QMessageBox.question(
            self,
            "Exit BufferVault",
            "Are you sure you want to exit BufferVault?\nClipboard monitoring will stop.",
            QMessageBox.Yes | QMessageBox.No,
            QMessageBox.No
        )
        
        if reply == QMessageBox.Yes:
            self.monitor.stop()
            QApplication.quit()


def main():
    """Main entry point for GUI"""
    app = QApplication(sys.argv)
    app.setQuitOnLastWindowClosed(False)  # Keep running in tray
    
    window = BufferVaultGUI()
    window.show()
    
    sys.exit(app.exec_())


if __name__ == '__main__':
    main()
