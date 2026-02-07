"""
Configuration management for BufferVault
"""
import json
import os
from pathlib import Path


class Config:
    """Configuration manager for BufferVault"""
    
    DEFAULT_CONFIG = {
        'max_history_items': 1000,
        'auto_start': False,
        'storage_path': 'clipboard_data',
        'encryption_enabled': True,
        'hotkey': 'Ctrl+Shift+V',
        'max_item_size_mb': 10
    }
    
    def __init__(self, config_path='config.json'):
        self.config_path = config_path
        self.config = self._load_config()
    
    def _load_config(self):
        """Load configuration from file or create default"""
        if os.path.exists(self.config_path):
            try:
                with open(self.config_path, 'r') as f:
                    loaded_config = json.load(f)
                    # Merge with defaults to ensure all keys exist
                    config = self.DEFAULT_CONFIG.copy()
                    config.update(loaded_config)
                    return config
            except Exception as e:
                print(f"Error loading config: {e}")
                return self.DEFAULT_CONFIG.copy()
        return self.DEFAULT_CONFIG.copy()
    
    def save_config(self):
        """Save current configuration to file"""
        try:
            with open(self.config_path, 'w') as f:
                json.dump(self.config, f, indent=4)
            return True
        except Exception as e:
            print(f"Error saving config: {e}")
            return False
    
    def get(self, key, default=None):
        """Get configuration value"""
        return self.config.get(key, default)
    
    def set(self, key, value):
        """Set configuration value"""
        self.config[key] = value
        self.save_config()
    
    def ensure_storage_path(self):
        """Ensure storage directory exists"""
        storage_path = self.get('storage_path')
        Path(storage_path).mkdir(parents=True, exist_ok=True)
        return storage_path
