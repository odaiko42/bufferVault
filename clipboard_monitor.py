"""
Clipboard monitoring service for BufferVault
Monitors clipboard changes and stores them in the vault
"""
import pyperclip
import time
import threading
from storage import StorageManager
from config import Config


class ClipboardMonitor:
    """Monitors clipboard and stores changes"""
    
    def __init__(self, storage_manager=None, config=None):
        self.config = config or Config()
        self.storage = storage_manager or StorageManager(
            storage_path=self.config.get('storage_path'),
            encryption_enabled=self.config.get('encryption_enabled')
        )
        
        self.running = False
        self.thread = None
        self.last_clipboard = ""
        self.check_interval = 0.5  # Check every 0.5 seconds
        
    def start(self):
        """Start monitoring clipboard"""
        if self.running:
            return
        
        self.running = True
        self.thread = threading.Thread(target=self._monitor_loop, daemon=True)
        self.thread.start()
        print("Clipboard monitoring started")
    
    def stop(self):
        """Stop monitoring clipboard"""
        self.running = False
        if self.thread:
            self.thread.join(timeout=2)
        print("Clipboard monitoring stopped")
    
    def _monitor_loop(self):
        """Main monitoring loop"""
        while self.running:
            try:
                current_clipboard = pyperclip.paste()
                
                # Check if clipboard has changed
                if current_clipboard != self.last_clipboard:
                    # Only store non-empty content
                    if current_clipboard and len(current_clipboard.strip()) > 0:
                        # Check size limit
                        max_size = self.config.get('max_item_size_mb', 10) * 1024 * 1024
                        if len(current_clipboard.encode('utf-8')) <= max_size:
                            self.storage.add_entry(current_clipboard, entry_type='text')
                            print(f"Captured: {current_clipboard[:50]}...")
                    
                    self.last_clipboard = current_clipboard
                
            except Exception as e:
                print(f"Error in clipboard monitor: {e}")
            
            time.sleep(self.check_interval)
    
    def get_history(self, limit=None):
        """Get clipboard history"""
        return self.storage.get_history(limit)
    
    def restore_to_clipboard(self, index):
        """Restore a history entry to clipboard"""
        entry = self.storage.get_entry(index)
        if entry and entry.entry_type == 'text':
            try:
                pyperclip.copy(entry.content)
                self.last_clipboard = entry.content  # Update to prevent re-capture
                return True
            except Exception as e:
                print(f"Error restoring to clipboard: {e}")
                return False
        return False
    
    def search(self, query):
        """Search clipboard history"""
        return self.storage.search_history(query)
    
    def clear_history(self):
        """Clear all history"""
        self.storage.clear_history()
        self.last_clipboard = ""
