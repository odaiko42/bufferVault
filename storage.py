"""
Storage module for BufferVault
Handles persistent storage of clipboard history
"""
import json
import os
import time
from datetime import datetime
from pathlib import Path
from encryption import EncryptionManager


class ClipboardEntry:
    """Represents a single clipboard entry"""
    
    def __init__(self, content, timestamp=None, entry_type='text', metadata=None):
        self.content = content
        self.timestamp = timestamp or time.time()
        self.entry_type = entry_type  # text, image, file, etc.
        self.metadata = metadata or {}
    
    def to_dict(self):
        """Convert entry to dictionary"""
        return {
            'content': self.content,
            'timestamp': self.timestamp,
            'entry_type': self.entry_type,
            'metadata': self.metadata
        }
    
    @staticmethod
    def from_dict(data):
        """Create entry from dictionary"""
        return ClipboardEntry(
            content=data['content'],
            timestamp=data['timestamp'],
            entry_type=data.get('entry_type', 'text'),
            metadata=data.get('metadata', {})
        )
    
    def get_display_time(self):
        """Get formatted timestamp for display"""
        dt = datetime.fromtimestamp(self.timestamp)
        return dt.strftime('%Y-%m-%d %H:%M:%S')
    
    def get_preview(self, max_length=100):
        """Get preview of content"""
        if self.entry_type == 'text':
            content_str = str(self.content)
            if len(content_str) > max_length:
                return content_str[:max_length] + '...'
            return content_str
        return f"[{self.entry_type}]"


class StorageManager:
    """Manages storage of clipboard history"""
    
    def __init__(self, storage_path='clipboard_data', encryption_enabled=True):
        self.storage_path = Path(storage_path)
        self.storage_path.mkdir(parents=True, exist_ok=True)
        self.encryption_enabled = encryption_enabled
        
        if encryption_enabled:
            self.encryption = EncryptionManager()
        else:
            self.encryption = None
        
        self.index_file = self.storage_path / 'index.json'
        self.history = self._load_history()
    
    def _load_history(self):
        """Load history from index file"""
        if not self.index_file.exists():
            return []
        
        try:
            with open(self.index_file, 'r') as f:
                data = json.load(f)
                return [ClipboardEntry.from_dict(entry) for entry in data]
        except Exception as e:
            print(f"Error loading history: {e}")
            return []
    
    def _save_history(self):
        """Save history to index file"""
        try:
            data = [entry.to_dict() for entry in self.history]
            with open(self.index_file, 'w') as f:
                json.dump(data, f, indent=2)
            return True
        except Exception as e:
            print(f"Error saving history: {e}")
            return False
    
    def add_entry(self, content, entry_type='text', metadata=None):
        """Add new clipboard entry"""
        # Check if content is the same as the last entry
        if self.history:
            last_entry = self.history[0]
            if last_entry.content == content:
                return None  # Don't add duplicate
        
        entry = ClipboardEntry(content, entry_type=entry_type, metadata=metadata)
        
        # Add to beginning of history (most recent first)
        self.history.insert(0, entry)
        
        # Store encrypted content if enabled
        if self.encryption_enabled and entry_type == 'text':
            self._store_encrypted_entry(entry)
        
        self._save_history()
        return entry
    
    def _store_encrypted_entry(self, entry):
        """Store encrypted version of entry content"""
        if not self.encryption:
            return
        
        # Create filename from timestamp
        filename = f"{int(entry.timestamp)}.vault"
        filepath = self.storage_path / filename
        
        try:
            encrypted = self.encryption.encrypt(entry.content)
            with open(filepath, 'wb') as f:
                f.write(encrypted)
        except Exception as e:
            print(f"Error storing encrypted entry: {e}")
    
    def get_history(self, limit=None):
        """Get clipboard history"""
        if limit:
            return self.history[:limit]
        return self.history
    
    def get_entry(self, index):
        """Get entry by index"""
        if 0 <= index < len(self.history):
            return self.history[index]
        return None
    
    def clear_history(self):
        """Clear all history"""
        # Remove encrypted files
        if self.encryption_enabled:
            for entry in self.history:
                filename = f"{int(entry.timestamp)}.vault"
                filepath = self.storage_path / filename
                if filepath.exists():
                    os.remove(filepath)
        
        self.history = []
        self._save_history()
    
    def remove_entry(self, index):
        """Remove specific entry"""
        if 0 <= index < len(self.history):
            entry = self.history.pop(index)
            
            # Remove encrypted file if exists
            if self.encryption_enabled:
                filename = f"{int(entry.timestamp)}.vault"
                filepath = self.storage_path / filename
                if filepath.exists():
                    os.remove(filepath)
            
            self._save_history()
            return True
        return False
    
    def search_history(self, query):
        """Search history for matching entries"""
        results = []
        query_lower = query.lower()
        
        for i, entry in enumerate(self.history):
            if entry.entry_type == 'text':
                if query_lower in entry.content.lower():
                    results.append((i, entry))
        
        return results
    
    def get_stats(self):
        """Get storage statistics"""
        return {
            'total_entries': len(self.history),
            'storage_path': str(self.storage_path),
            'encryption_enabled': self.encryption_enabled
        }
